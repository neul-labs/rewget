# Usage Guide

This guide covers all rewget features in detail.

## Understanding the Fallback Stages

rewget uses a three-stage fallback system:

### Stage 1: Plain wget

The fastest option. rewget runs wget directly with your arguments.

- **Overhead**: Zero
- **Success rate**: Works for most unprotected sites
- **When it fails**: 403, 429, 503, or bot detection pages

### Stage 2: TLS Impersonation

Retries with browser-like TLS and HTTP/2 fingerprints.

- **Overhead**: ~100ms for daemon startup (first request only)
- **Success rate**: Bypasses most TLS fingerprinting
- **Technique**: Mimics Chrome/Firefox TLS handshake and HTTP/2 settings

### Stage 3: JavaScript Preflight

Full headless browser session for JavaScript challenges.

- **Overhead**: 2-10 seconds (browser startup + page load)
- **Success rate**: Handles Cloudflare challenges, CAPTCHAs (some)
- **Technique**: Real Chromium browser executes JavaScript

## Controlling Fallback Behavior

### Disable Fallback

For scripts that need predictable behavior:

```bash
rewget --rewget-no-fallback https://example.com/
```

The command exits with wget's original exit code if blocked.

### Start at a Specific Stage

Skip earlier stages:

```bash
# Start at Stage 2 (impersonation)
rewget --rewget-fallback-stage=2 https://example.com/

# Start at Stage 3 (JS preflight)
rewget --rewget-fallback-stage=3 https://example.com/
# Or use the shorthand:
rewget --rewget-js https://example.com/
```

### Custom Fallback Codes

By default, rewget retries on 403, 429, 503, and 520-529. Customize this:

```bash
# Only retry on 403
rewget --rewget-fallback-codes=403 https://example.com/

# Retry on specific codes
rewget --rewget-fallback-codes=403,429,503 https://example.com/
```

### Disable Body Detection

rewget also checks response bodies for bot detection signatures. Disable this:

```bash
rewget --rewget-no-body-detection https://example.com/
```

## Browser Profiles

### List Available Profiles

```bash
rewget --rewget-list-profiles
```

### Use a Specific Profile

```bash
rewget --rewget-profile=chrome_131 https://example.com/
rewget --rewget-profile=firefox_136 https://example.com/
rewget --rewget-profile=safari_18 https://example.com/
```

### Update Profiles

Keep profiles current with the latest browser fingerprints:

```bash
rewget --rewget-update-profiles
```

Use a custom profile URL:

```bash
rewget --rewget-profile-url=https://my-server.com/profiles.json --rewget-update-profiles
```

Skip signature verification (not recommended):

```bash
rewget --rewget-no-verify --rewget-update-profiles
```

### Verify a Profile

Check a profile's fingerprint details:

```bash
rewget --rewget-verify-profile=chrome_131
```

## JavaScript Preflight Options

### Wait Conditions

Control when Stage 3 considers the page ready:

```bash
# Wait for network to be idle
rewget --rewget-js --rewget-js-wait=networkidle https://example.com/

# Wait for specific element
rewget --rewget-js --rewget-js-wait=selector:#content https://example.com/

# Wait fixed time (milliseconds)
rewget --rewget-js --rewget-js-wait=delay:5000 https://example.com/
```

### Chromium Management

Pre-download Chromium:

```bash
rewget --rewget-download-chromium
```

Check Chromium installation:

```bash
rewget --rewget-chromium-path
```

## Domain Stage Caching

When a stage succeeds for a domain, rewget caches it to skip failed stages on future requests.

### How It Works

```bash
# First request - tries Stage 1, fails, Stage 2 succeeds
$ rewget https://protected.example.com/file1.txt
[rewget] 403 Forbidden - retrying with impersonation...
[rewget] Success at Stage 2 (chrome_131)
[rewget] Cached: protected.example.com → Stage 2

# Second request - starts at Stage 2
$ rewget https://protected.example.com/file2.txt
[rewget] Using cached Stage 2 for protected.example.com
```

### Cache Location

```
~/.cache/rewget/stage-cache.json
```

### Clear Cache

```bash
rewget --rewget-clear-cache
```

### Disable Cache

```bash
rewget --rewget-no-cache https://example.com/
```

## Timeouts

Control how long each stage waits:

```bash
# Stage 1 timeout (uses wget's settings by default)
rewget --rewget-timeout-stage1=30000 https://example.com/

# Stage 2 timeout (default: 15 seconds)
rewget --rewget-timeout-stage2=30000 https://example.com/

# Stage 3 timeout (default: 30 seconds)
rewget --rewget-timeout-stage3=60000 https://example.com/
```

## Daemon Control

rewget uses a daemon (`rewgetd`) for Stage 2/3 operations.

### Daemon Modes

```bash
# Auto mode (default) - starts daemon when needed
rewget --rewget-daemon=auto https://example.com/

# Always use daemon
rewget --rewget-daemon=on https://example.com/

# Never use daemon (Stage 1 only)
rewget --rewget-daemon=off https://example.com/
```

## Engine Selection

Choose between wget and wget2:

```bash
# Use wget (default)
rewget --rewget-engine=wget https://example.com/

# Use wget2
rewget --rewget-engine=wget2 https://example.com/

# Set via environment variable
RWGET_ENGINE=wget2 rewget https://example.com/
```

## Output Control

### Quiet Mode

Suppress rewget status messages:

```bash
rewget --rewget-quiet https://example.com/
```

### Debug Mode

Enable verbose debug output:

```bash
rewget --rewget-debug https://example.com/
```

## Combining Options

Options can be combined:

```bash
rewget \
  --rewget-profile=chrome_131 \
  --rewget-fallback-codes=403,429 \
  --rewget-timeout-stage2=30000 \
  --rewget-quiet \
  -O output.html \
  https://protected-site.com/page
```

## Exit Codes

rewget uses wget's exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic error |
| 2 | Parse error |
| 3 | File I/O error |
| 4 | Network failure |
| 5 | SSL verification failure |
| 6 | Authentication required |
| 7 | Protocol error |
| 8 | Server error (includes 403, 404, etc.) |

When fallback succeeds, rewget exits with 0 regardless of which stage worked.
