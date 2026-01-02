//! rwgetd - Daemon for rwget Stage 2/3 execution
//!
//! This daemon handles:
//! - Stage 2: Impersonation requests with browser TLS/HTTP2 fingerprints
//! - Stage 3: JS preflight with headless Chromium

mod server;
mod impersonate;
mod preflight;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("rwgetd {} starting", env!("CARGO_PKG_VERSION"));

    // Run the server
    server::run()
}
