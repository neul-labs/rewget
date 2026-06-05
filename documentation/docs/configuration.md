# Configuration

rewget can be configured through command-line flags and environment variables.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RWGET_ENGINE` | Default wget engine (`wget` or `wget2`) | `wget` |

## Command-Line Flags

All rewget-specific flags use the `--rewget-*` prefix to avoid conflicts with wget flags.

### Core Options

#### `--rewget-no-fallback`

Disable fallback completely. rewget behaves exactly like wget.

```bash
rewget --rewget-no-fallback https://example.com/
```

#### `--rewget-engine=ENGINE`

Select the wget engine to use.

| Value | Description |
|-------|-------------|
| `wget` | GNU Wget (default) |
| `wget2` | GNU Wget2 |

```bash
rewget --rewget-engine=wget2 https://example.com/
```

#### `--rewget-quiet`

Suppress rewget status messages. wget's output is unchanged.

```bash
rewget --rewget-quiet https://example.com/
```

#### `--rewget-debug`

Enable verbose debug output for troubleshooting.

```bash
rewget --rewget-debug https://example.com/
```

### Fallback Control

#### `--rewget-fallback-codes=CODES`

Comma-separated HTTP status codes that trigger fallback.

**Default**: `403,429,503,520,521,522,523,524,525,526,527,528,529`

```bash
rewget --rewget-fallback-codes=403,429 https://example.com/
```

#### `--rewget-fallback-stage=N`

Start at a specific stage instead of Stage 1.

| Value | Stage |
|-------|-------|
| `1` | Plain wget (default) |
| `2` | TLS impersonation |
| `3` | JavaScript preflight |

```bash
rewget --rewget-fallback-stage=2 https://example.com/
```

#### `--rewget-no-body-detection`

Disable HTML body pattern detection for bot protection signatures.

```bash
rewget --rewget-no-body-detection https://example.com/
```

### Browser Profile Options

#### `--rewget-profile=NAME`

Use a specific browser profile for impersonation.

```bash
rewget --rewget-profile=chrome_131 https://example.com/
```

Available profiles:

| Profile | Browser |
|---------|---------|
| `chrome_131` | Chrome 131 on Windows |
| `chrome_130` | Chrome 130 on Windows |
| `firefox_136` | Firefox 136 on Windows |
| `firefox_133` | Firefox 133 on Windows |
| `safari_18` | Safari 18 on macOS |
| `edge_131` | Edge 131 on Windows |

#### `--rewget-list-profiles`

List all available browser profiles.

```bash
rewget --rewget-list-profiles
```

#### `--rewget-update-profiles`

Update browser profiles from the remote server.

```bash
rewget --rewget-update-profiles
```

#### `--rewget-profile-url=URL`

Custom URL for profile updates.

```bash
rewget --rewget-profile-url=https://my-server.com/profiles.json --rewget-update-profiles
```

#### `--rewget-no-verify`

Skip Ed25519 signature verification when updating profiles.

!!! warning
    Only use this for testing or with trusted profile sources.

```bash
rewget --rewget-no-verify --rewget-update-profiles
```

#### `--rewget-verify-profile=NAME`

Display detailed fingerprint information for a profile.

```bash
rewget --rewget-verify-profile=chrome_131
```

### JavaScript Preflight Options

#### `--rewget-js`

Force JavaScript preflight (Stage 3) from the start.

```bash
rewget --rewget-js https://example.com/
```

#### `--rewget-js-wait=CONDITION`

Wait condition for JavaScript preflight.

| Condition | Description |
|-----------|-------------|
| `networkidle` | Wait for network to be idle |
| `selector:CSS` | Wait for CSS selector to match |
| `delay:MS` | Wait fixed milliseconds |

```bash
rewget --rewget-js --rewget-js-wait=networkidle https://example.com/
rewget --rewget-js --rewget-js-wait=selector:#main-content https://example.com/
rewget --rewget-js --rewget-js-wait=delay:5000 https://example.com/
```

#### `--rewget-download-chromium`

Pre-download Chromium for JavaScript preflight.

```bash
rewget --rewget-download-chromium
```

#### `--rewget-chromium-path`

Print the Chromium installation path.

```bash
rewget --rewget-chromium-path
```

### Timeout Options

#### `--rewget-timeout-stage1=MS`

Stage 1 timeout in milliseconds. Uses wget's timeout settings by default.

```bash
rewget --rewget-timeout-stage1=30000 https://example.com/
```

#### `--rewget-timeout-stage2=MS`

Stage 2 timeout in milliseconds.

**Default**: `15000` (15 seconds)

```bash
rewget --rewget-timeout-stage2=30000 https://example.com/
```

#### `--rewget-timeout-stage3=MS`

Stage 3 timeout in milliseconds.

**Default**: `30000` (30 seconds)

```bash
rewget --rewget-timeout-stage3=60000 https://example.com/
```

### Cache Options

#### `--rewget-no-cache`

Disable domain stage caching. Always start at Stage 1.

```bash
rewget --rewget-no-cache https://example.com/
```

#### `--rewget-clear-cache`

Clear the domain stage cache and exit.

```bash
rewget --rewget-clear-cache
```

### Daemon Options

#### `--rewget-daemon=MODE`

Control daemon behavior.

| Mode | Description |
|------|-------------|
| `auto` | Start daemon when needed (default) |
| `on` | Always use daemon |
| `off` | Never use daemon (Stage 1 only) |

```bash
rewget --rewget-daemon=on https://example.com/
```

### Information Options

#### `--rewget-version`

Print rewget version and exit.

```bash
rewget --rewget-version
```

#### `--rewget-help`

Print help message and exit.

```bash
rewget --rewget-help
```

#### `--rewget-completions=SHELL`

Generate shell completions.

| Shell | Value |
|-------|-------|
| Bash | `bash` |
| Zsh | `zsh` |
| Fish | `fish` |
| PowerShell | `powershell` |

```bash
rewget --rewget-completions=bash
```

## File Locations

| File | Location | Description |
|------|----------|-------------|
| Config file | `~/.config/rewget/config.toml` | Optional persistent settings |
| Stage cache | `~/.cache/rewget/stage-cache.json` | Domain → stage mapping |
| Profiles | `~/.local/share/rewget/profiles/` | Browser profile definitions |
| Chromium | `~/.local/share/rewget/chromium/` | Chrome for Testing installation |

## Config File

rewget reads an optional TOML config file from `~/.config/rewget/config.toml`. All sections and keys are optional. Missing values fall back to the built-in defaults shown below.

```toml
[fallback]
# Master switch. If false, rewget never escalates (acts like --rewget-no-fallback).
enabled = true
# HTTP status codes that trigger fallback.
codes = [403, 429, 503, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529]
# Scan response bodies for known challenge markers.
body_detection = true

[daemon]
# How long rewgetd stays alive after the last request (seconds).
idle_timeout = 300
# Number of pre-warmed Chromium contexts in the pool.
browser_pool_size = 2

[profiles]
# Default profile name when --rewget-profile is not given.
default = "chrome"
# Auto-update profiles on first run of the day.
auto_update = false
```

## Configuration Precedence

1. Command-line flags (highest priority)
2. Environment variables (`RWGET_ENGINE`)
3. `~/.config/rewget/config.toml`
4. Default values (lowest priority)
