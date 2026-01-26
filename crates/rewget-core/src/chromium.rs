//! Chromium management for Stage 3 (JS preflight)
//!
//! Handles lazy downloading and management of the Chromium binary.

use crate::Result;
use std::fs;
use std::path::PathBuf;

/// Chromium version to download
pub const CHROMIUM_VERSION: &str = "131.0.6778.204";

/// Get the Chromium installation directory
pub fn chromium_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("rewget")
        .join("chromium")
}

/// Get the path to the Chromium executable
pub fn chromium_path() -> PathBuf {
    let dir = chromium_dir();

    #[cfg(target_os = "linux")]
    {
        dir.join("chrome-linux64").join("chrome")
    }

    #[cfg(target_os = "macos")]
    {
        dir.join("chrome-mac-x64")
            .join("Google Chrome for Testing.app")
            .join("Contents")
            .join("MacOS")
            .join("Google Chrome for Testing")
    }

    #[cfg(target_os = "windows")]
    {
        dir.join("chrome-win64").join("chrome.exe")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        dir.join("chrome")
    }
}

/// Check if Chromium is installed
pub fn is_installed() -> bool {
    let path = chromium_path();
    path.exists() && path.is_file()
}

/// Get the download URL for Chrome for Testing
pub fn download_url() -> String {
    let platform = get_platform();
    format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{}/{}/chrome-{}.zip",
        CHROMIUM_VERSION, platform, platform
    )
}

/// Get platform string for download URL
fn get_platform() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux64"
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "mac-x64"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "mac-arm64"
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win64"
    }

    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    {
        "win32"
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
    )))]
    {
        "linux64" // Fallback
    }
}

/// Download and install Chromium
///
/// This downloads Chrome for Testing from Google's official repository.
/// The download is approximately 150MB.
pub fn download_chromium<F>(progress_callback: F) -> Result<()>
where
    F: Fn(u64, u64), // (bytes_downloaded, total_bytes)
{
    let url = download_url();
    let dest_dir = chromium_dir();

    // Ensure destination directory exists
    fs::create_dir_all(&dest_dir)?;

    // Download the zip file
    let zip_path = dest_dir.join("chrome.zip");

    // Use a simple blocking HTTP client for download
    // In production, this would use reqwest or similar
    ureq_download(&url, &zip_path, progress_callback)?;

    // Extract the zip file
    extract_zip(&zip_path, &dest_dir)?;

    // Clean up zip file
    let _ = fs::remove_file(&zip_path);

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let chrome_path = chromium_path();
        if chrome_path.exists() {
            let mut perms = fs::metadata(&chrome_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&chrome_path, perms)?;
        }
    }

    Ok(())
}

/// Download a file using a simple HTTP GET
#[cfg(unix)]
fn ureq_download<F>(url: &str, dest: &PathBuf, _progress: F) -> Result<()>
where
    F: Fn(u64, u64),
{
    // Try wget first, then curl as fallback
    let status = std::process::Command::new("wget")
        .args(["-q", "--show-progress", "-O"])
        .arg(dest)
        .arg(url)
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // Try curl as fallback
            let status = std::process::Command::new("curl")
                .args(["-L", "-#", "-o"])
                .arg(dest)
                .arg(url)
                .status()?;

            if status.success() {
                Ok(())
            } else {
                Err(crate::Error::Config("Failed to download Chromium".to_string()))
            }
        }
    }
}

/// Download a file using PowerShell (Windows)
#[cfg(windows)]
fn ureq_download<F>(url: &str, dest: &PathBuf, _progress: F) -> Result<()>
where
    F: Fn(u64, u64),
{
    // Use PowerShell's Invoke-WebRequest
    let ps_script = format!(
        "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        url,
        dest.display()
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        // Try curl as fallback (curl is available on recent Windows)
        let status = std::process::Command::new("curl")
            .args(["-L", "-o"])
            .arg(dest)
            .arg(url)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(crate::Error::Config("Failed to download Chromium".to_string()))
        }
    }
}

/// Extract a zip file
#[cfg(unix)]
fn extract_zip(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
    // Use system unzip command
    let status = std::process::Command::new("unzip")
        .args(["-q", "-o"])
        .arg(zip_path)
        .arg("-d")
        .arg(dest_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(crate::Error::Config("Failed to extract Chromium".to_string()))
    }
}

/// Extract a zip file (Windows version using PowerShell)
#[cfg(windows)]
fn extract_zip(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
    let ps_script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        zip_path.display(),
        dest_dir.display()
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(crate::Error::Config("Failed to extract Chromium".to_string()))
    }
}

/// Get Chromium status information
#[derive(Debug, Clone)]
pub struct ChromiumStatus {
    pub installed: bool,
    pub path: PathBuf,
    pub version: Option<String>,
}

impl ChromiumStatus {
    pub fn check() -> Self {
        let path = chromium_path();
        let installed = is_installed();

        let version = if installed {
            // Try to get version from chrome --version
            std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        } else {
            None
        };

        Self {
            installed,
            path,
            version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromium_dir() {
        let dir = chromium_dir();
        assert!(dir.to_string_lossy().contains("rewget"));
        assert!(dir.to_string_lossy().contains("chromium"));
    }

    #[test]
    fn test_chromium_path() {
        let path = chromium_path();
        assert!(path.to_string_lossy().contains("chrome"));
    }

    #[test]
    fn test_download_url() {
        let url = download_url();
        assert!(url.contains("chrome-for-testing"));
        assert!(url.contains(CHROMIUM_VERSION));
    }

    #[test]
    fn test_chromium_status() {
        let status = ChromiumStatus::check();
        // Just check it doesn't panic
        let _ = status.installed;
        let _ = status.path;
    }
}
