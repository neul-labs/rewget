//! Browser impersonation using rquest
//!
//! Provides TLS and HTTP/2 fingerprint impersonation to bypass bot detection.

use rquest::Client;
use rquest_util::Emulation;
use rwget_core::{analyze_body, Request, Response};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Available browser profiles
pub fn available_profiles() -> Vec<String> {
    vec![
        "chrome131".to_string(),
        "chrome130".to_string(),
        "chrome126".to_string(),
        "chrome120".to_string(),
        "chrome116".to_string(),
        "firefox136".to_string(),
        "firefox133".to_string(),
        "firefox128".to_string(),
        "safari18".to_string(),
        "safari16".to_string(),
        "edge131".to_string(),
        "edge127".to_string(),
    ]
}

/// Get the Emulation variant for a profile name
fn get_emulation(profile: Option<&str>) -> Emulation {
    match profile {
        Some("chrome131") => Emulation::Chrome131,
        Some("chrome130") => Emulation::Chrome130,
        Some("chrome126") => Emulation::Chrome126,
        Some("chrome120") => Emulation::Chrome120,
        Some("chrome116") => Emulation::Chrome116,
        Some("firefox136") => Emulation::Firefox136,
        Some("firefox133") => Emulation::Firefox133,
        Some("firefox128") => Emulation::Firefox128,
        Some("safari18") => Emulation::Safari18,
        Some("safari16") => Emulation::Safari16,
        Some("edge131") => Emulation::Edge131,
        Some("edge127") => Emulation::Edge127,
        _ => Emulation::Chrome131, // Default to latest Chrome
    }
}

/// Perform an impersonated HTTP request
pub fn fetch(request: Request) -> Response {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Response::error(request.id, &format!("Failed to create runtime: {}", e)),
    };

    runtime.block_on(async { fetch_async(request).await })
}

async fn fetch_async(request: Request) -> Response {
    let emulation = get_emulation(request.profile.as_deref());
    let profile_name = request.profile.as_deref().unwrap_or("chrome131");

    info!(
        "Stage 2 request: {} {} (profile: {})",
        request.method, request.url, profile_name
    );

    // Build the client with emulation (browser fingerprint impersonation)
    let client = match Client::builder()
        .emulation(emulation)
        .timeout(Duration::from_millis(request.timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Response::error(request.id, &format!("Failed to create client: {}", e)),
    };

    // Build the request
    let mut req_builder = match request.method.as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "HEAD" => client.head(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        _ => return Response::error(request.id, &format!("Unsupported method: {}", request.method)),
    };

    // Add headers
    for (key, value) in &request.headers {
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    // Add body if present
    if let Some(body) = request.body {
        req_builder = req_builder.body(body);
    }

    // Execute the request
    let resp = match req_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Request failed: {}", e);
            return Response::error(request.id, &format!("Request failed: {}", e));
        }
    };

    let status = resp.status().as_u16();
    debug!("Response status: {}", status);

    // Extract headers
    let mut headers = std::collections::HashMap::new();
    for (key, value) in resp.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    // Check if blocked by status code
    let blocked_by_status = matches!(status, 403 | 429 | 503 | 520..=529);

    // Get the body
    let body_bytes = match resp.bytes().await {
        Ok(b) => b.to_vec(),
        Err(e) => {
            warn!("Failed to read body: {}", e);
            return Response::error(request.id, &format!("Failed to read body: {}", e));
        }
    };

    // Check for body patterns indicating blocks
    let body_str = String::from_utf8_lossy(&body_bytes);
    let blocked_by_body = analyze_body(&body_str, &[]).is_some();
    let blocked = blocked_by_status || blocked_by_body;

    let block_reason = if blocked_by_status {
        Some(format!("HTTP {}", status))
    } else if blocked_by_body {
        analyze_body(&body_str, &[]).map(|r| format!("{}", r))
    } else {
        None
    };

    // Write to file if output specified
    let bytes_written = if let Some(output_path) = &request.output {
        match File::create(output_path) {
            Ok(mut file) => {
                match file.write_all(&body_bytes) {
                    Ok(_) => Some(body_bytes.len() as u64),
                    Err(e) => {
                        warn!("Failed to write output: {}", e);
                        return Response::error(request.id, &format!("Failed to write output: {}", e));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create output file: {}", e);
                return Response::error(request.id, &format!("Failed to create output file: {}", e));
            }
        }
    } else {
        None
    };

    if blocked {
        let mut resp = Response::blocked(request.id, status, block_reason.as_deref().unwrap_or("Unknown"));
        resp.headers = headers;
        if bytes_written.is_none() {
            resp.body = Some(body_bytes);
        }
        resp.bytes_written = bytes_written;
        resp
    } else {
        let mut resp = Response::success(request.id, status);
        resp.headers = headers;
        if bytes_written.is_none() {
            resp.body = Some(body_bytes);
        }
        resp.bytes_written = bytes_written;
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_profiles() {
        let profiles = available_profiles();
        assert!(profiles.contains(&"chrome131".to_string()));
        assert!(profiles.contains(&"firefox136".to_string()));
        assert!(profiles.contains(&"safari16".to_string()));
    }

    #[test]
    fn test_get_emulation_default() {
        // Default should be Chrome 131
        let emu = get_emulation(None);
        assert!(matches!(emu, Emulation::Chrome131));
    }

    #[test]
    fn test_get_emulation_firefox() {
        let emu = get_emulation(Some("firefox136"));
        assert!(matches!(emu, Emulation::Firefox136));
    }
}
