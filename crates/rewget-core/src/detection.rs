//! Failure detection for rewget
//!
//! Detects when wget fails due to bot protection by analyzing:
//! 1. Exit codes (mapped to HTTP status codes)
//! 2. Response body patterns (Cloudflare, etc.)

/// Result of analyzing a wget execution
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Whether the request was blocked
    pub blocked: bool,

    /// Detected HTTP status code (if determinable)
    pub status_code: Option<u16>,

    /// Reason for detection
    pub reason: Option<BlockReason>,

    /// Raw exit code from wget
    pub exit_code: i32,
}

/// Reason why a request was detected as blocked
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// HTTP status code indicates blocking (403, 429, etc.)
    StatusCode(u16),

    /// Body pattern matched (e.g., Cloudflare challenge)
    BodyPattern(String),

    /// Connection was refused or timed out
    ConnectionFailed,
}

impl std::fmt::Display for BlockReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockReason::StatusCode(code) => {
                let desc = status_code_description(*code);
                write!(f, "{} {}", code, desc)
            }
            BlockReason::BodyPattern(pattern) => {
                write!(f, "Challenge page detected ({})", pattern)
            }
            BlockReason::ConnectionFailed => {
                write!(f, "Connection failed")
            }
        }
    }
}

/// Map wget exit codes to HTTP status information
///
/// wget exit codes: https://www.gnu.org/software/wget/manual/html_node/Exit-Status.html
/// - 0: No problems
/// - 1: Generic error
/// - 2: Parse error
/// - 3: File I/O error
/// - 4: Network failure
/// - 5: SSL verification failure
/// - 6: Username/password authentication failure
/// - 7: Protocol errors
/// - 8: Server issued an error response (we need to parse stderr for status)
pub fn analyze_exit_code(exit_code: i32, stderr: &str) -> DetectionResult {
    match exit_code {
        0 => DetectionResult {
            blocked: false,
            status_code: Some(200),
            reason: None,
            exit_code,
        },

        8 => {
            // Server error - try to extract status code from stderr
            // wget prints: "ERROR 403: Forbidden."
            if let Some(code) = extract_status_code_from_stderr(stderr) {
                DetectionResult {
                    blocked: is_blocking_status(code),
                    status_code: Some(code),
                    reason: if is_blocking_status(code) {
                        Some(BlockReason::StatusCode(code))
                    } else {
                        None
                    },
                    exit_code,
                }
            } else {
                DetectionResult {
                    blocked: false,
                    status_code: None,
                    reason: None,
                    exit_code,
                }
            }
        }

        4 => DetectionResult {
            blocked: true,
            status_code: None,
            reason: Some(BlockReason::ConnectionFailed),
            exit_code,
        },

        _ => DetectionResult {
            blocked: false,
            status_code: None,
            reason: None,
            exit_code,
        },
    }
}

/// Extract HTTP status code from wget stderr output
///
/// Looks for patterns like:
/// - "ERROR 403: Forbidden."
/// - "HTTP request sent, awaiting response... 403 Forbidden"
fn extract_status_code_from_stderr(stderr: &str) -> Option<u16> {
    // Pattern 1: "ERROR 403: ..."
    for line in stderr.lines() {
        if line.contains("ERROR") {
            if let Some(code) = extract_three_digit_code(line) {
                return Some(code);
            }
        }

        // Pattern 2: "awaiting response... 403"
        if line.contains("awaiting response") {
            if let Some(code) = extract_three_digit_code(line) {
                return Some(code);
            }
        }
    }

    None
}

/// Extract a 3-digit HTTP status code from a string
fn extract_three_digit_code(s: &str) -> Option<u16> {
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut num = String::new();
            num.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() && num.len() < 3 {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if num.len() == 3 {
                if let Ok(code) = num.parse::<u16>() {
                    if (100..=599).contains(&code) {
                        return Some(code);
                    }
                }
            }
        }
    }

    None
}

/// Check if a status code indicates blocking
fn is_blocking_status(code: u16) -> bool {
    matches!(
        code,
        403 | 429 | 503 | 520 | 521 | 522 | 523 | 524 | 525 | 526 | 527 | 528 | 529
    )
}

/// Check if a status code is in the configured list
pub fn is_fallback_code(code: u16, fallback_codes: &[u16]) -> bool {
    fallback_codes.contains(&code)
}

/// Get human-readable description for HTTP status code
fn status_code_description(code: u16) -> &'static str {
    match code {
        200 => "OK",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        // Cloudflare-specific
        520 => "Web Server Returned Unknown Error",
        521 => "Web Server Is Down",
        522 => "Connection Timed Out",
        523 => "Origin Is Unreachable",
        524 => "A Timeout Occurred",
        525 => "SSL Handshake Failed",
        526 => "Invalid SSL Certificate",
        527 => "Railgun Error",
        528 => "Railgun Error",
        529 => "Site Overloaded",
        _ => "Unknown",
    }
}

/// Known body patterns that indicate blocking/challenges
pub const BLOCK_PATTERNS: &[(&str, &str)] = &[
    ("cf-browser-verification", "Cloudflare JS challenge"),
    ("_cf_chl_opt", "Cloudflare challenge"),
    ("cf-challenge-running", "Cloudflare challenge"),
    ("Checking your browser", "Browser verification"),
    ("Please enable JavaScript", "JS required"),
    ("please enable javascript", "JS required"),
    ("Pardon Our Interruption", "Distil Networks"),
    ("press & hold", "Cloudflare Turnstile"),
    ("Just a moment", "Cloudflare waiting page"),
    ("Attention Required", "Cloudflare block"),
    ("Access denied", "Access denied"),
    ("captcha-delivery", "CAPTCHA required"),
];

/// Analyze response body for blocking patterns
pub fn analyze_body(body: &str, custom_patterns: &[String]) -> Option<BlockReason> {
    // Check built-in patterns
    for (pattern, description) in BLOCK_PATTERNS {
        if body.contains(pattern) {
            return Some(BlockReason::BodyPattern(description.to_string()));
        }
    }

    // Check custom patterns
    for pattern in custom_patterns {
        if body.contains(pattern.as_str()) {
            return Some(BlockReason::BodyPattern(format!("Custom: {}", pattern)));
        }
    }

    None
}

/// Check if response should be analyzed for body patterns
///
/// Only analyze if:
/// - Content-Type is text/html
/// - Response is small (< 100KB)
pub fn should_analyze_body(content_type: Option<&str>, content_length: Option<usize>) -> bool {
    let is_html = content_type
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);

    let is_small = content_length.map(|len| len < 100 * 1024).unwrap_or(true); // If unknown, analyze anyway

    is_html && is_small
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_status_code_error() {
        let stderr = "ERROR 403: Forbidden.";
        assert_eq!(extract_status_code_from_stderr(stderr), Some(403));
    }

    #[test]
    fn test_extract_status_code_awaiting() {
        let stderr = "HTTP request sent, awaiting response... 429 Too Many Requests";
        assert_eq!(extract_status_code_from_stderr(stderr), Some(429));
    }

    #[test]
    fn test_extract_status_code_multiline() {
        let stderr = r#"
--2024-01-15 10:00:00--  https://example.com/
Resolving example.com... 93.184.216.34
Connecting to example.com|93.184.216.34|:443... connected.
HTTP request sent, awaiting response... 503 Service Unavailable
2024-01-15 10:00:01 ERROR 503: Service Unavailable.
"#;
        assert_eq!(extract_status_code_from_stderr(stderr), Some(503));
    }

    #[test]
    fn test_analyze_exit_code_success() {
        let result = analyze_exit_code(0, "");
        assert!(!result.blocked);
        assert_eq!(result.status_code, Some(200));
    }

    #[test]
    fn test_analyze_exit_code_403() {
        let result = analyze_exit_code(8, "ERROR 403: Forbidden.");
        assert!(result.blocked);
        assert_eq!(result.status_code, Some(403));
        assert!(matches!(result.reason, Some(BlockReason::StatusCode(403))));
    }

    #[test]
    fn test_analyze_body_cloudflare() {
        let body =
            r#"<html><body>Just a moment...<script>cf-browser-verification</script></body></html>"#;
        let result = analyze_body(body, &[]);
        assert!(result.is_some());
    }

    #[test]
    fn test_analyze_body_custom_pattern() {
        let body = "Access blocked by firewall";
        let custom = vec!["blocked by firewall".to_string()];
        let result = analyze_body(body, &custom);
        assert!(result.is_some());
    }

    #[test]
    fn test_is_blocking_status() {
        assert!(is_blocking_status(403));
        assert!(is_blocking_status(429));
        assert!(is_blocking_status(503));
        assert!(is_blocking_status(520));
        assert!(!is_blocking_status(200));
        assert!(!is_blocking_status(404));
        assert!(!is_blocking_status(500));
    }

    #[test]
    fn test_block_reason_display() {
        assert_eq!(format!("{}", BlockReason::StatusCode(403)), "403 Forbidden");
        assert_eq!(
            format!("{}", BlockReason::BodyPattern("test".to_string())),
            "Challenge page detected (test)"
        );
    }
}
