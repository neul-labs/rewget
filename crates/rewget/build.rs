//! Build script for rewget
//!
//! Generates shell completions and man pages at compile time.

use clap::{Arg, ArgAction, Command};
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::env;
use std::fs;
use std::path::PathBuf;

fn build_cli() -> Command {
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
                .help("Comma-separated HTTP codes that trigger fallback (default: 403,429,503,520-529)")
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
                .help("Wait condition for JS preflight (delay:MS, selector:CSS, networkidle)")
        )
        .arg(
            Arg::new("update-profiles")
                .long("rewget-update-profiles")
                .help("Update browser fingerprint profiles from remote")
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
            Arg::new("version")
                .long("rewget-version")
                .help("Print rewget version")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("help")
                .long("rewget-help")
                .help("Print this help message")
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

fn main() {
    // Only generate completions when building release or when explicitly requested
    let out_dir = match env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return,
    };

    let cmd = build_cli();

    // Generate shell completions
    let completions_dir = out_dir.join("completions");
    fs::create_dir_all(&completions_dir).unwrap();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let _ = generate_to(shell, &mut cmd.clone(), "rewget", &completions_dir);
    }

    // Generate man page
    let man_dir = out_dir.join("man");
    fs::create_dir_all(&man_dir).unwrap();

    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).unwrap();
    fs::write(man_dir.join("rewget.1"), buffer).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
