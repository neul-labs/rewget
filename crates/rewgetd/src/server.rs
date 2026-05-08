//! nng IPC server for rewgetd

use anyhow::{Context, Result};
use nng::options::Options;
use nng::{Protocol, Socket};
use rewget_core::{socket_path, DaemonStatus, FetchStage, Request, Response};
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tracing::{debug, error, info, warn};

use crate::impersonate;

/// Default idle timeout in seconds (5 minutes)
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// Get idle timeout from environment or use default
fn get_idle_timeout() -> Duration {
    std::env::var("RWGETD_IDLE_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS))
}

/// Run the nng server with shared runtime
pub fn run(runtime: Arc<Runtime>) -> Result<()> {
    let socket_path = socket_path();
    let socket_url = format!("ipc://{}", socket_path.display());

    // Ensure socket directory exists
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove stale socket file if it exists
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    // Create the rep0 (reply) socket
    let socket = Socket::new(Protocol::Rep0).context("Failed to create nng socket")?;

    // Set receive timeout for idle detection
    let idle_timeout = get_idle_timeout();
    socket
        .set_opt::<nng::options::RecvTimeout>(Some(idle_timeout))
        .context("Failed to set recv timeout")?;

    socket
        .listen(&socket_url)
        .context(format!("Failed to listen on {}", socket_url))?;

    info!(
        "Listening on {} (idle timeout: {:?})",
        socket_url, idle_timeout
    );

    // Track last activity for graceful shutdown
    let mut last_activity = Instant::now();

    // Main loop with idle timeout
    loop {
        match socket.recv() {
            Ok(msg) => {
                last_activity = Instant::now();
                let response = handle_message(&msg, &runtime);
                let response_bytes = serde_json::to_vec(&response).unwrap_or_else(|e| {
                    error!("Failed to serialize response: {}", e);
                    b"{}".to_vec()
                });

                if let Err((_, e)) = socket.send(&response_bytes) {
                    error!("Failed to send response: {}", e);
                }
            }
            Err(nng::Error::TimedOut) => {
                // Check if we've been idle long enough
                if last_activity.elapsed() >= idle_timeout {
                    info!("Idle timeout reached ({:?}), shutting down", idle_timeout);
                    break;
                }
                // Otherwise continue waiting (spurious timeout or racing condition)
            }
            Err(e) => {
                error!("Failed to receive message: {}", e);
            }
        }
    }

    // Cleanup socket file
    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    info!("Daemon shutdown complete");
    Ok(())
}

/// Handle an incoming message
fn handle_message(msg: &[u8], runtime: &Arc<Runtime>) -> Response {
    // Try to parse as a Request
    match serde_json::from_slice::<Request>(msg) {
        Ok(request) => {
            debug!("Received request {} for {}", request.id, request.url);
            handle_request(request, runtime)
        }
        Err(e) => {
            // Check if it's a status request
            if let Ok(s) = std::str::from_utf8(msg) {
                if s.trim() == "status" {
                    return handle_status();
                }
            }

            warn!("Failed to parse request: {}", e);
            Response::error(0, &format!("Invalid request: {}", e))
        }
    }
}

/// Handle a fetch request
fn handle_request(request: Request, runtime: &Arc<Runtime>) -> Response {
    match request.stage {
        FetchStage::Impersonate => impersonate::fetch(request, runtime),
        FetchStage::Preflight => crate::preflight::fetch(request, runtime),
        FetchStage::Wget => Response::error(
            request.id,
            "Stage 1 (wget) should not be requested from daemon",
        ),
    }
}

/// Handle a status request
fn handle_status() -> Response {
    let chromium_status = rewget_core::ChromiumStatus::check();

    let status = DaemonStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        stage2_available: true,
        stage3_available: chromium_status.installed,
        chromium_path: if chromium_status.installed {
            Some(chromium_status.path)
        } else {
            None
        },
        profiles: impersonate::available_profiles(),
    };

    // Encode status as a response with body
    let mut response = Response::success(0, 200);
    response.body = Some(serde_json::to_vec(&status).unwrap_or_default());
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idle_timeout_default() {
        // Without env var, should return default
        std::env::remove_var("RWGETD_IDLE_TIMEOUT");
        let timeout = get_idle_timeout();
        assert_eq!(timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_handle_invalid_message() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let response = handle_message(b"invalid json", &runtime);
        assert!(!response.success);
        assert!(response.error.is_some());
    }
}
