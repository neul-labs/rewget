//! Execution logic for rewget
//!
//! Handles running wget in different modes:
//! - Strict mode (--rewget-no-fallback): exec() directly to wget (Unix) or spawn+wait (Windows)
//! - Default mode: spawn wget, capture output, fallback on failure

use anyhow::{Context, Result};
use rewget_core::{analyze_exit_code, extract_domain, is_fallback_code, Config, DetectionResult, DomainCache};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::daemon;

/// Run wget with the given configuration and arguments
pub fn run(config: Config, wget_args: Vec<String>) -> Result<()> {
    let engine_path = config.engine.find_binary()?;

    if config.no_fallback {
        // Strict mode: exec directly to wget (zero overhead)
        exec_wget(&engine_path, &wget_args)
    } else {
        // Default mode: spawn wget, handle fallback
        run_with_fallback(config, &engine_path, wget_args)
    }
}

/// Execute wget directly, replacing the current process
///
/// This is used in strict mode (--rewget-no-fallback) for zero overhead.
/// On Unix, uses exec() to replace the current process.
/// On Windows, spawns the process and waits for it.
#[cfg(unix)]
fn exec_wget(engine_path: &std::path::Path, wget_args: &[String]) -> Result<()> {
    let mut cmd = Command::new(engine_path);
    cmd.args(wget_args);

    // exec() replaces the current process - this never returns on success
    let err = cmd.exec();

    // If we get here, exec failed
    Err(err).context(format!("Failed to exec {}", engine_path.display()))
}

/// Execute wget directly (Windows version)
///
/// Windows doesn't have exec(), so we spawn the process and wait for it.
#[cfg(windows)]
fn exec_wget(engine_path: &std::path::Path, wget_args: &[String]) -> Result<()> {
    let mut cmd = Command::new(engine_path);
    cmd.args(wget_args);

    let status = cmd
        .status()
        .context(format!("Failed to run {}", engine_path.display()))?;

    if status.success() {
        Ok(())
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Run wget with fallback support
///
/// Captures wget output, detects failure conditions, and falls back to
/// higher stages (Stage 2: impersonation, Stage 3: JS preflight) as needed.
fn run_with_fallback(
    config: Config,
    engine_path: &std::path::Path,
    wget_args: Vec<String>,
) -> Result<()> {
    // Extract domain for caching
    let domain = find_url_in_args(&wget_args).and_then(|url| extract_domain(&url));

    // Load cache and check for cached stage
    let mut cache = if config.no_cache {
        DomainCache::default()
    } else {
        DomainCache::load()
    };

    let cached_stage = domain.as_ref().and_then(|d| cache.get(d));
    let start_stage = cached_stage.unwrap_or(1);

    if config.debug {
        eprintln!("[rewget] Engine: {} ({})", config.engine, engine_path.display());
        eprintln!("[rewget] Args: {:?}", wget_args);
        eprintln!("[rewget] Fallback: enabled (max Stage {})", config.fallback_stage);
        if let Some(d) = &domain {
            eprintln!("[rewget] Domain: {}", d);
        }
        if let Some(s) = cached_stage {
            eprintln!("[rewget] Cached stage: {} (skipping lower stages)", s);
        }
    }

    // If we have a cached stage > 1, we would skip Stage 1
    // For now, we still run Stage 1 since higher stages aren't implemented
    let current_stage = start_stage.min(1); // Clamp to 1 until Stage 2 implemented

    if current_stage == 1 {
        // Stage 1: Run plain wget with output capture
        let (exit_code, stderr_output) = run_wget_stage1(&config, engine_path, &wget_args)?;

        // Analyze the result
        let detection = analyze_exit_code(exit_code, &stderr_output);

        if config.debug {
            eprintln!("[rewget] Exit code: {}", exit_code);
            if let Some(status) = detection.status_code {
                eprintln!("[rewget] HTTP status: {}", status);
            }
            eprintln!("[rewget] Blocked: {}", detection.blocked);
        }

        // Check if we should fall back
        let should_fallback = should_trigger_fallback(&config, &detection);

        if !should_fallback && exit_code == 0 {
            // Success! Update cache if we have a domain
            if let Some(d) = &domain {
                if !config.no_cache {
                    cache.set(d, 1);
                    let _ = cache.save(); // Ignore save errors
                }
            }
            return Ok(());
        }

        if should_fallback && config.fallback_stage >= 2 {
            // Print fallback message
            print_fallback_message(&detection, 2);

            // Try Stage 2 via daemon
            let url = find_url_in_args(&wget_args);
            let output = find_output_in_args(&wget_args);

            if let Some(url) = url {
                match run_stage2(&config, &url, output.clone(), &domain, &mut cache) {
                    Ok(true) => return Ok(()), // Stage 2 succeeded
                    Ok(false) => {
                        // Stage 2 was blocked, try Stage 3 if available
                        if config.fallback_stage >= 3 {
                            print_fallback_message(&detection, 3);

                            match run_stage3(&config, &url, output, &domain, &mut cache) {
                                Ok(true) => return Ok(()), // Stage 3 succeeded
                                Ok(false) => {
                                    eprintln!("[rewget] All stages exhausted, request still blocked");
                                    std::process::exit(exit_code);
                                }
                                Err(e) => {
                                    eprintln!("[rewget] Stage 3 failed: {}", e);
                                    std::process::exit(exit_code);
                                }
                            }
                        }
                        std::process::exit(exit_code);
                    }
                    Err(e) => {
                        eprintln!("[rewget] Stage 2 failed: {}", e);
                        std::process::exit(exit_code);
                    }
                }
            } else {
                eprintln!("[rewget] No URL found in arguments");
                std::process::exit(exit_code);
            }
        }

        if exit_code != 0 {
            std::process::exit(exit_code);
        }
    }

    Ok(())
}

/// Find the URL in wget arguments
///
/// Looks for arguments that look like URLs (contain :// or start with www.)
fn find_url_in_args(args: &[String]) -> Option<String> {
    for arg in args {
        // Skip flags
        if arg.starts_with('-') {
            continue;
        }

        // Check if it looks like a URL
        if arg.contains("://") || arg.starts_with("www.") {
            return Some(arg.clone());
        }
    }

    None
}

/// Find the output file in wget arguments (-O or --output-document)
fn find_output_in_args(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-O" || arg == "--output-document" {
            if let Some(path) = iter.next() {
                if path != "-" && path != "/dev/null" {
                    return Some(PathBuf::from(path));
                }
            }
        } else if let Some(path) = arg.strip_prefix("-O") {
            if !path.is_empty() && path != "-" && path != "/dev/null" {
                return Some(PathBuf::from(path));
            }
        } else if let Some(path) = arg.strip_prefix("--output-document=") {
            if !path.is_empty() && path != "-" && path != "/dev/null" {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}

/// Run Stage 2 via daemon
///
/// Returns Ok(true) if successful, Ok(false) if still blocked, Err on error
fn run_stage2(
    config: &Config,
    url: &str,
    output: Option<PathBuf>,
    domain: &Option<String>,
    cache: &mut DomainCache,
) -> Result<bool> {
    // Ensure daemon is running
    daemon::ensure_running()?;

    if config.debug {
        eprintln!("[rewget] Stage 2: Requesting via daemon with browser impersonation");
    }

    // Make the request
    let response = daemon::stage2_fetch(
        url,
        output.clone(),
        config.profile.clone(),
        config.timeout_stage2,
    )?;

    if config.debug {
        eprintln!("[rewget] Stage 2 response: success={}, status={:?}, blocked={}",
            response.success, response.status_code, response.blocked);
    }

    if response.success && !response.blocked {
        // Success! Update cache
        if let Some(d) = domain {
            if !config.no_cache {
                cache.set(d, 2);
                let _ = cache.save();
            }
        }

        // Print success info
        if let Some(status) = response.status_code {
            if !config.quiet {
                eprintln!("[rewget] Stage 2 succeeded: HTTP {}", status);
            }
        }

        // If no output file, write body to stdout
        if output.is_none() {
            if let Some(body) = response.body {
                let _ = std::io::stdout().write_all(&body);
            }
        } else if let Some(bytes) = response.bytes_written {
            if !config.quiet {
                eprintln!("[rewget] Saved {} bytes", bytes);
            }
        }

        Ok(true)
    } else if response.blocked {
        if !config.quiet {
            eprintln!("[rewget] Stage 2 blocked: {:?}", response.block_reason);
        }
        Ok(false)
    } else {
        // Error
        if let Some(error) = response.error {
            anyhow::bail!("{}", error);
        }
        Ok(false)
    }
}

/// Run Stage 3 via daemon (JS preflight with headless Chromium)
///
/// Returns Ok(true) if successful, Ok(false) if still blocked, Err on error
fn run_stage3(
    config: &Config,
    url: &str,
    output: Option<PathBuf>,
    domain: &Option<String>,
    cache: &mut DomainCache,
) -> Result<bool> {
    // Ensure daemon is running
    daemon::ensure_running()?;

    if config.debug {
        eprintln!("[rewget] Stage 3: Requesting via daemon with JS preflight (headless Chromium)");
    }

    // Make the request
    let response = daemon::stage3_fetch(
        url,
        output.clone(),
        config.js_wait.clone(),
        config.timeout_stage3,
    )?;

    if config.debug {
        eprintln!("[rewget] Stage 3 response: success={}, status={:?}, blocked={}",
            response.success, response.status_code, response.blocked);
    }

    if response.success && !response.blocked {
        // Success! Update cache
        if let Some(d) = domain {
            if !config.no_cache {
                cache.set(d, 3);
                let _ = cache.save();
            }
        }

        // Print success info
        if !config.quiet {
            eprintln!("[rewget] Stage 3 succeeded (JS challenge bypassed)");
        }

        // If no output file, write body to stdout
        if output.is_none() {
            if let Some(body) = response.body {
                let _ = std::io::stdout().write_all(&body);
            }
        } else if let Some(bytes) = response.bytes_written {
            if !config.quiet {
                eprintln!("[rewget] Saved {} bytes", bytes);
            }
        }

        Ok(true)
    } else if response.blocked {
        if !config.quiet {
            eprintln!("[rewget] Stage 3 blocked: {:?}", response.block_reason);
        }
        Ok(false)
    } else {
        // Error
        if let Some(error) = response.error {
            anyhow::bail!("{}", error);
        }
        Ok(false)
    }
}

/// Determine if we should trigger fallback to a higher stage
fn should_trigger_fallback(config: &Config, detection: &DetectionResult) -> bool {
    // Check if blocked due to status code
    if detection.blocked {
        return true;
    }

    // Check if status code is in the configured fallback list
    if let Some(code) = detection.status_code {
        if is_fallback_code(code, &config.fallback_codes) {
            return true;
        }
    }

    false
}

/// Print a message to stderr when falling back to a higher stage
fn print_fallback_message(detection: &DetectionResult, target_stage: u8) {
    let reason = match &detection.reason {
        Some(r) => format!("{}", r),
        None => match detection.status_code {
            Some(code) => format!("HTTP {}", code),
            None => format!("exit code {}", detection.exit_code),
        },
    };

    eprintln!(
        "[rewget] {} detected, falling back to Stage {}",
        reason, target_stage
    );
}

/// Run Stage 1: Plain wget
///
/// Captures stderr for failure detection while still displaying it to the user.
/// Returns (exit_code, stderr_content).
fn run_wget_stage1(
    config: &Config,
    engine_path: &std::path::Path,
    wget_args: &[String],
) -> Result<(i32, String)> {
    let mut cmd = Command::new(engine_path);
    cmd.args(wget_args);

    // Pass through stdin and stdout, but capture stderr for detection
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context(format!("Failed to spawn {}", engine_path.display()))?;

    // Read stderr while also printing it to the user
    let stderr = child.stderr.take().expect("stderr was piped");
    let mut stderr_content = String::new();
    let reader = BufReader::new(stderr);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Print to user's stderr
                if !config.quiet {
                    let _ = writeln!(std::io::stderr(), "{}", line);
                }
                // Accumulate for detection
                stderr_content.push_str(&line);
                stderr_content.push('\n');
            }
            Err(_) => break,
        }
    }

    let status = child.wait().context("Failed to wait for wget")?;
    let exit_code = status.code().unwrap_or(1);

    Ok((exit_code, stderr_content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rewget_core::Config;

    #[test]
    fn test_should_trigger_fallback_blocked() {
        let config = Config::default();
        let detection = DetectionResult {
            blocked: true,
            status_code: Some(403),
            reason: Some(rewget_core::BlockReason::StatusCode(403)),
            exit_code: 8,
        };
        assert!(should_trigger_fallback(&config, &detection));
    }

    #[test]
    fn test_should_trigger_fallback_configured_code() {
        let config = Config::default(); // Includes 403 in fallback_codes
        let detection = DetectionResult {
            blocked: false, // Not auto-detected as blocked
            status_code: Some(403),
            reason: None,
            exit_code: 8,
        };
        assert!(should_trigger_fallback(&config, &detection));
    }

    #[test]
    fn test_should_not_fallback_success() {
        let config = Config::default();
        let detection = DetectionResult {
            blocked: false,
            status_code: Some(200),
            reason: None,
            exit_code: 0,
        };
        assert!(!should_trigger_fallback(&config, &detection));
    }

    #[test]
    fn test_should_not_fallback_404() {
        let config = Config::default();
        let detection = DetectionResult {
            blocked: false,
            status_code: Some(404),
            reason: None,
            exit_code: 8,
        };
        // 404 is not in default fallback codes (it's a legitimate "not found")
        assert!(!should_trigger_fallback(&config, &detection));
    }

    #[test]
    fn test_find_url_in_args_https() {
        let args = vec![
            "-O".to_string(),
            "output.txt".to_string(),
            "https://example.com/file.txt".to_string(),
        ];
        assert_eq!(
            find_url_in_args(&args),
            Some("https://example.com/file.txt".to_string())
        );
    }

    #[test]
    fn test_find_url_in_args_www() {
        let args = vec!["www.example.com".to_string()];
        assert_eq!(
            find_url_in_args(&args),
            Some("www.example.com".to_string())
        );
    }

    #[test]
    fn test_find_url_in_args_no_url() {
        let args = vec!["-O".to_string(), "output.txt".to_string()];
        assert_eq!(find_url_in_args(&args), None);
    }
}
