# rwget

rwget is a wget-compatible wrapper with an optional daemon. It starts with a plain wget request and automatically falls back through progressive enhancement stages when encountering blocking responses. Optional `--rwget-*` flags provide additional control while keeping wget's file and logging semantics.

## Principles

- Automatic fallback on failure: when wget encounters blocking responses (403, 429, etc.), rwget progressively retries with impersonation and JS preflight.
- Streaming output preserved byte-for-byte on success.
- Clear separation between the wrapper, daemon, and the wget engine.
- Scripting-friendly: fallback can be disabled with `--rwget-no-fallback` for predictable behavior.

## What rwget is

- A thin shim that accepts every wget option unchanged.
- A smart fallback layer that retries with browser emulation when sites block wget.
- A compatibility layer that can proxy execution through a daemon.
- A framework for preflight and replay that preserves wget semantics.

## What rwget is not

- A new downloader with its own semantics.
- A replacement for wget configuration or logging formats.

## Components

- `rwget`: CLI shim that parses only `--rwget-*` flags.
- `rwgetd`: daemon that handles Stage 2/3, manages browser pool and TLS sessions.
- `wget_engine`: pinned GNU Wget binary (default).
- `wget2_engine`: pinned GNU Wget2 binary (optional, `--rwget-engine=wget2`).

## Operating modes

- Direct exec: `rwget` runs `wget_engine` first.
- Automatic fallback: if wget fails with a blocking status code (403, 429, 503, etc.), rwget retries with progressive enhancement:
  1. **Stage 1**: Plain wget (fast, zero overhead)
  2. **Stage 2**: Impersonation preflight (browser-like headers and TLS fingerprint)
  3. **Stage 3**: Full JS preflight (real browser session for challenge pages)
- Daemon exec: `rwget` sends an RPC to `rwgetd` and streams output. The daemon is started inline on first use for Stage 2/3.
- The daemon auto-shuts down after an idle timeout.

## Installation

```bash
# Linux / macOS
curl -fsSL https://rwget.dev/install.sh | sh

# Windows (PowerShell)
irm https://rwget.dev/install.ps1 | iex
```

To use rwget as your default wget, add an alias to your shell config:

```bash
# Bash/Zsh
alias wget='rwget'
```

See `docs/installation.md` for platform-specific instructions and alternative methods.

## Quick usage

Download with automatic fallback (retries with browser emulation if blocked):

```bash
rwget https://example.com/file.tar.gz
```

Disable fallback for scripting (fail immediately on 403):

```bash
rwget --rwget-no-fallback https://example.com/file.tar.gz
```

Force JS preflight from the start (skip straight to Stage 3):

```bash
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/
```

Custom fallback codes (only retry on these status codes):

```bash
rwget --rwget-fallback-codes=403,429,503 https://example.com/
```

## Documentation

- `docs/installation.md` - platform-specific installation and wget aliasing
- `docs/overview.md` - goals, terminology, and behavior at a glance
- `docs/architecture.md` - execution flow, fallback stages, and failure detection
- `docs/cli.md` - `--rwget-*` flags and usage patterns
- `docs/impersonation.md` - TLS/HTTP/2 fingerprinting and profile format
- `docs/daemon.md` - daemon responsibilities and streaming guarantees
- `docs/compliance.md` - compatibility modes and golden suite expectations
- `docs/roadmap.md` - implementation phases and milestones

## Status

**v0.1.0 - Core functionality complete**

| Feature | Status |
|---------|--------|
| Phase 0: Foundation | ✅ Complete |
| Phase 1: Failure Detection | ✅ Complete |
| Phase 2: Daemon Infrastructure | ✅ Complete |
| Phase 3: Stage 2 Impersonation | ✅ Complete |
| Phase 4: Stage 3 JS Preflight | ✅ Complete |
| Phase 5: Profile Updates | ✅ Complete |
| Phase 6: Cross-Platform | ✅ Complete |
| Phase 7: Polish & 1.0 | ⏳ Pending |

### What works now

- **3-stage fallback**: wget → TLS impersonation → JS preflight
- **6 browser profiles**: Chrome 131/130, Firefox 136/133, Safari 18, Edge 131
- **Domain caching**: Remembers successful stage per domain (7-day TTL)
- **Auto Chromium download**: Downloads Chrome for Testing (~150MB) on first Stage 3 use
- **Remote profile updates**: `--rwget-update-profiles` with Ed25519 signature verification
- **Cross-platform**: Linux, macOS, and Windows support
- **CI/CD**: GitHub Actions for all platforms
- **56 tests passing**

### Build from source

```bash
git clone https://github.com/user/rwget
cd rwget
cargo build --release

# Binaries in target/release/
./target/release/rwget --rwget-version
./target/release/rwget --rwget-list-profiles
```

## Implementation notes

- The primary implementation language is Rust.
- IPC uses nng for request/stream transport between `rwget` and `rwgetd`.
- Stage 2 uses `rquest` with Emulation API for TLS/HTTP2 fingerprinting.
- Stage 3 uses `chromiumoxide` for headless Chromium control.
