# rewget

**wget, but it works everywhere.**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rewget.svg)](https://crates.io/crates/rewget)
[![npm](https://img.shields.io/npm/v/rewget.svg)](https://www.npmjs.com/package/rewget)
[![PyPI](https://img.shields.io/pypi/v/rewget.svg)](https://pypi.org/project/rewget/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/neul-labs/rewget/ci.yml?branch=main)](https://github.com/neul-labs/rewget/actions)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)](https://github.com/neul-labs/rewget/releases)
[![Docs](https://img.shields.io/badge/docs-docs.neullabs.com-blue)](https://docs.neullabs.com/rewget)

A drop-in wget replacement that automatically bypasses bot protection. When sites block wget with 403s or CAPTCHAs, rewget seamlessly retries with browser-like TLS fingerprints and JavaScript rendering.

---

## Quick Start

```bash
# Install via Homebrew (macOS / Linux)
brew install neul-labs/tap/rewget

# Or use the install script
curl -fsSL https://rewget.dev/install.sh | sh

# Windows (PowerShell)
irm https://rewget.dev/install.ps1 | iex
```

Use it exactly like wget:

```bash
rewget https://example.com/file.tar.gz
```

That's it. If the site blocks wget, rewget automatically retries with browser emulation.

## Why rewget?

| Problem | Solution |
|---------|----------|
| Site returns 403 Forbidden | Retries with Chrome/Firefox TLS fingerprint |
| CAPTCHA or challenge page | Runs headless browser to solve it |
| Rate limited (429) | Progressive fallback with session reuse |
| Works in browser but not wget | rewget makes it work |

## Usage

**Basic download** (automatic fallback on block):
```bash
rewget https://example.com/file.tar.gz
```

**Scripting mode** (fail fast, no retries):
```bash
rewget --rewget-no-fallback https://example.com/file.tar.gz
```

**Force JavaScript preflight** (for sites that always need a browser):
```bash
rewget --rewget-js https://example.com/
```

**Choose specific browser profile**:
```bash
rewget --rewget-profile=firefox136 https://example.com/
```

**List available profiles**:
```bash
rewget --rewget-list-profiles
# Chrome 131/130, Firefox 136/133, Safari 18, Edge 131
```

All standard wget options work unchanged. Add `--rewget-*` flags for enhanced behavior.

## How It Works

rewget uses a 3-stage fallback strategy:

```
Stage 1: wget          Fast, zero overhead
    ↓ (blocked?)
Stage 2: Impersonate   Browser TLS/HTTP2 fingerprint
    ↓ (still blocked?)
Stage 3: JS Preflight  Real headless browser
```

- **Stage 1**: Runs plain wget. If it succeeds, you get the exact same output.
- **Stage 2**: Retries with `rquest` using Chrome/Firefox TLS fingerprints.
- **Stage 3**: Launches headless Chromium to handle JavaScript challenges.

Results are cached per-domain (7-day TTL), so subsequent requests skip straight to the working stage.

## Installation

### Package Managers

```bash
# Homebrew (macOS / Linux)
brew install neul-labs/tap/rewget

# npm (all platforms)
npm install -g rewget

# PyPI (all platforms)
pip install rewget

# Cargo (from source)
cargo install rewget
```

### Direct Download

```bash
# Linux/macOS
curl -fsSL https://rewget.dev/install.sh | sh

# Windows PowerShell
irm https://rewget.dev/install.ps1 | iex
```

### Shell Completions

```bash
# Bash
eval "$(rewget --rewget-completions=bash)"

# Zsh
eval "$(rewget --rewget-completions=zsh)"

# Fish
rewget --rewget-completions=fish | source
```

### Use as Default wget

```bash
# Add to ~/.bashrc or ~/.zshrc
alias wget='rewget'
```

## CLI Reference

| Flag | Description |
|------|-------------|
| `--rewget-no-fallback` | Disable automatic retry on block |
| `--rewget-js` | Force JavaScript preflight (Stage 3) |
| `--rewget-js-wait=EVENT` | Wait condition: `load`, `domcontentloaded`, `networkidle` |
| `--rewget-profile=NAME` | Use specific browser profile |
| `--rewget-fallback-codes=N,N` | Only retry on these HTTP status codes |
| `--rewget-engine=wget\|wget2` | Choose wget engine |
| `--rewget-list-profiles` | List available browser profiles |
| `--rewget-update-profiles` | Fetch latest profiles (Ed25519 verified) |
| `--rewget-version` | Show rewget version |

See `man rewget` or the [CLI Reference](https://docs.neullabs.com/rewget/cli) for full details.

---

## For Developers

### Architecture

```
┌─────────┐     ┌─────────┐     ┌─────────────┐
│  rewget  │────▶│ rewgetd  │────▶│  Chromium   │
│  (CLI)  │ IPC │(daemon) │     │ (Stage 3)   │
└─────────┘     └─────────┘     └─────────────┘
     │               │
     │               ├── rquest (Stage 2 impersonation)
     │               └── Browser profile pool
     │
     └── wget/wget2 engine (Stage 1)
```

- **rewget**: Thin CLI shim, parses `--rewget-*` flags, forwards everything else to wget
- **rewgetd**: Daemon handling Stage 2/3, manages browser pool and TLS sessions
- **rewget-core**: Shared library with detection, caching, and profile logic

### Building from Source

```bash
git clone https://github.com/neul-labs/rewget
cd rewget
cargo build --release

# Binaries in target/release/
./target/release/rewget --rewget-version
```

### Running Tests

```bash
cargo test
# 42 tests covering all stages and edge cases
```

### Project Structure

```
crates/
├── rewget/       # CLI binary
├── rewgetd/      # Daemon binary
└── rewget-core/  # Shared library
docs/
├── architecture.md
├── cli.md
├── daemon.md
└── ...
```

### Tech Stack

- **Rust** with Tokio async runtime
- **rquest** for TLS/HTTP2 fingerprint impersonation
- **chromiumoxide** for headless browser control
- **nng** for IPC between CLI and daemon
- **mimalloc** for optimized memory allocation

## Documentation

| Document | Description |
|----------|-------------|
| [Installation](docs/installation.md) | Platform-specific setup |
| [Overview](docs/overview.md) | Goals and terminology |
| [Architecture](docs/architecture.md) | Execution flow and stages |
| [CLI Reference](docs/cli.md) | All flags and options |
| [Impersonation](docs/impersonation.md) | TLS fingerprinting details |
| [Daemon](docs/daemon.md) | Daemon internals |

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

---

*Built with Rust. Made for humans who just want downloads to work.*
