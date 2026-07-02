# rewget

> **wget, but it actually works.**

A drop-in replacement for `wget` that automatically bypasses bot protection. When sites block standard wget with 403s, CAPTCHAs, or TLS fingerprinting, rewget seamlessly retries with browser-like behavior — no manual intervention required.

**[Website](https://rewget.neullabs.com) · [Documentation](https://docs.neullabs.com/rewget) · [GitHub](https://github.com/neul-labs/rewget)**

> **See also: [recurl](https://github.com/neul-labs/recurl)** — the `curl` counterpart to rewget.

---

## Install

```bash
npm install -g rewget
```

That's it. Binaries for your platform are downloaded automatically on first install.

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

- **Node.js >= 18** (for this wrapper)
- **wget** installed on your system (Stage 1 dependency)

## Documentation

Full docs: [https://docs.neullabs.com/rewget](https://docs.neullabs.com/rewget)

Repository: [https://github.com/neul-labs/rewget](https://github.com/neul-labs/rewget)

## Part of the Neul Labs toolchain

rewget and [recurl](https://github.com/neul-labs/recurl) are a natural pair — the `wget` and `curl` halves of the same "just works" toolkit:

| Project | What it does |
|---------|--------------|
| [recurl](https://github.com/neul-labs/recurl) | curl that just works — the `curl` counterpart to rewget. |
| [stout](https://github.com/neul-labs/stout) | A drop-in replacement for the Homebrew CLI that's 10-100x faster. |
| [stratafs](https://github.com/neul-labs/stratafs) | A semantic filesystem for AI-era search. |

Explore the full toolchain at [neullabs.com](https://www.neullabs.com).

## License

MIT — see [LICENSE](https://github.com/neul-labs/rewget/blob/main/LICENSE) for details.
