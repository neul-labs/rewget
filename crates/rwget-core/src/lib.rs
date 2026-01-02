//! rwget-core: Shared library for rwget components

pub mod cache;
pub mod chromium;
pub mod config;
pub mod detection;
pub mod engine;
pub mod error;
pub mod ipc;
pub mod profile;

pub use cache::{extract_domain, DomainCache};
pub use chromium::{chromium_dir, chromium_path, is_installed as chromium_installed, download_chromium, ChromiumStatus, CHROMIUM_VERSION};
pub use config::{Config, DaemonMode};
pub use detection::{analyze_body, analyze_exit_code, is_fallback_code, DetectionResult, BlockReason, BLOCK_PATTERNS};
pub use engine::Engine;
pub use error::{Error, Result};
pub use ipc::{socket_path, Request, Response, DaemonStatus};
pub use profile::{Profile, ProfileCollection, BrowserInfo, TlsSettings, Http2Settings, ProfileUpdateResult, DEFAULT_PROFILE_URL};
