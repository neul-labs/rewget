# CLI Reference

Complete reference for all rewget command-line options.

## Synopsis

```
rewget [RWGET_OPTIONS] [WGET_OPTIONS] [URL...]
```

rewget accepts all wget options unchanged. Options starting with `--rewget-*` are processed by rewget; all others are passed to wget.

## rewget Options

### Core

| Option | Description |
|--------|-------------|
| `--rewget-no-fallback` | Disable fallback, behave exactly like wget |
| `--rewget-engine=ENGINE` | Select wget engine: `wget` (default), `wget2` |
| `--rewget-quiet` | Suppress rewget status messages |
| `--rewget-debug` | Enable verbose debug output |

### Fallback Control

| Option | Description |
|--------|-------------|
| `--rewget-fallback-codes=CODES` | HTTP codes that trigger fallback (default: `403,429,503,520-529`) |
| `--rewget-fallback-stage=N` | Start at stage N: `1`=wget, `2`=impersonate, `3`=js |
| `--rewget-no-body-detection` | Disable HTML body pattern detection |

### Browser Profiles

| Option | Description |
|--------|-------------|
| `--rewget-profile=NAME` | Use specific browser profile |
| `--rewget-list-profiles` | List available profiles and exit |
| `--rewget-update-profiles` | Update profiles from remote and exit |
| `--rewget-profile-url=URL` | Custom profile update URL |
| `--rewget-no-verify` | Skip Ed25519 signature verification |
| `--rewget-verify-profile=NAME` | Verify profile fingerprints and exit |

### JavaScript Preflight

| Option | Description |
|--------|-------------|
| `--rewget-js` | Force JavaScript preflight (Stage 3) |
| `--rewget-js-wait=COND` | Wait condition: `networkidle`, `selector:CSS`, `delay:MS` |
| `--rewget-download-chromium` | Pre-download Chromium and exit |
| `--rewget-chromium-path` | Print Chromium path and exit |

### Timeouts

| Option | Description |
|--------|-------------|
| `--rewget-timeout-stage1=MS` | Stage 1 timeout (default: wget settings) |
| `--rewget-timeout-stage2=MS` | Stage 2 timeout (default: 15000) |
| `--rewget-timeout-stage3=MS` | Stage 3 timeout (default: 30000) |

### Caching

| Option | Description |
|--------|-------------|
| `--rewget-no-cache` | Disable domain stage caching |
| `--rewget-clear-cache` | Clear stage cache and exit |

### Daemon

| Option | Description |
|--------|-------------|
| `--rewget-daemon=MODE` | Daemon mode: `auto` (default), `on`, `off` |

### Information

| Option | Description |
|--------|-------------|
| `--rewget-version` | Print version and exit |
| `--rewget-help` | Print help and exit |
| `--rewget-completions=SHELL` | Generate completions: `bash`, `zsh`, `fish`, `powershell` |

## Common wget Options

These are the most commonly used wget options. All wget options work with rewget.

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

rewget uses wget's exit codes:

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
rewget https://example.com/file.tar.gz

# Save with specific filename
rewget -O myfile.tar.gz https://example.com/file.tar.gz

# Resume interrupted download
rewget -c https://example.com/large-file.iso
```

### Fallback Control

```bash
# Disable fallback (strict mode)
rewget --rewget-no-fallback https://example.com/

# Only retry on 403
rewget --rewget-fallback-codes=403 https://example.com/

# Start at Stage 2
rewget --rewget-fallback-stage=2 https://example.com/
```

### JavaScript Sites

```bash
# Force JS preflight
rewget --rewget-js https://protected.example.com/

# Wait for content to load
rewget --rewget-js --rewget-js-wait=networkidle https://protected.example.com/

# Wait for specific element
rewget --rewget-js --rewget-js-wait=selector:#main-content https://example.com/
```

### Profile Management

```bash
# List profiles
rewget --rewget-list-profiles

# Use specific profile
rewget --rewget-profile=firefox_136 https://example.com/

# Update profiles
rewget --rewget-update-profiles

# Verify profile
rewget --rewget-verify-profile=chrome_131
```

### Debugging

```bash
# Debug output
rewget --rewget-debug https://example.com/

# Quiet mode
rewget --rewget-quiet https://example.com/
```

### Combining Options

```bash
rewget \
  --rewget-profile=chrome_131 \
  --rewget-timeout-stage2=30000 \
  --rewget-fallback-codes=403,429,503 \
  --rewget-quiet \
  -O output.html \
  --limit-rate=1M \
  https://example.com/page
```
