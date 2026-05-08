//! Daemon client for communicating with rewgetd
//!
//! Handles IPC to rewgetd for Stage 2/3 requests.

use anyhow::{Context, Result};
use nng::options::Options;
use nng::{Message, Protocol, Socket};
use rewget_core::{socket_path, FetchStage, Request, Response};
use std::path::PathBuf;
use std::time::Duration;

/// Check if the daemon is running
pub fn is_running() -> bool {
    let socket_path = socket_path();
    socket_path.exists() && ping().is_ok()
}

/// Ping the daemon to check connectivity
fn ping() -> Result<()> {
    let socket = connect()?;
    let msg = Message::from(b"status".as_slice());
    socket
        .send(msg)
        .map_err(|(_, e)| anyhow::anyhow!("Send failed: {}", e))?;
    socket
        .recv()
        .map_err(|e| anyhow::anyhow!("Recv failed: {}", e))?;
    Ok(())
}

/// Connect to the daemon
fn connect() -> Result<Socket> {
    let socket_path = socket_path();
    let socket_url = format!("ipc://{}", socket_path.display());

    let socket = Socket::new(Protocol::Req0).context("Failed to create nng socket")?;

    // Set receive timeout
    socket
        .set_opt::<nng::options::RecvTimeout>(Some(Duration::from_secs(30)))
        .context("Failed to set recv timeout")?;

    socket
        .dial(&socket_url)
        .context(format!("Failed to connect to {}", socket_url))?;

    Ok(socket)
}

/// Send a request to the daemon and get a response
pub fn send_request(request: &Request) -> Result<Response> {
    let socket = connect()?;

    let request_bytes = serde_json::to_vec(request).context("Failed to serialize request")?;

    let msg = Message::from(request_bytes.as_slice());
    socket
        .send(msg)
        .map_err(|(_, e)| anyhow::anyhow!("Failed to send request: {}", e))?;

    let response_bytes = socket.recv().context("Failed to receive response")?;

    let response: Response =
        serde_json::from_slice(&response_bytes).context("Failed to deserialize response")?;

    Ok(response)
}

/// Execute Stage 2 request via daemon
pub fn stage2_fetch(
    url: &str,
    output: Option<PathBuf>,
    profile: Option<String>,
    timeout_ms: u64,
) -> Result<Response> {
    let request = Request::get(url)
        .with_stage(FetchStage::Impersonate)
        .with_timeout(timeout_ms);

    let request = if let Some(path) = output {
        request.with_output(path)
    } else {
        request
    };

    let request = if let Some(p) = profile {
        request.with_profile(&p)
    } else {
        request
    };

    send_request(&request)
}

/// Execute Stage 3 request via daemon (JS preflight)
pub fn stage3_fetch(
    url: &str,
    output: Option<PathBuf>,
    js_wait: Option<String>,
    timeout_ms: u64,
) -> Result<Response> {
    let mut request = Request::get(url)
        .with_stage(FetchStage::Preflight)
        .with_timeout(timeout_ms);

    if let Some(path) = output {
        request = request.with_output(path);
    }

    if let Some(wait) = js_wait {
        request.js_wait = Some(wait);
    }

    send_request(&request)
}

/// Spawn the daemon if not running
pub fn ensure_running() -> Result<()> {
    if is_running() {
        return Ok(());
    }

    // Try to spawn the daemon
    spawn_daemon()?;

    // Wait for it to be ready
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if is_running() {
            return Ok(());
        }
    }

    anyhow::bail!("Daemon failed to start within 5 seconds")
}

/// Spawn the daemon process
fn spawn_daemon() -> Result<()> {
    // Find the daemon binary
    let daemon_path = find_daemon_binary()?;

    let log_path = rewget_core::DomainCache::cache_dir().join("rewgetd.log");
    let stderr = if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        std::process::Stdio::from(file)
    } else {
        std::process::Stdio::null()
    };

    std::process::Command::new(&daemon_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(stderr)
        .spawn()
        .context(format!("Failed to spawn {}", daemon_path.display()))?;

    Ok(())
}

/// Find the rewgetd binary
fn find_daemon_binary() -> Result<PathBuf> {
    // First, check if rewgetd is in the same directory as rewget
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let daemon_path = parent.join("rewgetd");
            if daemon_path.exists() {
                return Ok(daemon_path);
            }
        }
    }

    // Check PATH
    if let Ok(path) = which::which("rewgetd") {
        return Ok(path);
    }

    anyhow::bail!("rewgetd not found. Install it or add it to your PATH.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_exists() {
        // Just check we can get a path
        let path = socket_path();
        assert!(path.to_string_lossy().contains("rewgetd"));
    }
}
