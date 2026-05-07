# rewget

> **wget, but it actually works.**

A drop-in replacement for `wget` that automatically bypasses bot protection. When sites block standard wget with 403s, CAPTCHAs, or TLS fingerprinting, rewget seamlessly retries with browser-like behavior — no manual intervention required.

---

## Install

```bash
pip install rewget
```

That's it. The correct native binary for your platform is downloaded automatically on first use.

## Usage

Use it exactly like wget:

```bash
# Basic download (automatic fallback on block)
rewget https://example.com/file.tar.gz

# Scripting mode (fail fast, no retries)
rewget --rewget-no-fallback https://example.com/file.tar.gz

# Force headless browser for JavaScript challenges
rewget --rewget-js https://heavily-protected.com/

# Pick a specific browser profile
rewget --rewget-profile=firefox136 https://example.com/
```

All standard wget options work unchanged.

## Why rewget?

| Problem | Solution |
|---------|----------|
| Site returns **403 Forbidden** | Retries with Chrome/Firefox TLS fingerprint |
| **CAPTCHA** or challenge page | Runs headless Chromium to solve it |
| **Rate limited** (429) | Progressive fallback with session reuse |
| Works in browser but **not wget** | rewget makes it just work |

## How It Works

```
Stage 1: Plain wget     → Fast, zero overhead
    ↓ (blocked?)
Stage 2: Impersonate    → Browser TLS/HTTP2 fingerprint via rquest
    ↓ (still blocked?)
Stage 3: JS Preflight   → Real headless Chromium session
```

Results are cached per-domain, so subsequent requests skip straight to the working stage.

## Requirements

- **Python >= 3.8**
- **wget** installed on your system (Stage 1 dependency)

## Documentation

Full docs: [https://docs.neullabs.com/rewget](https://docs.neullabs.com/rewget)

Repository: [https://github.com/neul-labs/rewget](https://github.com/neul-labs/rewget)

## License

MIT — see [LICENSE](https://github.com/neul-labs/rewget/blob/main/LICENSE) for details.
