# Architecture

Technical overview of rwget's internal architecture.

## Project Structure

```
rwget/
├── crates/
│   ├── rwget/           # CLI binary
│   │   ├── src/
│   │   │   ├── main.rs      # Entry point
│   │   │   ├── args.rs      # Argument parsing
│   │   │   ├── cli.rs       # Clap definitions
│   │   │   ├── exec.rs      # wget execution
│   │   │   └── daemon.rs    # Daemon communication
│   │   └── build.rs     # Shell completions, man page
│   │
│   ├── rwgetd/          # Daemon binary
│   │   └── src/
│   │       ├── main.rs      # Daemon entry point
│   │       ├── server.rs    # IPC server
│   │       ├── impersonate.rs  # Stage 2 logic
│   │       └── preflight.rs    # Stage 3 logic
│   │
│   └── rwget-core/      # Shared library
│       └── src/
│           ├── lib.rs       # Public API
│           ├── config.rs    # Configuration types
│           ├── profile.rs   # Browser profiles
│           ├── detection.rs # Bot detection patterns
│           ├── cache.rs     # Domain stage cache
│           ├── ipc.rs       # IPC protocol
│           └── chromium.rs  # Chromium management
│
├── documentation/       # MkDocs documentation
├── Formula/            # Homebrew formula
└── scripts/            # Build and install scripts
```

## Component Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                           User                                    │
└───────────────────────────────┬──────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                          rwget CLI                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  args.rs    │  │  exec.rs    │  │  daemon.rs  │              │
│  │  (parsing)  │  │  (Stage 1)  │  │  (IPC)      │              │
│  └─────────────┘  └─────────────┘  └──────┬──────┘              │
└───────────────────────────────────────────┼──────────────────────┘
                                            │ nng IPC
                                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                         rwgetd Daemon                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ server.rs   │  │impersonate  │  │ preflight   │              │
│  │ (IPC recv)  │  │  (Stage 2)  │  │  (Stage 3)  │              │
│  └─────────────┘  └──────┬──────┘  └──────┬──────┘              │
└──────────────────────────┼────────────────┼──────────────────────┘
                           │                │
                           ▼                ▼
               ┌───────────────────┐  ┌───────────────────┐
               │      rquest       │  │   chromiumoxide   │
               │  (TLS emulation)  │  │  (headless CDP)   │
               └───────────────────┘  └───────────────────┘
```

## Data Flow

### Stage 1: Direct wget

```
rwget args
    │
    ├─ Parse --rwget-* flags
    │
    ├─ Check domain cache
    │      │
    │      └─ If cached Stage 2/3, skip to that stage
    │
    ├─ Spawn wget subprocess
    │      │
    │      ├─ Stream stdout/stderr
    │      │
    │      └─ Wait for exit
    │
    └─ Check result
           │
           ├─ Success (exit 0) → Done
           │
           └─ Failure (403, etc.) → Try Stage 2
```

### Stage 2: Impersonation

```
rwget
    │
    ├─ Connect to rwgetd (spawn if needed)
    │
    ├─ Send Stage 2 request
    │      {url, profile, timeout, headers}
    │
    └─ Receive response
           │
           ├─ Success → Write body, Done
           │
           └─ Failure → Try Stage 3
```

### Stage 3: JavaScript Preflight

```
rwgetd
    │
    ├─ Launch Chromium (download if needed)
    │
    ├─ Navigate to URL
    │
    ├─ Wait for condition
    │      │
    │      ├─ networkidle
    │      ├─ selector match
    │      └─ delay timeout
    │
    ├─ Extract cookies + body
    │
    └─ Return response
```

## IPC Protocol

rwget and rwgetd communicate via nng (nanomsg-next-gen) using JSON messages.

### Request Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "stage2",
  "url": "https://example.com/path",
  "profile": "chrome_131",
  "timeout": 15000,
  "headers": {
    "Accept": "text/html"
  },
  "output_path": "/tmp/rwget-output.tmp",
  "js_wait": null
}
```

### Response Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true,
  "status": 200,
  "headers": {
    "Content-Type": "text/html",
    "Content-Length": "1234"
  },
  "body_path": "/tmp/rwget-output.tmp",
  "cookies": [
    {"name": "cf_clearance", "value": "...", "domain": ".example.com"}
  ],
  "error": null
}
```

### Commands

| Command | Description |
|---------|-------------|
| `stage2` | TLS impersonation request |
| `stage3` | JavaScript preflight request |
| `status` | Health check |

## Browser Profile Structure

```json
{
  "name": "chrome_131",
  "version": 1,
  "signature": "base64-ed25519-signature",

  "browser": {
    "name": "Chrome",
    "version": "131.0.0.0",
    "platform": "Windows",
    "user_agent": "Mozilla/5.0..."
  },

  "tls": {
    "versions": ["TLSv1.2", "TLSv1.3"],
    "cipher_suites": ["TLS_AES_128_GCM_SHA256", ...],
    "extensions": [0, 23, 65281, ...],
    "curves": ["x25519", "secp256r1"],
    "alpn": ["h2", "http/1.1"],
    "grease": true
  },

  "http2": {
    "settings": {
      "HEADER_TABLE_SIZE": 65536,
      "MAX_CONCURRENT_STREAMS": 1000,
      ...
    },
    "window_update": 15663105,
    "pseudo_header_order": [":method", ":authority", ":scheme", ":path"]
  },

  "headers": {
    "Accept": "text/html,...",
    "Accept-Language": "en-US,en;q=0.9"
  }
}
```

## Domain Cache Structure

```json
{
  "example.com": {
    "stage": 2,
    "profile": "chrome_131",
    "timestamp": 1704067200,
    "expires": 1704672000
  }
}
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `rquest` | HTTP client with TLS impersonation |
| `chromiumoxide` | Chrome DevTools Protocol client |
| `nng` | IPC transport |
| `clap` | CLI argument parsing |
| `serde` | JSON serialization |
| `ed25519-dalek` | Profile signature verification |
| `tokio` | Async runtime (daemon) |

## Platform-Specific Code

### Unix vs Windows

```rust
// exec.rs - Process execution
#[cfg(unix)]
fn exec_wget(...) {
    // Use exec() syscall - replaces process
    cmd.exec()
}

#[cfg(windows)]
fn exec_wget(...) {
    // Use spawn + wait - no exec() on Windows
    cmd.status()
}
```

```rust
// ipc.rs - Socket paths
#[cfg(unix)]
pub fn socket_path() -> PathBuf {
    // ~/.cache/rwget/rwgetd.sock
    dirs::runtime_dir().join("rwget/rwgetd.sock")
}

#[cfg(windows)]
pub fn socket_path() -> PathBuf {
    // Named pipe
    PathBuf::from(r"\\.\pipe\rwget")
}
```

### Chromium Download

```rust
#[cfg(unix)]
fn download_chromium() {
    // Use wget or curl
}

#[cfg(windows)]
fn download_chromium() {
    // Use PowerShell Invoke-WebRequest
}
```

## Build System

### Workspace Structure

```toml
# Cargo.toml (root)
[workspace]
members = ["crates/*"]

[workspace.package]
version = "1.0.0"

[workspace.dependencies]
# Shared dependencies
```

### Build Script

`crates/rwget/build.rs` generates:

- Shell completions (bash, zsh, fish, PowerShell)
- Man page (rwget.1)

### Release Profile

```toml
[profile.release]
lto = "thin"
strip = true
codegen-units = 1
opt-level = 3
panic = "abort"
```

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p rwget-core
cargo test -p rwgetd

# Run with output
cargo test -- --nocapture
```

## Extending rwget

### Adding a New Profile

1. Create JSON file in `~/.local/share/rwget/profiles/`
2. Follow the profile structure above
3. Test with `--rwget-verify-profile=name`

### Adding a New Detection Pattern

Edit `crates/rwget-core/src/detection.rs`:

```rust
pub const BLOCK_PATTERNS: &[&str] = &[
    "cloudflare",
    "just a moment",
    "your-new-pattern",
];
```

### Adding a New Command

1. Add variant to `Command` enum in `args.rs`
2. Add parsing logic in `Args::parse()`
3. Add handler in `main.rs`
