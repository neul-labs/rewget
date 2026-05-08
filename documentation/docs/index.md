# rewget

**wget-compatible wrapper with automatic fallback**

rewget is a drop-in replacement for wget that automatically retries with browser emulation when websites block standard wget requests.

<div class="grid cards" markdown>

-   :material-download:{ .lg .middle } __Drop-in Replacement__

    ---

    Use rewget exactly like wget. All wget options work unchanged.

    ```bash
    rewget https://example.com/file.tar.gz
    ```

-   :material-shield-check:{ .lg .middle } __Automatic Bypass__

    ---

    Automatically bypasses bot protection using browser emulation.

    No manual configuration needed.

-   :material-rocket-launch:{ .lg .middle } __Three-Stage Fallback__

    ---

    1. Plain wget (fast)
    2. TLS impersonation (stealth)
    3. Full browser (JavaScript)

-   :material-cog:{ .lg .middle } __Highly Configurable__

    ---

    Fine-tune behavior with `--rewget-*` flags while keeping wget semantics.

</div>

## Quick Example

```bash
# Download with automatic fallback
rewget https://protected-site.com/file.tar.gz

# If the site blocks wget, rewget automatically:
# 1. Detects the 403/429 response
# 2. Retries with browser-like TLS fingerprint
# 3. Falls back to full browser if needed
```

## Why rewget?

Many websites now block wget and curl with bot detection systems like:

- **Cloudflare** - TLS fingerprinting, JavaScript challenges
- **Akamai** - Bot Manager, behavioral analysis
- **PerimeterX** - Advanced bot detection
- **DataDome** - Real-time bot protection

rewget handles these automatically by progressively escalating through detection bypass techniques.

## Features

| Feature | Description |
|---------|-------------|
| **3-Stage Fallback** | wget → TLS impersonation → JavaScript preflight |
| **6 Browser Profiles** | Chrome, Firefox, Safari, Edge with accurate fingerprints |
| **Domain Caching** | Remembers successful stage per domain (7-day TTL) |
| **Auto Chromium** | Downloads browser on first use (~150MB) |
| **Remote Updates** | Keep profiles current with `--rewget-update-profiles` |
| **Cross-Platform** | Linux, macOS |

## Installation

=== "Homebrew"

    ```bash
    brew install neul-labs/tap/rewget
    ```

=== "npm"

    ```bash
    npm install -g rewget
    ```

=== "PyPI"

    ```bash
    pip install rewget
    ```

=== "Install Script"

    ```bash
    curl -fsSL https://rewget.dev/install.sh | sh
    ```

=== "From Source"

    ```bash
    git clone https://github.com/neul-labs/rewget
    cd rewget
    cargo build --release
    ```

[Get Started :material-arrow-right:](installation.md){ .md-button .md-button--primary }
[View on GitHub :material-github:](https://github.com/neul-labs/rewget){ .md-button }

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                         rewget                                │
├─────────────────────────────────────────────────────────────┤
│  Stage 1: wget          "Try plain wget first"              │
│     ↓ (403/429?)                                            │
│  Stage 2: Impersonate   "Retry with browser TLS"            │
│     ↓ (still blocked?)                                      │
│  Stage 3: JS Preflight  "Full browser session"              │
└─────────────────────────────────────────────────────────────┘
```

rewget starts with the fastest option (plain wget) and only escalates when needed. Most downloads complete at Stage 1 with zero overhead.

## License

MIT License - see [LICENSE](https://github.com/neul-labs/rewget/blob/main/LICENSE) for details.
