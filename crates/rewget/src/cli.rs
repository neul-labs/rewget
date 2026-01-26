//! CLI definition for shell completion generation

use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, Shell};
use std::io;

/// Build the clap Command for rewget
///
/// This is used for shell completion generation. The actual argument parsing
/// is done in args.rs for more control over wget passthrough.
pub fn build_cli() -> Command {
    Command::new("rewget")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_version_flag(true)
        .disable_help_flag(true)
        .about("wget-compatible wrapper with automatic fallback")
        .long_about(
            "rewget is a drop-in replacement for wget that automatically retries \
             with browser emulation when sites block standard wget requests."
        )
        .arg(
            Arg::new("no-fallback")
                .long("rewget-no-fallback")
                .help("Disable fallback, behave exactly like wget")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("engine")
                .long("rewget-engine")
                .value_name("ENGINE")
                .help("Select wget engine (wget, wget2)")
        )
        .arg(
            Arg::new("quiet")
                .long("rewget-quiet")
                .help("Suppress rewget status messages")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("debug")
                .long("rewget-debug")
                .help("Enable verbose debug output")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("fallback-codes")
                .long("rewget-fallback-codes")
                .value_name("CODES")
                .help("Comma-separated HTTP codes that trigger fallback")
        )
        .arg(
            Arg::new("fallback-stage")
                .long("rewget-fallback-stage")
                .value_name("N")
                .help("Start at stage N (1=wget, 2=impersonate, 3=js)")
        )
        .arg(
            Arg::new("no-body-detection")
                .long("rewget-no-body-detection")
                .help("Disable HTML body pattern detection")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("profile")
                .long("rewget-profile")
                .value_name("NAME")
                .help("Use specific browser profile for impersonation")
        )
        .arg(
            Arg::new("daemon")
                .long("rewget-daemon")
                .value_name("MODE")
                .help("Daemon mode: auto, on, off")
        )
        .arg(
            Arg::new("timeout-stage1")
                .long("rewget-timeout-stage1")
                .value_name("MS")
                .help("Stage 1 timeout in milliseconds")
        )
        .arg(
            Arg::new("timeout-stage2")
                .long("rewget-timeout-stage2")
                .value_name("MS")
                .help("Stage 2 timeout (default: 15000)")
        )
        .arg(
            Arg::new("timeout-stage3")
                .long("rewget-timeout-stage3")
                .value_name("MS")
                .help("Stage 3 timeout (default: 30000)")
        )
        .arg(
            Arg::new("no-cache")
                .long("rewget-no-cache")
                .help("Disable domain stage caching")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("clear-cache")
                .long("rewget-clear-cache")
                .help("Clear stage cache and exit")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("js")
                .long("rewget-js")
                .help("Force JS preflight (Stage 3)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("js-wait")
                .long("rewget-js-wait")
                .value_name("COND")
                .help("Wait condition for JS preflight")
        )
        .arg(
            Arg::new("update-profiles")
                .long("rewget-update-profiles")
                .help("Update browser fingerprint profiles")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("profile-url")
                .long("rewget-profile-url")
                .value_name("URL")
                .help("Custom profile update URL")
        )
        .arg(
            Arg::new("no-verify")
                .long("rewget-no-verify")
                .help("Skip Ed25519 signature verification")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("list-profiles")
                .long("rewget-list-profiles")
                .help("List available profiles")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verify-profile")
                .long("rewget-verify-profile")
                .value_name("NAME")
                .help("Verify profile fingerprints")
        )
        .arg(
            Arg::new("download-chromium")
                .long("rewget-download-chromium")
                .help("Pre-download Chromium for JS preflight")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("chromium-path")
                .long("rewget-chromium-path")
                .help("Print Chromium installation path")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("completions")
                .long("rewget-completions")
                .value_name("SHELL")
                .help("Generate shell completions")
        )
        .arg(
            Arg::new("version")
                .long("rewget-version")
                .help("Print rewget version")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("help")
                .long("rewget-help")
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
    generate(shell, &mut cmd, "rewget", &mut io::stdout());
    Ok(())
}
