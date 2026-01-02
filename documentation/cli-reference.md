# CLI Reference

Complete reference for all rwget command-line options.

## Synopsis

```
rwget [RWGET_OPTIONS] [WGET_OPTIONS] [URL...]
```

rwget accepts all wget options unchanged. Options starting with `--rwget-*` are processed by rwget; all others are passed to wget.

## rwget Options

### Core

| Option | Description |
|--------|-------------|
| `--rwget-no-fallback` | Disable fallback, behave exactly like wget |
| `--rwget-engine=ENGINE` | Select wget engine: `wget` (default), `wget2` |
| `--rwget-quiet` | Suppress rwget status messages |
| `--rwget-debug` | Enable verbose debug output |

### Fallback Control

| Option | Description |
|--------|-------------|
| `--rwget-fallback-codes=CODES` | HTTP codes that trigger fallback (default: `403,429,503,520-529`) |
| `--rwget-fallback-stage=N` | Start at stage N: `1`=wget, `2`=impersonate, `3`=js |
| `--rwget-no-body-detection` | Disable HTML body pattern detection |

### Browser Profiles

| Option | Description |
|--------|-------------|
| `--rwget-profile=NAME` | Use specific browser profile |
| `--rwget-list-profiles` | List available profiles and exit |
| `--rwget-update-profiles` | Update profiles from remote and exit |
| `--rwget-profile-url=URL` | Custom profile update URL |
| `--rwget-no-verify` | Skip Ed25519 signature verification |
| `--rwget-verify-profile=NAME` | Verify profile fingerprints and exit |

### JavaScript Preflight

| Option | Description |
|--------|-------------|
| `--rwget-js` | Force JavaScript preflight (Stage 3) |
| `--rwget-js-wait=COND` | Wait condition: `networkidle`, `selector:CSS`, `delay:MS` |
| `--rwget-download-chromium` | Pre-download Chromium and exit |
| `--rwget-chromium-path` | Print Chromium path and exit |

### Timeouts

| Option | Description |
|--------|-------------|
| `--rwget-timeout-stage1=MS` | Stage 1 timeout (default: wget settings) |
| `--rwget-timeout-stage2=MS` | Stage 2 timeout (default: 15000) |
| `--rwget-timeout-stage3=MS` | Stage 3 timeout (default: 30000) |

### Caching

| Option | Description |
|--------|-------------|
| `--rwget-no-cache` | Disable domain stage caching |
| `--rwget-clear-cache` | Clear stage cache and exit |

### Daemon

| Option | Description |
|--------|-------------|
| `--rwget-daemon=MODE` | Daemon mode: `auto` (default), `on`, `off` |

### Information

| Option | Description |
|--------|-------------|
| `--rwget-version` | Print version and exit |
| `--rwget-help` | Print help and exit |
| `--rwget-completions=SHELL` | Generate completions: `bash`, `zsh`, `fish`, `powershell` |

## Common wget Options

These are the most commonly used wget options. All wget options work with rwget.

### Output

| Option | Description |
|--------|-------------|
| `-O FILE` | Write to FILE |
| `-P DIR` | Save files to DIR |
| `-c, --continue` | Resume partial download |
| `-N, --timestamping` | Only download newer files |

### Download Control

| Option | Description |
|--------|-------------|
| `-t NUM, --tries=NUM` | Retry NUM times (0 = unlimited) |
| `-T SEC, --timeout=SEC` | Set timeout |
| `--limit-rate=RATE` | Limit download speed |
| `-w SEC, --wait=SEC` | Wait between requests |

### HTTP Options

| Option | Description |
|--------|-------------|
| `--header=STRING` | Add custom header |
| `--user-agent=AGENT` | Set User-Agent |
| `--referer=URL` | Set Referer header |
| `--post-data=STRING` | POST data |
| `--post-file=FILE` | POST file contents |

### Authentication

| Option | Description |
|--------|-------------|
| `--user=USER` | HTTP username |
| `--password=PASS` | HTTP password |
| `--http-user=USER` | HTTP username |
| `--http-password=PASS` | HTTP password |

### SSL/TLS

| Option | Description |
|--------|-------------|
| `--no-check-certificate` | Don't verify SSL certificates |
| `--ca-certificate=FILE` | CA certificate file |

### Recursive Download

| Option | Description |
|--------|-------------|
| `-r, --recursive` | Recursive download |
| `-l NUM, --level=NUM` | Maximum recursion depth |
| `-k, --convert-links` | Convert links for local viewing |
| `-p, --page-requisites` | Download page prerequisites |

For complete wget documentation, run `wget --help` or `man wget`.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RWGET_ENGINE` | Default wget engine |

## Exit Codes

rwget uses wget's exit codes:

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Generic error |
| 2 | Parse error |
| 3 | File I/O error |
| 4 | Network failure |
| 5 | SSL verification failure |
| 6 | Authentication failure |
| 7 | Protocol error |
| 8 | Server error |

## Examples

### Basic Usage

```bash
# Simple download
rwget https://example.com/file.tar.gz

# Save with specific filename
rwget -O myfile.tar.gz https://example.com/file.tar.gz

# Resume interrupted download
rwget -c https://example.com/large-file.iso
```

### Fallback Control

```bash
# Disable fallback (strict mode)
rwget --rwget-no-fallback https://example.com/

# Only retry on 403
rwget --rwget-fallback-codes=403 https://example.com/

# Start at Stage 2
rwget --rwget-fallback-stage=2 https://example.com/
```

### JavaScript Sites

```bash
# Force JS preflight
rwget --rwget-js https://protected.example.com/

# Wait for content to load
rwget --rwget-js --rwget-js-wait=networkidle https://protected.example.com/

# Wait for specific element
rwget --rwget-js --rwget-js-wait=selector:#main-content https://example.com/
```

### Profile Management

```bash
# List profiles
rwget --rwget-list-profiles

# Use specific profile
rwget --rwget-profile=firefox_136 https://example.com/

# Update profiles
rwget --rwget-update-profiles

# Verify profile
rwget --rwget-verify-profile=chrome_131
```

### Debugging

```bash
# Debug output
rwget --rwget-debug https://example.com/

# Quiet mode
rwget --rwget-quiet https://example.com/
```

### Combining Options

```bash
rwget \
  --rwget-profile=chrome_131 \
  --rwget-timeout-stage2=30000 \
  --rwget-fallback-codes=403,429,503 \
  --rwget-quiet \
  -O output.html \
  --limit-rate=1M \
  https://example.com/page
```
