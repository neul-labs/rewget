//! Execution logic for rewget
//!
//! Handles running wget in different modes:
//! - Strict mode (--rewget-no-fallback): exec() directly to wget (Unix) or spawn+wait (Windows)
//! - Default mode: spawn wget, capture output, fallback on failure

use anyhow::{Context, Result};
use rewget_core::{
    analyze_body, analyze_exit_code, extract_domain, Config, DaemonMode, DetectionResult,
    DomainCache, FetchAction, FetchOrchestrator,
};
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
#[cfg(unix)]
fn exec_wget(engine_path: &std::path::Path, wget_args: &[String]) -> Result<()> {
    let mut cmd = Command::new(engine_path);
    cmd.args(wget_args);
    let err = cmd.exec();
    Err(err).context(format!("Failed to exec {}", engine_path.display()))
}

/// Execute wget directly (Windows version)
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

/// Run wget with fallback support using the FetchOrchestrator state machine.
fn run_with_fallback(
    config: Config,
    engine_path: &std::path::Path,
    wget_args: Vec<String>,
) -> Result<()> {
    let domain = find_url_in_args(&wget_args).and_then(|url| extract_domain(&url));
    let url = find_url_in_args(&wget_args);
    let output = find_output_in_args(&wget_args);

    let cache = if config.no_cache {
        DomainCache::default()
    } else {
        DomainCache::load()
    };

    if config.debug {
        eprintln!(
            "[rewget] Engine: {} ({})",
            config.engine,
            engine_path.display()
        );
        eprintln!("[rewget] Args: {:?}", wget_args);
        eprintln!(
            "[rewget] Fallback: enabled (max Stage {})",
            config.fallback_stage.number()
        );
        if let Some(d) = &domain {
            eprintln!("[rewget] Domain: {}", d);
        }
        if let Some(s) = cache.get(domain.as_deref().unwrap_or("")) {
            eprintln!("[rewget] Cached stage: {} (skipping lower stages)", s);
        }
    }

    let mut orchestrator = FetchOrchestrator::new(config.clone(), cache, domain.clone());

    while let Some(action) = orchestrator.next_action() {
        match action {
            FetchAction::RunWget { stage } => {
                if config.debug {
                    eprintln!("[rewget] Running Stage {} ({})...", stage.number(), stage);
                }

                let (exit_code, stderr_output, stdout_output) =
                    run_wget_stage1(&config, engine_path, &wget_args)?;

                let detection = analyze_exit_code(exit_code, &stderr_output);

                // Also check body patterns if body detection is enabled and we captured stdout.
                let body_analysis = if config.body_detection {
                    if let Some(ref body) = stdout_output {
                        if let Ok(body_str) = std::str::from_utf8(body) {
                            analyze_body(body_str, &[])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let detection = if let Some(reason) = body_analysis {
                    DetectionResult {
                        blocked: true,
                        status_code: detection.status_code,
                        reason: Some(reason),
                        exit_code,
                    }
                } else {
                    detection
                };

                if config.debug {
                    eprintln!("[rewget] Exit code: {}", exit_code);
                    if let Some(status) = detection.status_code {
                        eprintln!("[rewget] HTTP status: {}", status);
                    }
                    eprintln!("[rewget] Blocked: {}", detection.blocked);
                }

                orchestrator.report_stage1(exit_code, detection, stdout_output);
            }

            FetchAction::RunImpersonate { stage, .. } => {
                if config.debug {
                    eprintln!(
                        "[rewget] Falling back to Stage {} ({})...",
                        stage.number(),
                        stage
                    );
                }

                if let Some(ref url) = url {
                    if config.daemon_mode == DaemonMode::Off {
                        orchestrator.report_stage2(
                            false,
                            true,
                            None,
                            Some("Daemon mode is off".to_string()),
                        );
                        continue;
                    }

                    match run_stage2(&config, url, output.clone(), &domain) {
                        Ok((success, blocked, status_code, reason)) => {
                            orchestrator.report_stage2(success, blocked, status_code, reason);
                        }
                        Err(e) => {
                            orchestrator.report_error(e.to_string());
                        }
                    }
                } else {
                    orchestrator.report_error("No URL found in arguments".to_string());
                }
            }

            FetchAction::RunPreflight { stage, .. } => {
                if config.debug {
                    eprintln!(
                        "[rewget] Falling back to Stage {} ({})...",
                        stage.number(),
                        stage
                    );
                }

                if let Some(ref url) = url {
                    if config.daemon_mode == DaemonMode::Off {
                        orchestrator.report_stage3(
                            false,
                            true,
                            None,
                            Some("Daemon mode is off".to_string()),
                        );
                        continue;
                    }

                    match run_stage3(&config, url, output.clone(), &domain) {
                        Ok((success, blocked, status_code, reason)) => {
                            orchestrator.report_stage3(success, blocked, status_code, reason);
                        }
                        Err(e) => {
                            orchestrator.report_error(e.to_string());
                        }
                    }
                } else {
                    orchestrator.report_error("No URL found in arguments".to_string());
                }
            }

            FetchAction::CacheHit { stage } => {
                if config.debug {
                    eprintln!(
                        "[rewget] Cache hit for Stage {} ({}), skipping lower stages",
                        stage.number(),
                        stage
                    );
                }
            }

            FetchAction::Complete { stage } => {
                if !config.quiet {
                    eprintln!("[rewget] Success at Stage {} ({})", stage.number(), stage);
                }
                // Save cache if it was modified.
                if let Some(_d) = &domain {
                    if !config.no_cache {
                        let _ = orchestrator.cache.save();
                    }
                }
                return Ok(());
            }

            FetchAction::GiveUp { last_reason } => {
                let reason = last_reason
                    .as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "Request blocked".to_string());
                eprintln!("[rewget] All stages exhausted: {}", reason);
                std::process::exit(1);
            }

            FetchAction::Fatal { error } => {
                eprintln!("[rewget] Fatal error: {}", error);
                std::process::exit(1);
            }

            FetchAction::Propagate { exit_code } => {
                if config.debug {
                    eprintln!("[rewget] Propagating exit code {}", exit_code);
                }
                std::process::exit(exit_code);
            }
        }
    }

    Ok(())
}

/// Find the URL in wget arguments
fn find_url_in_args(args: &[String]) -> Option<String> {
    for arg in args {
        if arg.starts_with('-') {
            continue;
        }
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
/// Returns (success, blocked, status_code, reason)
fn run_stage2(
    config: &Config,
    url: &str,
    output: Option<PathBuf>,
    _domain: &Option<String>,
) -> Result<(bool, bool, Option<u16>, Option<String>)> {
    if config.daemon_mode == DaemonMode::On {
        if !daemon::is_running() {
            anyhow::bail!("Daemon mode is 'on' but rewgetd is not running");
        }
    } else {
        daemon::ensure_running()?;
    }

    if config.debug {
        eprintln!("[rewget] Stage 2: Requesting via daemon with browser impersonation");
    }

    let response = daemon::stage2_fetch(
        url,
        output.clone(),
        config.profile.clone(),
        config.timeout_stage2,
    )?;

    if config.debug {
        eprintln!(
            "[rewget] Stage 2 response: success={}, status={:?}, blocked={}",
            response.success, response.status_code, response.blocked
        );
    }

    if response.success && !response.blocked {
        if output.is_none() {
            if let Some(body) = response.body {
                let _ = std::io::stdout().write_all(&body);
            }
        } else if let Some(bytes) = response.bytes_written {
            if !config.quiet {
                eprintln!("[rewget] Saved {} bytes", bytes);
            }
        }
        Ok((true, false, response.status_code, None))
    } else if response.blocked {
        Ok((false, true, response.status_code, response.block_reason))
    } else {
        Err(anyhow::anyhow!(response
            .error
            .unwrap_or_else(|| "Stage 2 failed".to_string())))
    }
}

/// Run Stage 3 via daemon (JS preflight)
///
/// Returns (success, blocked, status_code, reason)
fn run_stage3(
    config: &Config,
    url: &str,
    output: Option<PathBuf>,
    _domain: &Option<String>,
) -> Result<(bool, bool, Option<u16>, Option<String>)> {
    if config.daemon_mode == DaemonMode::On {
        if !daemon::is_running() {
            anyhow::bail!("Daemon mode is 'on' but rewgetd is not running");
        }
    } else {
        daemon::ensure_running()?;
    }

    if config.debug {
        eprintln!("[rewget] Stage 3: Requesting via daemon with JS preflight (headless Chromium)");
    }

    let response = daemon::stage3_fetch(
        url,
        output.clone(),
        config.js_wait.clone(),
        config.timeout_stage3,
    )?;

    if config.debug {
        eprintln!(
            "[rewget] Stage 3 response: success={}, status={:?}, blocked={}",
            response.success, response.status_code, response.blocked
        );
    }

    if response.success && !response.blocked {
        if output.is_none() {
            if let Some(body) = response.body {
                let _ = std::io::stdout().write_all(&body);
            }
        } else if let Some(bytes) = response.bytes_written {
            if !config.quiet {
                eprintln!("[rewget] Saved {} bytes", bytes);
            }
        }
        Ok((true, false, response.status_code, None))
    } else if response.blocked {
        Ok((false, true, response.status_code, response.block_reason))
    } else {
        Err(anyhow::anyhow!(response
            .error
            .unwrap_or_else(|| "Stage 3 failed".to_string())))
    }
}

/// Run Stage 1: Plain wget
///
/// Captures stderr for failure detection while still displaying it to the user.
/// Also optionally captures stdout if body detection is enabled.
/// Returns (exit_code, stderr_content, stdout_content).
fn run_wget_stage1(
    config: &Config,
    engine_path: &std::path::Path,
    wget_args: &[String],
) -> Result<(i32, String, Option<Vec<u8>>)> {
    let mut cmd = Command::new(engine_path);
    cmd.args(wget_args);

    // If body detection is enabled, capture stdout; otherwise inherit it.
    if config.body_detection {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    } else {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());
    }

    let mut child = cmd
        .spawn()
        .context(format!("Failed to spawn {}", engine_path.display()))?;

    // Read stderr while also printing it to the user.
    let stderr = child.stderr.take().expect("stderr was piped");
    let mut stderr_content = String::new();
    let reader = BufReader::new(stderr);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !config.quiet {
                    let _ = writeln!(std::io::stderr(), "{}", line);
                }
                stderr_content.push_str(&line);
                stderr_content.push('\n');
            }
            Err(_) => break,
        }
    }

    // Optionally capture stdout for body analysis.
    let stdout_output = if config.body_detection {
        let stdout = child.stdout.take().expect("stdout was piped");
        let mut stdout_buf = Vec::new();
        let mut stdout_reader = BufReader::new(stdout);
        let _ = std::io::copy(&mut stdout_reader, &mut stdout_buf);
        Some(stdout_buf)
    } else {
        None
    };

    let status = child.wait().context("Failed to wait for wget")?;
    let exit_code = status.code().unwrap_or(1);

    Ok((exit_code, stderr_content, stdout_output))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(find_url_in_args(&args), Some("www.example.com".to_string()));
    }

    #[test]
    fn test_find_url_in_args_no_url() {
        let args = vec!["-O".to_string(), "output.txt".to_string()];
        assert_eq!(find_url_in_args(&args), None);
    }
}
