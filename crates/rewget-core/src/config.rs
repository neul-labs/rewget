//! Configuration for rewget

use crate::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Runtime configuration for rewget
#[derive(Debug, Clone)]
pub struct Config {
    /// Selected wget engine
    pub engine: Engine,

    /// Disable automatic fallback (strict mode)
    pub no_fallback: bool,

    /// HTTP status codes that trigger fallback
    pub fallback_codes: Vec<u16>,

    /// Enable body pattern detection
    pub body_detection: bool,

    /// Suppress rewget messages
    pub quiet: bool,

    /// Enable debug output
    pub debug: bool,

    /// Stage to start at (1, 2, or 3)
    pub fallback_stage: u8,

    /// Disable domain stage caching
    pub no_cache: bool,

    /// Browser profile for impersonation
    pub profile: Option<String>,

    /// Daemon mode (auto, on, off)
    pub daemon_mode: DaemonMode,

    /// Stage timeouts in milliseconds
    pub timeout_stage1: Option<u64>,
    pub timeout_stage2: u64,
    pub timeout_stage3: u64,

    /// JS wait condition for Stage 3
    pub js_wait: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
            no_fallback: false,
            fallback_codes: vec![403, 429, 503, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529],
            body_detection: true,
            quiet: false,
            debug: false,
            fallback_stage: 1,
            no_cache: false,
            profile: None,
            daemon_mode: DaemonMode::Auto,
            timeout_stage1: None, // Inherits wget timeout
            timeout_stage2: 15000,
            timeout_stage3: 30000,
            js_wait: None,
        }
    }
}

/// Daemon operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DaemonMode {
    #[default]
    Auto,
    On,
    Off,
}

impl DaemonMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(DaemonMode::Auto),
            "on" => Some(DaemonMode::On),
            "off" => Some(DaemonMode::Off),
            _ => None,
        }
    }
}

/// Persistent configuration file format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub fallback: FallbackConfig,

    #[serde(default)]
    pub daemon: DaemonConfig,

    #[serde(default)]
    pub profiles: ProfileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_fallback_codes")]
    pub codes: Vec<u16>,

    #[serde(default = "default_true")]
    pub body_detection: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            codes: default_fallback_codes(),
            body_detection: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    #[serde(default = "default_browser_pool_size")]
    pub browser_pool_size: u8,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            idle_timeout: 300,
            browser_pool_size: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default = "default_profile")]
    pub default: String,

    #[serde(default)]
    pub auto_update: bool,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            default: "chrome".to_string(),
            auto_update: false,
        }
    }
}

fn default_true() -> bool { true }
fn default_fallback_codes() -> Vec<u16> { vec![403, 429, 503, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529] }
fn default_idle_timeout() -> u64 { 300 }
fn default_browser_pool_size() -> u8 { 2 }
fn default_profile() -> String { "chrome".to_string() }

impl ConfigFile {
    /// Get config directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("rewget"))
    }

    /// Get config file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }
}
