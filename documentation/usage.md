# Usage Guide

This guide covers all rwget features in detail.

## Understanding the Fallback Stages

rwget uses a three-stage fallback system:

### Stage 1: Plain wget

The fastest option. rwget runs wget directly with your arguments.

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
rwget --rwget-no-fallback https://example.com/
```

The command exits with wget's original exit code if blocked.

### Start at a Specific Stage

Skip earlier stages:

```bash
# Start at Stage 2 (impersonation)
rwget --rwget-fallback-stage=2 https://example.com/

# Start at Stage 3 (JS preflight)
rwget --rwget-fallback-stage=3 https://example.com/
# Or use the shorthand:
rwget --rwget-js https://example.com/
```

### Custom Fallback Codes

By default, rwget retries on 403, 429, 503, and 520-529. Customize this:

```bash
# Only retry on 403
rwget --rwget-fallback-codes=403 https://example.com/

# Retry on specific codes
rwget --rwget-fallback-codes=403,429,503 https://example.com/
```

### Disable Body Detection

rwget also checks response bodies for bot detection signatures. Disable this:

```bash
rwget --rwget-no-body-detection https://example.com/
```

## Browser Profiles

### List Available Profiles

```bash
rwget --rwget-list-profiles
```

### Use a Specific Profile

```bash
rwget --rwget-profile=chrome_131 https://example.com/
rwget --rwget-profile=firefox_136 https://example.com/
rwget --rwget-profile=safari_18 https://example.com/
```

### Update Profiles

Keep profiles current with the latest browser fingerprints:

```bash
rwget --rwget-update-profiles
```

Use a custom profile URL:

```bash
rwget --rwget-profile-url=https://my-server.com/profiles.json --rwget-update-profiles
```

Skip signature verification (not recommended):

```bash
rwget --rwget-no-verify --rwget-update-profiles
```

### Verify a Profile

Check a profile's fingerprint details:

```bash
rwget --rwget-verify-profile=chrome_131
```

## JavaScript Preflight Options

### Wait Conditions

Control when Stage 3 considers the page ready:

```bash
# Wait for network to be idle
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/

# Wait for specific element
rwget --rwget-js --rwget-js-wait=selector:#content https://example.com/

# Wait fixed time (milliseconds)
rwget --rwget-js --rwget-js-wait=delay:5000 https://example.com/
```

### Chromium Management

Pre-download Chromium:

```bash
rwget --rwget-download-chromium
```

Check Chromium installation:

```bash
rwget --rwget-chromium-path
```

## Domain Stage Caching

When a stage succeeds for a domain, rwget caches it to skip failed stages on future requests.

### How It Works

```bash
# First request - tries Stage 1, fails, Stage 2 succeeds
$ rwget https://protected.example.com/file1.txt
[rwget] 403 Forbidden - retrying with impersonation...
[rwget] Success at Stage 2 (chrome_131)
[rwget] Cached: protected.example.com → Stage 2

# Second request - starts at Stage 2
$ rwget https://protected.example.com/file2.txt
[rwget] Using cached Stage 2 for protected.example.com
```

### Cache Location

```
~/.cache/rwget/stage-cache.json
```

### Clear Cache

```bash
rwget --rwget-clear-cache
```

### Disable Cache

```bash
rwget --rwget-no-cache https://example.com/
```

## Timeouts

Control how long each stage waits:

```bash
# Stage 1 timeout (uses wget's settings by default)
rwget --rwget-timeout-stage1=30000 https://example.com/

# Stage 2 timeout (default: 15 seconds)
rwget --rwget-timeout-stage2=30000 https://example.com/

# Stage 3 timeout (default: 30 seconds)
rwget --rwget-timeout-stage3=60000 https://example.com/
```

## Daemon Control

rwget uses a daemon (`rwgetd`) for Stage 2/3 operations.

### Daemon Modes

```bash
# Auto mode (default) - starts daemon when needed
rwget --rwget-daemon=auto https://example.com/

# Always use daemon
rwget --rwget-daemon=on https://example.com/

# Never use daemon (Stage 1 only)
rwget --rwget-daemon=off https://example.com/
```

## Engine Selection

Choose between wget and wget2:

```bash
# Use wget (default)
rwget --rwget-engine=wget https://example.com/

# Use wget2
rwget --rwget-engine=wget2 https://example.com/

# Set via environment variable
RWGET_ENGINE=wget2 rwget https://example.com/
```

## Output Control

### Quiet Mode

Suppress rwget status messages:

```bash
rwget --rwget-quiet https://example.com/
```

### Debug Mode

Enable verbose debug output:

```bash
rwget --rwget-debug https://example.com/
```

## Combining Options

Options can be combined:

```bash
rwget \
  --rwget-profile=chrome_131 \
  --rwget-fallback-codes=403,429 \
  --rwget-timeout-stage2=30000 \
  --rwget-quiet \
  -O output.html \
  https://protected-site.com/page
```

## Exit Codes

rwget uses wget's exit codes:

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

When fallback succeeds, rwget exits with 0 regardless of which stage worked.
