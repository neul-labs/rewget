//! IPC protocol for rwget <-> rwgetd communication
//!
//! Uses nng for request/reply pattern over Unix domain sockets (or named pipes on Windows).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Default socket path for IPC
///
/// On Unix: Uses runtime dir or cache dir with .sock extension
/// On Windows: Uses local app data with .pipe extension
pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        dirs::runtime_dir()
            .or_else(|| dirs::cache_dir())
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("rwget")
            .join("rwgetd.sock")
    }

    #[cfg(windows)]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("rwget")
            .join("rwgetd.pipe")
    }

    #[cfg(not(any(unix, windows)))]
    {
        PathBuf::from("/tmp/rwget/rwgetd.sock")
    }
}

/// Request from rwget to rwgetd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Request ID for correlation
    pub id: u64,

    /// The URL to fetch
    pub url: String,

    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request headers
    pub headers: HashMap<String, String>,

    /// Request body (for POST, etc.)
    pub body: Option<Vec<u8>>,

    /// Browser profile to impersonate
    pub profile: Option<String>,

    /// Output file path (None = stdout)
    pub output: Option<PathBuf>,

    /// Timeout in milliseconds
    pub timeout_ms: u64,

    /// Stage to execute (2 or 3)
    pub stage: u8,

    /// JS wait condition for Stage 3
    pub js_wait: Option<String>,
}

impl Request {
    /// Create a new GET request
    pub fn get(url: &str) -> Self {
        Self {
            id: 0,
            url: url.to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            profile: None,
            output: None,
            timeout_ms: 15000,
            stage: 2,
            js_wait: None,
        }
    }

    /// Set the browser profile
    pub fn with_profile(mut self, profile: &str) -> Self {
        self.profile = Some(profile.to_string());
        self
    }

    /// Set the output path
    pub fn with_output(mut self, path: PathBuf) -> Self {
        self.output = Some(path);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set stage
    pub fn with_stage(mut self, stage: u8) -> Self {
        self.stage = stage;
        self
    }
}

/// Response from rwgetd to rwget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID for correlation
    pub id: u64,

    /// Whether the request succeeded
    pub success: bool,

    /// HTTP status code (if applicable)
    pub status_code: Option<u16>,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body (if not written to file)
    pub body: Option<Vec<u8>>,

    /// Bytes written to file (if output was specified)
    pub bytes_written: Option<u64>,

    /// Error message (if !success)
    pub error: Option<String>,

    /// Whether the response appears to be blocked
    pub blocked: bool,

    /// Block reason (if blocked)
    pub block_reason: Option<String>,
}

impl Response {
    /// Create a success response
    pub fn success(id: u64, status_code: u16) -> Self {
        Self {
            id,
            success: true,
            status_code: Some(status_code),
            headers: HashMap::new(),
            body: None,
            bytes_written: None,
            error: None,
            blocked: false,
            block_reason: None,
        }
    }

    /// Create an error response
    pub fn error(id: u64, error: &str) -> Self {
        Self {
            id,
            success: false,
            status_code: None,
            headers: HashMap::new(),
            body: None,
            bytes_written: None,
            error: Some(error.to_string()),
            blocked: false,
            block_reason: None,
        }
    }

    /// Create a blocked response
    pub fn blocked(id: u64, status_code: u16, reason: &str) -> Self {
        Self {
            id,
            success: false,
            status_code: Some(status_code),
            headers: HashMap::new(),
            body: None,
            bytes_written: None,
            error: None,
            blocked: true,
            block_reason: Some(reason.to_string()),
        }
    }
}

/// Daemon status for health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Version of the daemon
    pub version: String,

    /// Whether Stage 2 (impersonation) is available
    pub stage2_available: bool,

    /// Whether Stage 3 (JS preflight) is available
    pub stage3_available: bool,

    /// Chromium path (if Stage 3 available)
    pub chromium_path: Option<PathBuf>,

    /// Available browser profiles
    pub profiles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let req = Request::get("https://example.com")
            .with_profile("chrome131")
            .with_timeout(5000)
            .with_stage(2);

        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.profile, Some("chrome131".to_string()));
        assert_eq!(req.timeout_ms, 5000);
        assert_eq!(req.stage, 2);
    }

    #[test]
    fn test_response_success() {
        let resp = Response::success(1, 200);
        assert!(resp.success);
        assert_eq!(resp.status_code, Some(200));
        assert!(!resp.blocked);
    }

    #[test]
    fn test_response_blocked() {
        let resp = Response::blocked(1, 403, "Cloudflare");
        assert!(!resp.success);
        assert!(resp.blocked);
        assert_eq!(resp.block_reason, Some("Cloudflare".to_string()));
    }

    #[test]
    fn test_socket_path() {
        let path = socket_path();
        assert!(path.to_string_lossy().contains("rwgetd.sock"));
    }
}
