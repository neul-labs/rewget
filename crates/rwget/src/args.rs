//! Argument parsing for rwget
//!
//! Separates --rwget-* flags from wget arguments while preserving order.

use anyhow::{anyhow, Result};
use rwget_core::{Config, DaemonMode, Engine};

/// Parsed command-line arguments
#[derive(Debug)]
pub struct Args {
    /// rwget configuration from parsed flags
    pub config: Config,

    /// Command to execute
    pub command: Command,

    /// Arguments to pass to wget (non-rwget flags)
    pub wget_args: Vec<String>,
}

/// Command to execute
#[derive(Debug)]
pub enum Command {
    /// Run wget with the given arguments
    Run,
    /// Print version and exit
    Version,
    /// Print help and exit
    Help,
    /// Clear stage cache and exit
    ClearCache,
    /// List available profiles and exit
    ListProfiles,
    /// Update profiles and exit (with optional URL)
    UpdateProfiles {
        url: Option<String>,
        no_verify: bool,
    },
    /// Download Chromium and exit
    DownloadChromium,
    /// Print Chromium path and exit
    ChromiumPath,
    /// Verify a profile and exit
    VerifyProfile(String),
    /// Generate shell completions
    GenerateCompletions(String),
}

impl Args {
    /// Parse command-line arguments
    pub fn parse(args: Vec<String>) -> Result<Self> {
        let mut config = Config::default();
        let mut command = Command::Run;
        let mut wget_args = Vec::new();

        // Track profile update options
        let mut profile_url: Option<String> = None;
        let mut profile_no_verify = false;
        let mut is_update_profiles = false;

        // Check environment for engine
        if let Ok(engine_str) = std::env::var("RWGET_ENGINE") {
            config.engine = Engine::from_str(&engine_str)?;
        }

        // Skip program name
        let args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

        let mut i = 0;
        while i < args.len() {
            let arg = args[i];

            if arg.starts_with("--rwget-") {
                // Parse rwget-specific flags
                if let Some((key, value)) = parse_rwget_flag(arg) {
                    match key {
                        "no-fallback" => config.no_fallback = true,
                        "quiet" => config.quiet = true,
                        "debug" => config.debug = true,
                        "no-body-detection" => config.body_detection = false,
                        "no-cache" => config.no_cache = true,
                        "js" => config.fallback_stage = 3,
                        "version" => command = Command::Version,
                        "help" => command = Command::Help,
                        "clear-cache" => command = Command::ClearCache,
                        "list-profiles" => command = Command::ListProfiles,
                        "update-profiles" => is_update_profiles = true,
                        "download-chromium" => command = Command::DownloadChromium,
                        "chromium-path" => command = Command::ChromiumPath,
                        "profile-url" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-profile-url requires a value"))?;
                            profile_url = Some(v.to_string());
                        }
                        "no-verify" => profile_no_verify = true,

                        "engine" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-engine requires a value"))?;
                            config.engine = Engine::from_str(v)?;
                        }

                        "fallback-codes" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-fallback-codes requires a value"))?;
                            config.fallback_codes = parse_codes(v)?;
                        }

                        "fallback-stage" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-fallback-stage requires a value"))?;
                            config.fallback_stage = v.parse().map_err(|_| anyhow!("Invalid stage: {}", v))?;
                            if config.fallback_stage < 1 || config.fallback_stage > 3 {
                                return Err(anyhow!("Stage must be 1, 2, or 3"));
                            }
                        }

                        "fallback-patterns" => {
                            let _v = value.ok_or_else(|| anyhow!("--rwget-fallback-patterns requires a value"))?;
                            // TODO: Store custom patterns
                        }

                        "profile" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-profile requires a value"))?;
                            config.profile = Some(v.to_string());
                        }

                        "daemon" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-daemon requires a value"))?;
                            config.daemon_mode = DaemonMode::from_str(v)
                                .ok_or_else(|| anyhow!("Invalid daemon mode: {}. Use auto, on, or off", v))?;
                        }

                        "timeout-stage1" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-timeout-stage1 requires a value"))?;
                            config.timeout_stage1 = Some(v.parse().map_err(|_| anyhow!("Invalid timeout: {}", v))?);
                        }

                        "timeout-stage2" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-timeout-stage2 requires a value"))?;
                            config.timeout_stage2 = v.parse().map_err(|_| anyhow!("Invalid timeout: {}", v))?;
                        }

                        "timeout-stage3" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-timeout-stage3 requires a value"))?;
                            config.timeout_stage3 = v.parse().map_err(|_| anyhow!("Invalid timeout: {}", v))?;
                        }

                        "js-wait" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-js-wait requires a value"))?;
                            config.js_wait = Some(v.to_string());
                        }

                        "verify-profile" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-verify-profile requires a value"))?;
                            command = Command::VerifyProfile(v.to_string());
                        }

                        "completions" => {
                            let v = value.ok_or_else(|| anyhow!("--rwget-completions requires a shell name (bash, zsh, fish, powershell)"))?;
                            command = Command::GenerateCompletions(v.to_string());
                        }

                        _ => {
                            return Err(anyhow!("Unknown rwget option: --rwget-{}", key));
                        }
                    }
                }
            } else {
                // Pass through to wget
                wget_args.push(arg.to_string());
            }

            i += 1;
        }

        // Build UpdateProfiles command if requested
        if is_update_profiles {
            command = Command::UpdateProfiles {
                url: profile_url,
                no_verify: profile_no_verify,
            };
        }

        Ok(Args {
            config,
            command,
            wget_args,
        })
    }
}

/// Parse an --rwget-* flag into (key, optional_value)
fn parse_rwget_flag(arg: &str) -> Option<(&str, Option<&str>)> {
    let without_prefix = arg.strip_prefix("--rwget-")?;

    if let Some((key, value)) = without_prefix.split_once('=') {
        Some((key, Some(value)))
    } else {
        Some((without_prefix, None))
    }
}

/// Parse comma-separated HTTP status codes
fn parse_codes(s: &str) -> Result<Vec<u16>> {
    s.split(',')
        .map(|code| {
            code.trim()
                .parse::<u16>()
                .map_err(|_| anyhow!("Invalid HTTP status code: {}", code))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "https://example.com".to_string(),
        ])
        .unwrap();

        assert!(!args.config.no_fallback);
        assert_eq!(args.wget_args, vec!["https://example.com"]);
    }

    #[test]
    fn test_parse_no_fallback() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "--rwget-no-fallback".to_string(),
            "https://example.com".to_string(),
        ])
        .unwrap();

        assert!(args.config.no_fallback);
        assert_eq!(args.wget_args, vec!["https://example.com"]);
    }

    #[test]
    fn test_parse_engine() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "--rwget-engine=wget2".to_string(),
            "https://example.com".to_string(),
        ])
        .unwrap();

        assert_eq!(args.config.engine, Engine::Wget2);
    }

    #[test]
    fn test_parse_mixed_flags() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "-O".to_string(),
            "output.txt".to_string(),
            "--rwget-quiet".to_string(),
            "--continue".to_string(),
            "https://example.com".to_string(),
        ])
        .unwrap();

        assert!(args.config.quiet);
        assert_eq!(
            args.wget_args,
            vec!["-O", "output.txt", "--continue", "https://example.com"]
        );
    }

    #[test]
    fn test_parse_version() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "--rwget-version".to_string(),
        ])
        .unwrap();

        assert!(matches!(args.command, Command::Version));
    }

    #[test]
    fn test_parse_fallback_codes() {
        let args = Args::parse(vec![
            "rwget".to_string(),
            "--rwget-fallback-codes=403,429".to_string(),
            "https://example.com".to_string(),
        ])
        .unwrap();

        assert_eq!(args.config.fallback_codes, vec![403, 429]);
    }
}
