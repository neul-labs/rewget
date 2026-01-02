//! nng IPC server for rwgetd

use anyhow::{Context, Result};
use nng::{Protocol, Socket};
use rwget_core::{socket_path, Request, Response, DaemonStatus};
use std::fs;
use tracing::{debug, error, info, warn};

use crate::impersonate;

/// Run the nng server
pub fn run() -> Result<()> {
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
    let socket = Socket::new(Protocol::Rep0)
        .context("Failed to create nng socket")?;

    socket.listen(&socket_url)
        .context(format!("Failed to listen on {}", socket_url))?;

    info!("Listening on {}", socket_url);

    // Main loop
    loop {
        match socket.recv() {
            Ok(msg) => {
                let response = handle_message(&msg);
                let response_bytes = serde_json::to_vec(&response)
                    .unwrap_or_else(|e| {
                        error!("Failed to serialize response: {}", e);
                        b"{}".to_vec()
                    });

                if let Err((_, e)) = socket.send(&response_bytes) {
                    error!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to receive message: {}", e);
            }
        }
    }
}

/// Handle an incoming message
fn handle_message(msg: &[u8]) -> Response {
    // Try to parse as a Request
    match serde_json::from_slice::<Request>(msg) {
        Ok(request) => {
            debug!("Received request {} for {}", request.id, request.url);
            handle_request(request)
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
fn handle_request(request: Request) -> Response {
    match request.stage {
        2 => impersonate::fetch(request),
        3 => crate::preflight::fetch(request),
        _ => Response::error(request.id, &format!("Invalid stage: {}", request.stage)),
    }
}

/// Handle a status request
fn handle_status() -> Response {
    let chromium_status = rwget_core::ChromiumStatus::check();

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
    fn test_handle_invalid_message() {
        let response = handle_message(b"invalid json");
        assert!(!response.success);
        assert!(response.error.is_some());
    }
}
