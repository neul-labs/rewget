//! CLI definition for shell completion generation

use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, Shell};
use std::io;

/// Build the clap Command for rwget
///
/// This is used for shell completion generation. The actual argument parsing
/// is done in args.rs for more control over wget passthrough.
pub fn build_cli() -> Command {
    Command::new("rwget")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_version_flag(true)
        .disable_help_flag(true)
        .about("wget-compatible wrapper with automatic fallback")
        .long_about(
            "rwget is a drop-in replacement for wget that automatically retries \
             with browser emulation when sites block standard wget requests."
        )
        .arg(
            Arg::new("no-fallback")
                .long("rwget-no-fallback")
                .help("Disable fallback, behave exactly like wget")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("engine")
                .long("rwget-engine")
                .value_name("ENGINE")
                .help("Select wget engine (wget, wget2)")
        )
        .arg(
            Arg::new("quiet")
                .long("rwget-quiet")
                .help("Suppress rwget status messages")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("debug")
                .long("rwget-debug")
                .help("Enable verbose debug output")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("fallback-codes")
                .long("rwget-fallback-codes")
                .value_name("CODES")
                .help("Comma-separated HTTP codes that trigger fallback")
        )
        .arg(
            Arg::new("fallback-stage")
                .long("rwget-fallback-stage")
                .value_name("N")
                .help("Start at stage N (1=wget, 2=impersonate, 3=js)")
        )
        .arg(
            Arg::new("no-body-detection")
                .long("rwget-no-body-detection")
                .help("Disable HTML body pattern detection")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("profile")
                .long("rwget-profile")
                .value_name("NAME")
                .help("Use specific browser profile for impersonation")
        )
        .arg(
            Arg::new("daemon")
                .long("rwget-daemon")
                .value_name("MODE")
                .help("Daemon mode: auto, on, off")
        )
        .arg(
            Arg::new("timeout-stage1")
                .long("rwget-timeout-stage1")
                .value_name("MS")
                .help("Stage 1 timeout in milliseconds")
        )
        .arg(
            Arg::new("timeout-stage2")
                .long("rwget-timeout-stage2")
                .value_name("MS")
                .help("Stage 2 timeout (default: 15000)")
        )
        .arg(
            Arg::new("timeout-stage3")
                .long("rwget-timeout-stage3")
                .value_name("MS")
                .help("Stage 3 timeout (default: 30000)")
        )
        .arg(
            Arg::new("no-cache")
                .long("rwget-no-cache")
                .help("Disable domain stage caching")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("clear-cache")
                .long("rwget-clear-cache")
                .help("Clear stage cache and exit")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("js")
                .long("rwget-js")
                .help("Force JS preflight (Stage 3)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("js-wait")
                .long("rwget-js-wait")
                .value_name("COND")
                .help("Wait condition for JS preflight")
        )
        .arg(
            Arg::new("update-profiles")
                .long("rwget-update-profiles")
                .help("Update browser fingerprint profiles")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("profile-url")
                .long("rwget-profile-url")
                .value_name("URL")
                .help("Custom profile update URL")
        )
        .arg(
            Arg::new("no-verify")
                .long("rwget-no-verify")
                .help("Skip Ed25519 signature verification")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("list-profiles")
                .long("rwget-list-profiles")
                .help("List available profiles")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verify-profile")
                .long("rwget-verify-profile")
                .value_name("NAME")
                .help("Verify profile fingerprints")
        )
        .arg(
            Arg::new("download-chromium")
                .long("rwget-download-chromium")
                .help("Pre-download Chromium for JS preflight")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("chromium-path")
                .long("rwget-chromium-path")
                .help("Print Chromium installation path")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("completions")
                .long("rwget-completions")
                .value_name("SHELL")
                .help("Generate shell completions")
        )
        .arg(
            Arg::new("version")
                .long("rwget-version")
                .help("Print rwget version")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("help")
                .long("rwget-help")
                .help("Print help message")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("url")
                .help("URL(s) to download")
                .action(ArgAction::Append)
        )
        .trailing_var_arg(true)
        .allow_external_subcommands(true)
}

/// Generate shell completions and print to stdout
pub fn generate_completions(shell_name: &str) -> anyhow::Result<()> {
    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        _ => {
            anyhow::bail!(
                "Unknown shell '{}'. Supported: bash, zsh, fish, powershell",
                shell_name
            );
        }
    };

    let mut cmd = build_cli();
    generate(shell, &mut cmd, "rwget", &mut io::stdout());
    Ok(())
}
