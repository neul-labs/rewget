# Roadmap

This document outlines the implementation phases for rwget. Each phase builds on the previous and has clear deliverables.

## Current Status (January 2026)

```
┌─────────────────────────────────────────────────────────────┐
│  rwget v0.1.0 - Phases 0-6 Complete                         │
├─────────────────────────────────────────────────────────────┤
│  ✅ Phase 0: Foundation           - CLI, args, engine       │
│  ✅ Phase 1: Failure Detection    - 403/429/503 detection   │
│  ✅ Phase 2: Daemon Infrastructure - nng IPC, rwgetd        │
│  ✅ Phase 3: Impersonation        - TLS/HTTP2 fingerprints  │
│  ✅ Phase 4: JS Preflight         - Headless Chromium       │
│  ✅ Phase 5: Profile Updates      - Ed25519 signed updates  │
│  ✅ Phase 6: Cross-Platform       - Linux/macOS/Windows     │
│  ⏳ Phase 7: Polish & 1.0         - Final cleanup           │
├─────────────────────────────────────────────────────────────┤
│  Tests: 56 passing | Platforms: 5 targets | Profiles: 6    │
└─────────────────────────────────────────────────────────────┘
```

### Key Features Implemented

| Feature | Description |
|---------|-------------|
| 3-stage fallback | wget → impersonation → JS preflight |
| Browser profiles | Chrome 131/130, Firefox 136/133, Safari 18, Edge 131 |
| Domain caching | 7-day TTL per-domain stage memory |
| Auto Chromium | Downloads Chrome for Testing (~150MB) on first use |
| Remote updates | `--rwget-update-profiles` with Ed25519 verification |
| Cross-platform | Linux x64/arm64, macOS x64/arm64, Windows x64 |
| CI/CD | GitHub Actions for all platforms |

---

## Phase 0: Foundation ✅ COMPLETE

**Goal**: Basic CLI that passes through to wget/wget2 with strict mode working.

### Deliverables

- [x] `rwget` binary that parses `--rwget-*` flags
- [x] `--rwget-no-fallback` exec's wget directly (zero overhead)
- [x] `--rwget-engine=wget|wget2` flag
- [x] `--rwget-version` and `--rwget-help`
- [x] Strict mode golden tests passing on Linux

### Technical Tasks

1. ✅ Set up Rust project structure (`rwget`, `rwgetd`, `rwget-core`)
2. ✅ Implement argument parser that separates `--rwget-*` from wget flags
3. ✅ Implement exec path for `--rwget-no-fallback`
4. ⏭️ Bundle `wget_engine` (using system wget for now)
5. ⏭️ Bundle `wget2_engine` (using system wget2 for now)
6. ✅ Implement engine selection logic
7. ⏭️ Set up CI with golden test harness
8. ⏭️ Package for Linux x86_64

### Engine Selection

```bash
# Default: wget
rwget https://example.com/file.txt

# Explicit wget2
rwget --rwget-engine=wget2 https://example.com/file.txt

# Environment variable
RWGET_ENGINE=wget2 rwget https://example.com/file.txt
```

### Exit Criteria

```bash
rwget --rwget-no-fallback https://example.com/file.txt
# Identical to: wget https://example.com/file.txt

rwget --rwget-no-fallback --rwget-engine=wget2 https://example.com/file.txt
# Identical to: wget2 https://example.com/file.txt
```

---

## Phase 1: Failure Detection ✅ COMPLETE

**Goal**: Detect when wget fails due to bot protection and prepare for fallback.

### Deliverables

- [x] Spawn wget as subprocess (not exec) in default mode
- [x] Capture exit code and detect 403/429/503
- [x] Buffer small HTML responses for body pattern detection
- [x] `--rwget-fallback-codes` configuration
- [x] `--rwget-no-body-detection` flag
- [x] Fallback messages to stderr

### Technical Tasks

1. ✅ Implement subprocess spawning with output capture
2. ✅ Parse wget exit codes to HTTP status mapping
3. ✅ Implement body pattern matching (Cloudflare signatures, etc.)
4. ✅ Add `--rwget-quiet` flag
5. ✅ Add fallback message formatting

### Exit Criteria

```bash
rwget https://protected-site.com/
# [rwget] 403 Forbidden - would retry with impersonation...
# (exits with wget's exit code, no actual retry yet)
```

---

## Phase 2: Daemon Infrastructure ✅ COMPLETE

**Goal**: Working daemon that can execute wget jobs and stream output.

### Deliverables

- [x] `rwgetd` binary with nng IPC
- [x] Inline daemon spawning from `rwget`
- [x] Stage 2/3 RPC handlers
- [ ] Idle timeout auto-shutdown
- [x] `--rwget-daemon=auto|on|off`

### Technical Tasks

1. ✅ Implement `rwgetd` with nng listener
2. ✅ Define JSON schema for RPC (Request/Response in ipc.rs)
3. ✅ Implement Stage 2/3 handlers
4. ✅ Implement client-side daemon spawning
5. ✅ Add health check RPC (status command)
6. ⏭️ Test signal propagation (Ctrl-C)

### Exit Criteria

```bash
rwget --rwget-daemon=on https://example.com/file.txt
# Works identically to direct wget, but via daemon
```

---

## Phase 3: Impersonation (Stage 2) ✅ COMPLETE

**Goal**: Browser-like TLS/HTTP2 fingerprinting without a real browser.

### Deliverables

- [x] Custom TLS client with configurable fingerprint (via rquest)
- [x] Custom HTTP/2 client with configurable SETTINGS (via rquest)
- [x] Profile JSON format and bundled profiles
- [x] `--rwget-profile` flag
- [x] `--rwget-list-profiles`
- [x] Stage 1 → Stage 2 automatic fallback
- [x] Domain-level stage caching
- [x] `--rwget-no-cache` and `--rwget-clear-cache` flags

### Technical Tasks

1. ✅ Use `rquest` with Emulation API for fingerprint control
2. ✅ Use `rquest` built-in HTTP/2 SETTINGS control
3. ✅ Implement profile loader (JSON → config)
4. ✅ Create 6 browser profiles (Chrome 131/130, Firefox 136/133, Safari 18, Edge 131)
5. ✅ Implement response handling with body/file output
6. ⏭️ Fingerprint verification tests against browserleaks.com

### Dependencies

- Phase 2 (daemon infrastructure)

### Domain Stage Cache

When Stage 2 succeeds for a domain, cache it to skip Stage 1 on future requests:

```bash
# First request: tries Stage 1, fails, Stage 2 succeeds
rwget https://protected.example.com/file1.txt
# [rwget] 403 Forbidden - retrying with impersonation...
# [rwget] Success at Stage 2 (chrome_120)
# [rwget] Cached: protected.example.com → Stage 2

# Second request: starts at Stage 2
rwget https://protected.example.com/file2.txt
# [rwget] Using cached Stage 2 for protected.example.com
# [rwget] Success at Stage 2 (chrome_120)
```

Cache stored in `~/.cache/rwget/stage-cache.json`:
```json
{
  "protected.example.com": {"stage": 2, "profile": "chrome_120", "expires": 1704067200},
  "cloudflare-site.net": {"stage": 3, "expires": 1704067200}
}
```

Flags:
- `--rwget-no-cache`: Disable stage caching, always start at Stage 1
- `--rwget-clear-cache`: Clear the stage cache

### Exit Criteria

```bash
rwget https://cloudflare-protected-site.com/
# [rwget] 403 Forbidden - retrying with impersonation...
# [rwget] Success at Stage 2 (chrome_120)
# (file downloads successfully)
```

---

## Phase 4: JS Preflight (Stage 3) ✅ COMPLETE

**Goal**: Full browser session for sites requiring JavaScript execution.

### Deliverables

- [x] Lazy Chromium download on first Stage 3 use
- [x] Headless Chromium integration in daemon
- [ ] Browser pool management
- [x] `--rwget-js` flag
- [x] `--rwget-js-wait` conditions
- [x] Stage 2 → Stage 3 automatic fallback
- [x] Cookie export from browser session

### Technical Tasks

1. ✅ Implement Chromium downloader (auto-download on first Stage 3 use, ~150MB)
2. ✅ Store Chromium in `~/.local/share/rwget/chromium/`
3. ✅ Integrate `chromiumoxide` for headless browser control
4. ⏭️ Implement browser pool with warm instances
5. ✅ Implement navigation and wait conditions (delay, selector, networkidle)
6. ✅ Implement cookie extraction from browser
7. ⏭️ Handle Cloudflare Turnstile and similar challenges
8. ✅ Implement per-stage timeouts

### Chromium Management

```bash
# First Stage 3 use triggers download
rwget https://js-protected-site.com/
# [rwget] Stage 3 requires Chromium. Downloading (~150MB)...
# [rwget] Chromium downloaded to ~/.local/share/rwget/chromium/
# [rwget] Success at Stage 3 (JS preflight)

# Manual management
rwget --rwget-download-chromium   # Pre-download
rwget --rwget-chromium-path       # Show install path
```

### Dependencies

- Phase 2 (daemon infrastructure)
- Phase 3 (impersonation, for fallback chain)

### Exit Criteria

```bash
rwget https://heavily-protected-site.com/
# [rwget] 403 Forbidden - retrying with impersonation...
# [rwget] 403 Forbidden - retrying with JS preflight...
# [rwget] Success at Stage 3 (JS preflight)
# (file downloads successfully)
```

---

## Phase 5: Profile Updates ✅ COMPLETE

**Goal**: Keep browser fingerprints up-to-date without new releases.

### Deliverables

- [ ] Profile update server infrastructure (needs hosting)
- [x] Ed25519 signed profile verification
- [x] `--rwget-update-profiles` command (fetches from remote URL)
- [x] `--rwget-verify-profile` command
- [x] `--rwget-profile-url` custom URL flag
- [x] `--rwget-no-verify` skip signature verification
- [ ] Auto-update check (optional, off by default)

### Technical Tasks

1. ⏭️ Set up profile distribution CDN
2. ⏭️ Implement profile capture automation (Selenium/Playwright)
3. ⏭️ Implement signing infrastructure (key generation/management)
4. ✅ Implement local profile management
5. ✅ Implement profile verification (SHA256 + Ed25519)

### Dependencies

- Phase 3 (impersonation, profiles to update)

### Exit Criteria

```bash
rwget --rwget-update-profiles
# [rwget] Downloading profile index...
# [rwget] Updated: chrome_125, firefox_126
# [rwget] 2 profiles updated
```

---

## Phase 6: Cross-Platform ✅ COMPLETE

**Goal**: Full support for macOS and Windows.

### Deliverables

- [x] macOS builds (x86_64, arm64)
- [x] Windows builds (x86_64)
- [ ] Platform-specific installers
- [ ] Golden tests passing on all platforms

### Technical Tasks

1. ✅ Set up cross-compilation in CI (GitHub Actions)
2. ✅ Handle platform-specific paths (XDG, Library, APPDATA)
3. ✅ Handle platform-specific Chromium download (wget/curl vs PowerShell)
4. ⏭️ Create macOS .pkg or Homebrew formula
5. ⏭️ Create Windows installer or winget manifest
6. ✅ Platform-specific IPC socket paths

### Dependencies

- Phase 4 (feature complete)

### Exit Criteria

- All golden tests pass on Linux, macOS, Windows
- Install scripts work on all platforms

---

## Phase 7: Polish & Optimization

**Goal**: Production-ready quality and performance.

### Deliverables

- [ ] Performance benchmarks
- [ ] Memory usage optimization
- [ ] Startup time optimization
- [ ] Comprehensive error messages
- [ ] Man page and shell completions

### Technical Tasks

1. Profile and optimize cold start time
2. Reduce binary size (strip, LTO)
3. Implement connection pooling in Stage 2
4. Add shell completions (bash, zsh, fish, PowerShell)
5. Generate man page from CLI docs
6. Audit and improve error messages

### Dependencies

- Phase 6 (cross-platform)

---

## Milestone Summary

| Milestone | Phases | Description | Status |
|-----------|--------|-------------|--------|
| M1: Proof of Concept | 0-1 | CLI works, detects failures | ✅ Complete |
| M2: Daemon Working | 2 | IPC infrastructure complete | ✅ Complete |
| M3: Impersonation MVP | 3 | Stage 2 bypasses most bot detection | ✅ Complete |
| M4: Full Fallback | 4 | All 3 stages working | ✅ Complete |
| M5: Self-Updating | 5 | Profiles stay current | ✅ Complete |
| M6: Cross-Platform | 6 | Works everywhere | ✅ Complete |
| M7: 1.0 Release | 7 | Production ready | ⏳ Pending |

## Dependency Graph

```
Phase 0 (Foundation)
    │
    ▼
Phase 1 (Failure Detection)
    │
    ▼
Phase 2 (Daemon) ──────────────────┐
    │                              │
    ▼                              ▼
Phase 3 (Impersonation)      Phase 4 (JS Preflight)
    │                              │
    ▼                              │
Phase 5 (Profile Updates)          │
    │                              │
    └──────────────┬───────────────┘
                   ▼
            Phase 6 (Cross-Platform)
                   │
                   ▼
            Phase 7 (Polish)
```

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| IPC transport | nng | Cross-platform, lightweight, supports streaming |
| Chromium | Lazy download | Download on first Stage 3 use, ~150MB |
| wget2 | Supported | Alternative engine, user-selectable |
| Stage caching | Per-domain | Remember successful stage, skip failed stages |

## Open Questions

### Technical

1. **Chromium version pinning**: How often to update bundled Chromium version?

### Product

1. **Telemetry**: Should we collect anonymous usage stats for profile prioritization?
2. **Commercial**: Is there a paid tier for faster profile updates or support?

## Contributing

See `CONTRIBUTING.md` for how to help with implementation. Priority areas for Phase 7:
- Performance optimization and benchmarking
- Shell completions (bash, zsh, fish, PowerShell)
- Man page generation
- Platform-specific installers (Homebrew, winget)
- Profile distribution infrastructure

## Changelog

### v0.1.0 (January 2026)
- Initial release with Phases 0-6 complete
- Full 3-stage fallback system (wget → impersonation → JS preflight)
- 6 browser profiles with Ed25519-signed remote updates
- Cross-platform support (Linux, macOS, Windows)
- GitHub Actions CI/CD for 5 build targets
- 56 unit tests passing
