//! rewget - wget-compatible wrapper with automatic fallback

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;
mod cli;
mod daemon;
mod exec;

use anyhow::Result;
use args::Args;
use cli::generate_completions;

fn main() -> Result<()> {
    let mut args = Args::parse(std::env::args().collect())?;

    // Load config file and merge with CLI args (CLI takes precedence)
    let config_file = rewget_core::ConfigFile::load();
    config_file.merge_into(&mut args.config);

    match args.command {
        args::Command::Version => {
            println!("rewget {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        args::Command::Help => {
            print_help();
            Ok(())
        }
        args::Command::ClearCache => {
            clear_cache()?;
            Ok(())
        }
        args::Command::ListProfiles => {
            list_profiles()?;
            Ok(())
        }
        args::Command::UpdateProfiles { url, no_verify } => {
            update_profiles(url.as_deref(), no_verify)?;
            Ok(())
        }
        args::Command::DownloadChromium => {
            download_chromium()?;
            Ok(())
        }
        args::Command::ChromiumPath => {
            print_chromium_path();
            Ok(())
        }
        args::Command::VerifyProfile(name) => {
            verify_profile(&name)?;
            Ok(())
        }
        args::Command::GenerateCompletions(shell) => {
            generate_completions(&shell)?;
            Ok(())
        }
        args::Command::Run => exec::run(args.config, args.wget_args),
    }
}

fn print_help() {
    println!(
        r#"rewget {} - wget-compatible wrapper with automatic fallback

USAGE:
    rewget [RWGET_OPTIONS] [WGET_OPTIONS] [URL...]

DESCRIPTION:
    rewget is a drop-in replacement for wget that automatically retries
    with browser emulation when sites block standard wget requests.

RWGET OPTIONS:
    --rewget-no-fallback       Disable fallback, behave exactly like wget
    --rewget-engine=<ENGINE>   Select wget engine (wget, wget2)
    --rewget-quiet             Suppress rewget status messages
    --rewget-debug             Enable verbose debug output

    --rewget-fallback-codes=<CODES>
                              Comma-separated HTTP codes that trigger fallback
                              Default: 403,429,503,520-529

    --rewget-fallback-stage=<N>
                              Start at stage N (1=wget, 2=impersonate, 3=js)

    --rewget-no-body-detection
                              Disable HTML body pattern detection

    --rewget-profile=<NAME>    Use specific browser profile for impersonation
    --rewget-daemon=<MODE>     Daemon mode: auto, on, off

    --rewget-timeout-stage1=<MS>   Stage 1 timeout (default: wget settings)
    --rewget-timeout-stage2=<MS>   Stage 2 timeout (default: 15000)
    --rewget-timeout-stage3=<MS>   Stage 3 timeout (default: 30000)

    --rewget-no-cache          Disable domain stage caching
    --rewget-clear-cache       Clear stage cache and exit

    --rewget-js                Force JS preflight (Stage 3)
    --rewget-js-wait=<COND>    Wait condition for JS preflight

    --rewget-update-profiles   Update browser fingerprint profiles from remote
    --rewget-profile-url=<URL> Custom profile update URL
    --rewget-no-verify         Skip Ed25519 signature verification
    --rewget-list-profiles     List available profiles
    --rewget-verify-profile=<NAME>
                              Verify profile fingerprints

    --rewget-download-chromium Pre-download Chromium for JS preflight
    --rewget-chromium-path     Print Chromium installation path

    --rewget-completions=<SHELL>
                              Generate shell completions (bash, zsh, fish, powershell)

    --rewget-version           Print rewget version
    --rewget-help              Print this help message

ENVIRONMENT:
    RWGET_ENGINE              Default engine (wget or wget2)

EXAMPLES:
    # Download with automatic fallback
    rewget https://example.com/file.tar.gz

    # Strict mode (no fallback)
    rewget --rewget-no-fallback https://example.com/file.tar.gz

    # Use wget2 engine
    rewget --rewget-engine=wget2 https://example.com/file.tar.gz

    # Force Stage 3 (JS preflight)
    rewget --rewget-js https://protected-site.com/

For wget options, run: wget --help
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn clear_cache() -> Result<()> {
    let mut cache = rewget_core::DomainCache::load();
    let count = cache.len();

    if count > 0 {
        cache.clear();
        cache.save()?;
        eprintln!("[rewget] Cleared {} cached domain entries", count);
    } else {
        eprintln!("[rewget] Cache is empty");
    }

    Ok(())
}

fn list_profiles() -> Result<()> {
    use rewget_core::ProfileCollection;

    // Initialize defaults if needed
    let _ = ProfileCollection::init_defaults();

    let collection = ProfileCollection::load();

    if collection.profiles.is_empty() {
        eprintln!("[rewget] No profiles found");
        return Ok(());
    }

    println!("Available browser profiles:");
    println!();

    for profile in &collection.profiles {
        println!("  {} - {}", profile.name, profile.description);
        println!(
            "    Browser: {} {}",
            profile.browser.name, profile.browser.version
        );
        println!("    Platform: {}", profile.browser.platform);
        println!(
            "    TLS: {} cipher suites, GREASE: {}",
            profile.tls.cipher_suites.len(),
            if profile.tls.grease { "yes" } else { "no" }
        );
        println!("    HTTP/2: {} settings", profile.http2.settings.len());
        println!();
    }

    println!("Total: {} profiles", collection.profiles.len());
    println!();
    println!("Usage: rewget --rewget-profile=<name> <url>");

    Ok(())
}

fn update_profiles(url: Option<&str>, no_verify: bool) -> Result<()> {
    use rewget_core::{ProfileCollection, DEFAULT_PROFILE_URL};

    let source_url = url.unwrap_or(DEFAULT_PROFILE_URL);
    eprintln!("[rewget] Fetching profiles from: {}", source_url);

    if no_verify {
        eprintln!("[rewget] Warning: Signature verification disabled");
    }

    match ProfileCollection::update_from_remote(url, !no_verify) {
        Ok(result) => {
            if !result.updated.is_empty() {
                eprintln!("[rewget] Updated: {}", result.updated.join(", "));
            }
            if !result.added.is_empty() {
                eprintln!("[rewget] Added: {}", result.added.join(", "));
            }
            if result.unchanged > 0 {
                eprintln!("[rewget] Unchanged: {} profiles", result.unchanged);
            }
            eprintln!("[rewget] Total: {} profiles", result.total);
            eprintln!(
                "[rewget] Profiles saved to: {}",
                ProfileCollection::builtin_path().display()
            );
            Ok(())
        }
        Err(e) => {
            // If remote fails, offer to reset to defaults
            eprintln!("[rewget] Remote update failed: {}", e);
            eprintln!("[rewget] Falling back to built-in defaults...");

            let collection = ProfileCollection::default_profiles();
            collection.save_builtin()?;

            eprintln!(
                "[rewget] Reset to {} built-in profiles",
                collection.profiles.len()
            );
            Ok(())
        }
    }
}

fn download_chromium() -> Result<()> {
    use rewget_core::{
        chromium_installed, chromium_path, download_chromium as do_download, CHROMIUM_VERSION,
    };

    if chromium_installed() {
        eprintln!(
            "[rewget] Chromium already installed at: {}",
            chromium_path().display()
        );
        return Ok(());
    }

    eprintln!(
        "[rewget] Downloading Chrome for Testing v{}...",
        CHROMIUM_VERSION
    );
    eprintln!("[rewget] This is approximately 150MB and only needs to be done once.");
    eprintln!();

    match do_download(|_downloaded, _total| {
        // Progress callback (wget/curl shows progress)
    }) {
        Ok(_) => {
            eprintln!();
            eprintln!(
                "[rewget] Chromium installed at: {}",
                chromium_path().display()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("[rewget] Download failed: {}", e);
            Err(anyhow::anyhow!("Chromium download failed: {}", e))
        }
    }
}

fn print_chromium_path() {
    use rewget_core::chromium_path;

    let status = rewget_core::ChromiumStatus::check();

    if status.installed {
        println!("{}", status.path.display());
        if let Some(version) = status.version {
            eprintln!("[rewget] Version: {}", version);
        }
    } else {
        println!("{}", chromium_path().display());
        eprintln!("[rewget] Chromium not installed. Run: rewget --rewget-download-chromium");
    }
}

fn verify_profile(name: &str) -> Result<()> {
    use rewget_core::ProfileCollection;

    let _ = ProfileCollection::init_defaults();
    let collection = ProfileCollection::load();

    match collection.get(name) {
        Some(profile) => {
            println!("Profile: {}", profile.name);
            println!("Description: {}", profile.description);
            println!("Version: {}", profile.version);
            if let Some(updated) = &profile.updated_at {
                println!("Updated: {}", updated);
            }
            println!();

            println!("Browser:");
            println!("  Name: {}", profile.browser.name);
            println!("  Version: {}", profile.browser.version);
            println!("  Platform: {}", profile.browser.platform);
            println!("  User-Agent: {}", profile.browser.user_agent);
            println!();

            println!("TLS Settings:");
            println!("  Versions: {}", profile.tls.versions.join(", "));
            println!("  Cipher Suites: {} total", profile.tls.cipher_suites.len());
            for (i, suite) in profile.tls.cipher_suites.iter().enumerate().take(5) {
                println!("    {}. {}", i + 1, suite);
            }
            if profile.tls.cipher_suites.len() > 5 {
                println!("    ... and {} more", profile.tls.cipher_suites.len() - 5);
            }
            println!("  Extensions: {} total", profile.tls.extensions.len());
            println!("  Curves: {}", profile.tls.curves.join(", "));
            println!("  ALPN: {}", profile.tls.alpn.join(", "));
            println!(
                "  GREASE: {}",
                if profile.tls.grease {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!();

            println!("HTTP/2 Settings:");
            for (key, value) in &profile.http2.settings {
                println!("  {}: {}", key, value);
            }
            println!("  Window Update: {}", profile.http2.window_update);
            println!(
                "  Pseudo-header Order: {}",
                profile.http2.pseudo_header_order.join(", ")
            );
            println!();

            println!("Default Headers: {}", profile.headers.len());
            for (key, value) in &profile.headers {
                let display_value = if value.len() > 60 {
                    format!("{}...", &value[..57])
                } else {
                    value.clone()
                };
                println!("  {}: {}", key, display_value);
            }

            println!();
            println!("[rewget] Profile '{}' is valid", name);
            Ok(())
        }
        None => {
            eprintln!("[rewget] Profile '{}' not found", name);
            eprintln!();
            eprintln!("Available profiles:");
            for pname in collection.list_names() {
                eprintln!("  {}", pname);
            }
            Err(anyhow::anyhow!("Profile not found: {}", name))
        }
    }
}
