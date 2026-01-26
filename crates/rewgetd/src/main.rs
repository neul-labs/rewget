//! rewgetd - Daemon for rewget Stage 2/3 execution
//!
//! This daemon handles:
//! - Stage 2: Impersonation requests with browser TLS/HTTP2 fingerprints
//! - Stage 3: JS preflight with headless Chromium

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod server;
mod impersonate;
mod preflight;

use anyhow::Result;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("rewgetd {} starting", env!("CARGO_PKG_VERSION"));

    // Create persistent tokio runtime (shared across all requests)
    let runtime = Arc::new(Runtime::new()?);
    info!("Tokio runtime initialized");

    // Run the server with shared runtime
    server::run(runtime)
}
