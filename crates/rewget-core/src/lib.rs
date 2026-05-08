//! rewget-core: Shared library for rewget components

pub mod cache;
pub mod chromium;
pub mod config;
pub mod detection;
pub mod engine;
pub mod error;
pub mod ipc;
pub mod orchestrator;
pub mod profile;
pub mod stage;

pub use cache::{extract_domain, DomainCache};
pub use chromium::{
    chromium_dir, chromium_path, download_chromium, is_installed as chromium_installed,
    ChromiumStatus, CHROMIUM_VERSION,
};
pub use config::{Config, ConfigFile, DaemonMode};
pub use detection::{
    analyze_body, analyze_exit_code, is_fallback_code, BlockReason, DetectionResult, BLOCK_PATTERNS,
};
pub use engine::Engine;
pub use error::{Error, Result};
pub use ipc::{socket_path, DaemonStatus, Request, Response};
pub use orchestrator::{FetchAction, FetchOrchestrator, FetchState, StageOutput};
pub use profile::{
    BrowserInfo, Http2Settings, Profile, ProfileCollection, ProfileUpdateResult, TlsSettings,
    DEFAULT_PROFILE_URL,
};
pub use stage::FetchStage;
