//! Configuration for rewget

use crate::{Engine, FetchStage};
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

    /// Stage to start at (Wget, Impersonate, or Preflight)
    pub fallback_stage: FetchStage,

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
            fallback_codes: vec![
                403, 429, 503, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529,
            ],
            body_detection: true,
            quiet: false,
            debug: false,
            fallback_stage: FetchStage::Preflight,
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

impl std::str::FromStr for DaemonMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(DaemonMode::Auto),
            "on" => Ok(DaemonMode::On),
            "off" => Ok(DaemonMode::Off),
            _ => Err(format!(
                "Unknown daemon mode '{}'. Valid options: auto, on, off",
                s
            )),
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

fn default_true() -> bool {
    true
}
fn default_fallback_codes() -> Vec<u16> {
    vec![
        403, 429, 503, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529,
    ]
}
fn default_idle_timeout() -> u64 {
    300
}
fn default_browser_pool_size() -> u8 {
    2
}
fn default_profile() -> String {
    "chrome".to_string()
}

impl ConfigFile {
    /// Get config directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("rewget"))
    }

    /// Get config file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    /// Load config from disk, or return default if not found.
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) if path.exists() => match std::fs::read_to_string(&path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            },
            _ => Self::default(),
        }
    }

    /// Merge file config into a runtime `Config`, with file values acting as defaults.
    pub fn merge_into(&self, config: &mut Config) {
        if !self.fallback.enabled {
            config.no_fallback = true;
        }
        if !config.fallback_codes.is_empty() && !self.fallback.codes.is_empty() {
            config.fallback_codes = self.fallback.codes.clone();
        }
        if !self.fallback.body_detection {
            config.body_detection = false;
        }
        if config.profile.is_none() && !self.profiles.default.is_empty() {
            config.profile = Some(self.profiles.default.clone());
        }
    }
}
