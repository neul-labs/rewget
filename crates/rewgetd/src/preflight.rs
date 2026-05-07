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
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

/// Default wait time for JS challenges (5 seconds)
const DEFAULT_JS_WAIT_MS: u64 = 5000;

/// Maximum wait time for page load (30 seconds)
const MAX_PAGE_LOAD_MS: u64 = 30000;

/// Perform a JS preflight request using shared runtime
pub fn fetch(request: Request, runtime: &Arc<Runtime>) -> Response {
    let mut session = PreflightSession::new(request);

    // Ensure Chromium is installed before entering async
    if let Some(resp) = session.ensure_chromium() {
        return resp;
    }

    runtime.block_on(async { session.run().await })
}

/// State machine for the JS preflight lifecycle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PreflightState {
    Init,
    ChromiumReady,
    BrowserLaunching,
    PageCreated,
    Navigating,
    WaitingForJs { condition: WaitCondition, start: Instant },
    Extracting,
    Complete { response: Response },
    Failed { error: String },
}

/// Session that drives a single Stage 3 preflight request.
struct PreflightSession {
    state: PreflightState,
    request: Request,
}

impl PreflightSession {
    fn new(request: Request) -> Self {
        Self {
            state: PreflightState::Init,
            request,
        }
    }

    /// Ensure Chromium is installed (synchronous check/download).
    /// Returns `Some(Response)` if an error occurs, otherwise `None`.
    fn ensure_chromium(&mut self) -> Option<Response> {
        if !chromium_installed() {
            info!("Chromium not installed, downloading Chrome for Testing v{}...", CHROMIUM_VERSION);
            eprintln!("[rewget] Downloading Chrome for Testing v{} (~150MB, one-time setup)...", CHROMIUM_VERSION);

            if let Err(e) = download_chromium(|_downloaded, _total| {}) {
                self.state = PreflightState::Failed {
                    error: format!("Failed to download Chromium: {}", e),
                };
                return Some(Response::error(
                    self.request.id,
                    &format!("Failed to download Chromium: {}. You can manually run: rewget --rewget-download-chromium", e),
                ));
            }

            eprintln!("[rewget] Chromium installed successfully");
            info!("Chromium installed at: {}", chromium_path().display());
        }

        self.state = PreflightState::ChromiumReady;
        None
    }

    /// Run the full preflight lifecycle asynchronously.
    async fn run(mut self) -> Response {
        info!(
            "Stage 3 request: {} {} (JS preflight)",
            self.request.method, self.request.url
        );

        let chrome_path = chromium_path();

        // -- BrowserLaunching --
        self.state = PreflightState::BrowserLaunching;
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
            Err(e) => {
                self.state = PreflightState::Failed {
                    error: format!("Failed to configure browser: {}", e),
                };
                return Response::error(self.request.id, &format!("Failed to configure browser: {}", e));
            }
        };

        let (mut browser, mut handler) = match Browser::launch(config).await {
            Ok(b) => b,
            Err(e) => {
                self.state = PreflightState::Failed {
                    error: format!("Failed to launch browser: {}", e),
                };
                return Response::error(self.request.id, &format!("Failed to launch browser: {}", e));
            }
        };

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                debug!("Browser event: {:?}", event);
            }
        });

        // -- PageCreated --
        self.state = PreflightState::PageCreated;
        let page = match browser.new_page("about:blank").await {
            Ok(p) => p,
            Err(e) => {
                self.state = PreflightState::Failed {
                    error: format!("Failed to create page: {}", e),
                };
                return Response::error(self.request.id, &format!("Failed to create page: {}", e));
            }
        };

        // -- Navigating --
        self.state = PreflightState::Navigating;
        debug!("Navigating to {}", self.request.url);
        if let Err(e) = page.goto(&self.request.url).await {
            self.state = PreflightState::Failed {
                error: format!("Failed to navigate: {}", e),
            };
            return Response::error(self.request.id, &format!("Failed to navigate: {}", e));
        }

        let load_timeout = Duration::from_millis(MAX_PAGE_LOAD_MS);
        if let Err(e) = tokio::time::timeout(load_timeout, page.wait_for_navigation()).await {
            warn!("Page load timeout: {}", e);
        }

        // -- WaitingForJs --
        let js_wait = self.request.js_wait.as_ref()
            .and_then(|s| parse_wait_condition(s))
            .unwrap_or(WaitCondition::Delay(DEFAULT_JS_WAIT_MS));

        self.state = PreflightState::WaitingForJs {
            condition: js_wait.clone(),
            start: Instant::now(),
        };

        match js_wait {
            WaitCondition::Delay(ms) => {
                debug!("Waiting {}ms for JS challenges", ms);
                tokio::time::sleep(Duration::from_millis(ms)).await;
            }
            WaitCondition::Selector(ref selector) => {
                debug!("Waiting for selector: {}", selector);
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
                let timeout = Duration::from_millis(self.request.timeout_ms);
                match tokio::time::timeout(timeout, page.evaluate(js)).await {
                    Ok(Ok(_)) => debug!("Selector found"),
                    Ok(Err(e)) => warn!("Selector wait failed: {}", e),
                    Err(_) => warn!("Selector wait timeout"),
                }
            }
            WaitCondition::NetworkIdle => {
                debug!("Waiting for network idle");
                tokio::time::sleep(Duration::from_millis(2000)).await;
            }
        }

        // -- Extracting --
        self.state = PreflightState::Extracting;

        let final_url = page.url().await.unwrap_or_else(|_| Some(self.request.url.clone())).unwrap_or(self.request.url.clone());
        debug!("Final URL: {}", final_url);

        let content = match page.content().await {
            Ok(c) => c,
            Err(e) => {
                self.state = PreflightState::Failed {
                    error: format!("Failed to get content: {}", e),
                };
                return Response::error(self.request.id, &format!("Failed to get content: {}", e));
            }
        };

        let cookies = page.get_cookies().await.unwrap_or_default();
        debug!("Got {} cookies", cookies.len());

        // Check for blocking patterns in content
        let block_reason = rewget_core::analyze_body(&content, &[]).map(|r| format!("{}", r));
        let blocked = block_reason.is_some();

        // Close browser
        let _ = browser.close().await;
        handler_task.abort();

        let body_bytes = content.into_bytes();

        // Write to file if output specified
        let bytes_written = if let Some(output_path) = &self.request.output {
            match File::create(output_path) {
                Ok(mut file) => match file.write_all(&body_bytes) {
                    Ok(_) => Some(body_bytes.len() as u64),
                    Err(e) => {
                        self.state = PreflightState::Failed {
                            error: format!("Failed to write output: {}", e),
                        };
                        return Response::error(self.request.id, &format!("Failed to write output: {}", e))
                    }
                },
                Err(e) => {
                    self.state = PreflightState::Failed {
                        error: format!("Failed to create output: {}", e),
                    };
                    return Response::error(self.request.id, &format!("Failed to create output: {}", e))
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
                self.request.id,
                503, // Service Unavailable (challenge page)
                block_reason.as_deref().unwrap_or("JS challenge detected"),
            );
            resp.headers = headers;
            if bytes_written.is_none() {
                resp.body = Some(body_bytes);
            }
            resp.bytes_written = bytes_written;
            self.state = PreflightState::Complete { response: resp.clone() };
            resp
        } else {
            let mut resp = Response::success(self.request.id, 200);
            resp.headers = headers;
            if bytes_written.is_none() {
                resp.body = Some(body_bytes);
            }
            resp.bytes_written = bytes_written;
            self.state = PreflightState::Complete { response: resp.clone() };
            resp
        }
    }
}

/// Wait condition for JS challenges
#[derive(Debug, Clone)]
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
    if let Some(rest) = s.strip_prefix("delay:") {
        rest.parse().ok().map(WaitCondition::Delay)
    } else if let Some(rest) = s.strip_prefix("selector:") {
        Some(WaitCondition::Selector(rest.to_string()))
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
