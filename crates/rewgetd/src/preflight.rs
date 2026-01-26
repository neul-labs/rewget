//! JS Preflight execution for Stage 3
//!
//! Uses headless Chromium to:
//! 1. Navigate to the target URL
//! 2. Wait for JS challenges to complete
//! 3. Extract cookies and session data
//! 4. Return content or make a clean request with obtained cookies

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use rewget_core::{chromium_installed, chromium_path, download_chromium, Request, Response, CHROMIUM_VERSION};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

/// Default wait time for JS challenges (5 seconds)
const DEFAULT_JS_WAIT_MS: u64 = 5000;

/// Maximum wait time for page load (30 seconds)
const MAX_PAGE_LOAD_MS: u64 = 30000;

/// Perform a JS preflight request using shared runtime
pub fn fetch(request: Request, runtime: &Arc<Runtime>) -> Response {
    // Auto-download Chromium if not installed (blocking operation)
    if !chromium_installed() {
        info!("Chromium not installed, downloading Chrome for Testing v{}...", CHROMIUM_VERSION);
        eprintln!("[rewget] Downloading Chrome for Testing v{} (~150MB, one-time setup)...", CHROMIUM_VERSION);

        if let Err(e) = download_chromium(|_downloaded, _total| {
            // Progress is shown by wget/curl
        }) {
            return Response::error(
                request.id,
                &format!("Failed to download Chromium: {}. You can manually run: rewget --rewget-download-chromium", e),
            );
        }

        eprintln!("[rewget] Chromium installed successfully");
        info!("Chromium installed at: {}", chromium_path().display());
    }

    runtime.block_on(async { fetch_async(request).await })
}

async fn fetch_async(request: Request) -> Response {
    info!(
        "Stage 3 request: {} {} (JS preflight)",
        request.method, request.url
    );

    let chrome_path = chromium_path();

    // Configure browser
    let config = match BrowserConfig::builder()
        .chrome_executable(&chrome_path)
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-extensions")
        .arg("--disable-background-networking")
        .arg("--disable-sync")
        .arg("--disable-translate")
        .arg("--mute-audio")
        .arg("--no-first-run")
        .arg("--safebrowsing-disable-auto-update")
        .build()
    {
        Ok(c) => c,
        Err(e) => return Response::error(request.id, &format!("Failed to configure browser: {}", e)),
    };

    // Launch browser
    let (mut browser, mut handler) = match Browser::launch(config).await {
        Ok(b) => b,
        Err(e) => return Response::error(request.id, &format!("Failed to launch browser: {}", e)),
    };

    // Spawn handler task
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            // Handle browser events
            debug!("Browser event: {:?}", event);
        }
    });

    // Create new page
    let page = match browser.new_page("about:blank").await {
        Ok(p) => p,
        Err(e) => return Response::error(request.id, &format!("Failed to create page: {}", e)),
    };

    // Navigate to URL
    debug!("Navigating to {}", request.url);
    if let Err(e) = page.goto(&request.url).await {
        return Response::error(request.id, &format!("Failed to navigate: {}", e));
    }

    // Wait for page load
    let load_timeout = Duration::from_millis(MAX_PAGE_LOAD_MS);
    if let Err(e) = tokio::time::timeout(load_timeout, page.wait_for_navigation()).await {
        warn!("Page load timeout: {}", e);
    }

    // Wait for JS challenges (configurable or default)
    let js_wait = request.js_wait.as_ref()
        .and_then(|s| parse_wait_condition(s))
        .unwrap_or(WaitCondition::Delay(DEFAULT_JS_WAIT_MS));

    match js_wait {
        WaitCondition::Delay(ms) => {
            debug!("Waiting {}ms for JS challenges", ms);
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
        WaitCondition::Selector(ref selector) => {
            debug!("Waiting for selector: {}", selector);
            // Use JavaScript to wait for selector
            let js = format!(
                r#"new Promise((resolve) => {{
                    const check = () => {{
                        if (document.querySelector('{}')) {{
                            resolve(true);
                        }} else {{
                            setTimeout(check, 100);
                        }}
                    }};
                    check();
                }})"#,
                selector.replace('\'', "\\'")
            );
            let timeout = Duration::from_millis(request.timeout_ms);
            match tokio::time::timeout(timeout, page.evaluate(js)).await {
                Ok(Ok(_)) => debug!("Selector found"),
                Ok(Err(e)) => warn!("Selector wait failed: {}", e),
                Err(_) => warn!("Selector wait timeout"),
            }
        }
        WaitCondition::NetworkIdle => {
            debug!("Waiting for network idle");
            // Simple approximation - wait a bit after load
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }
    }

    // Get final URL (in case of redirects)
    let final_url = page.url().await.unwrap_or_else(|_| Some(request.url.clone())).unwrap_or(request.url.clone());
    debug!("Final URL: {}", final_url);

    // Get page content
    let content = match page.content().await {
        Ok(c) => c,
        Err(e) => return Response::error(request.id, &format!("Failed to get content: {}", e)),
    };

    // Get cookies
    let cookies = page.get_cookies().await.unwrap_or_default();
    debug!("Got {} cookies", cookies.len());

    // Check for blocking patterns in content
    let blocked = rewget_core::analyze_body(&content, &[]).is_some();
    let block_reason = if blocked {
        rewget_core::analyze_body(&content, &[]).map(|r| format!("{}", r))
    } else {
        None
    };

    // Close browser
    let _ = browser.close().await;
    handler_task.abort();

    // Build response
    let body_bytes = content.into_bytes();

    // Write to file if output specified
    let bytes_written = if let Some(output_path) = &request.output {
        match File::create(output_path) {
            Ok(mut file) => match file.write_all(&body_bytes) {
                Ok(_) => Some(body_bytes.len() as u64),
                Err(e) => {
                    return Response::error(request.id, &format!("Failed to write output: {}", e))
                }
            },
            Err(e) => {
                return Response::error(request.id, &format!("Failed to create output: {}", e))
            }
        }
    } else {
        None
    };

    // Convert cookies to headers for response
    let mut headers = HashMap::new();
    if !cookies.is_empty() {
        let cookie_str: Vec<String> = cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();
        headers.insert("Set-Cookie".to_string(), cookie_str.join("; "));
    }
    headers.insert("X-Final-URL".to_string(), final_url);

    if blocked {
        let mut resp = Response::blocked(
            request.id,
            200, // Page loaded but contains challenge
            block_reason.as_deref().unwrap_or("JS challenge detected"),
        );
        resp.headers = headers;
        if bytes_written.is_none() {
            resp.body = Some(body_bytes);
        }
        resp.bytes_written = bytes_written;
        resp
    } else {
        let mut resp = Response::success(request.id, 200);
        resp.headers = headers;
        if bytes_written.is_none() {
            resp.body = Some(body_bytes);
        }
        resp.bytes_written = bytes_written;
        resp
    }
}

/// Wait condition for JS challenges
enum WaitCondition {
    /// Wait for a fixed delay in milliseconds
    Delay(u64),
    /// Wait for a CSS selector to appear
    Selector(String),
    /// Wait for network to be idle
    NetworkIdle,
}

/// Parse a wait condition string
fn parse_wait_condition(s: &str) -> Option<WaitCondition> {
    if s.starts_with("delay:") {
        s[6..].parse().ok().map(WaitCondition::Delay)
    } else if s.starts_with("selector:") {
        Some(WaitCondition::Selector(s[9..].to_string()))
    } else if s == "networkidle" {
        Some(WaitCondition::NetworkIdle)
    } else if let Ok(ms) = s.parse::<u64>() {
        Some(WaitCondition::Delay(ms))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wait_delay() {
        let cond = parse_wait_condition("delay:5000");
        assert!(matches!(cond, Some(WaitCondition::Delay(5000))));
    }

    #[test]
    fn test_parse_wait_selector() {
        let cond = parse_wait_condition("selector:#main-content");
        assert!(matches!(cond, Some(WaitCondition::Selector(_))));
    }

    #[test]
    fn test_parse_wait_networkidle() {
        let cond = parse_wait_condition("networkidle");
        assert!(matches!(cond, Some(WaitCondition::NetworkIdle)));
    }

    #[test]
    fn test_parse_wait_number() {
        let cond = parse_wait_condition("3000");
        assert!(matches!(cond, Some(WaitCondition::Delay(3000))));
    }
}
