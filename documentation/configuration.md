# Configuration

rwget can be configured through command-line flags and environment variables.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RWGET_ENGINE` | Default wget engine (`wget` or `wget2`) | `wget` |

## Command-Line Flags

All rwget-specific flags use the `--rwget-*` prefix to avoid conflicts with wget flags.

### Core Options

#### `--rwget-no-fallback`

Disable fallback completely. rwget behaves exactly like wget.

```bash
rwget --rwget-no-fallback https://example.com/
```

#### `--rwget-engine=ENGINE`

Select the wget engine to use.

| Value | Description |
|-------|-------------|
| `wget` | GNU Wget (default) |
| `wget2` | GNU Wget2 |

```bash
rwget --rwget-engine=wget2 https://example.com/
```

#### `--rwget-quiet`

Suppress rwget status messages. wget's output is unchanged.

```bash
rwget --rwget-quiet https://example.com/
```

#### `--rwget-debug`

Enable verbose debug output for troubleshooting.

```bash
rwget --rwget-debug https://example.com/
```

### Fallback Control

#### `--rwget-fallback-codes=CODES`

Comma-separated HTTP status codes that trigger fallback.

**Default**: `403,429,503,520,521,522,523,524,525,526,527,528,529`

```bash
rwget --rwget-fallback-codes=403,429 https://example.com/
```

#### `--rwget-fallback-stage=N`

Start at a specific stage instead of Stage 1.

| Value | Stage |
|-------|-------|
| `1` | Plain wget (default) |
| `2` | TLS impersonation |
| `3` | JavaScript preflight |

```bash
rwget --rwget-fallback-stage=2 https://example.com/
```

#### `--rwget-no-body-detection`

Disable HTML body pattern detection for bot protection signatures.

```bash
rwget --rwget-no-body-detection https://example.com/
```

### Browser Profile Options

#### `--rwget-profile=NAME`

Use a specific browser profile for impersonation.

```bash
rwget --rwget-profile=chrome_131 https://example.com/
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

#### `--rwget-list-profiles`

List all available browser profiles.

```bash
rwget --rwget-list-profiles
```

#### `--rwget-update-profiles`

Update browser profiles from the remote server.

```bash
rwget --rwget-update-profiles
```

#### `--rwget-profile-url=URL`

Custom URL for profile updates.

```bash
rwget --rwget-profile-url=https://my-server.com/profiles.json --rwget-update-profiles
```

#### `--rwget-no-verify`

Skip Ed25519 signature verification when updating profiles.

!!! warning
    Only use this for testing or with trusted profile sources.

```bash
rwget --rwget-no-verify --rwget-update-profiles
```

#### `--rwget-verify-profile=NAME`

Display detailed fingerprint information for a profile.

```bash
rwget --rwget-verify-profile=chrome_131
```

### JavaScript Preflight Options

#### `--rwget-js`

Force JavaScript preflight (Stage 3) from the start.

```bash
rwget --rwget-js https://example.com/
```

#### `--rwget-js-wait=CONDITION`

Wait condition for JavaScript preflight.

| Condition | Description |
|-----------|-------------|
| `networkidle` | Wait for network to be idle |
| `selector:CSS` | Wait for CSS selector to match |
| `delay:MS` | Wait fixed milliseconds |

```bash
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/
rwget --rwget-js --rwget-js-wait=selector:#main-content https://example.com/
rwget --rwget-js --rwget-js-wait=delay:5000 https://example.com/
```

#### `--rwget-download-chromium`

Pre-download Chromium for JavaScript preflight.

```bash
rwget --rwget-download-chromium
```

#### `--rwget-chromium-path`

Print the Chromium installation path.

```bash
rwget --rwget-chromium-path
```

### Timeout Options

#### `--rwget-timeout-stage1=MS`

Stage 1 timeout in milliseconds. Uses wget's timeout settings by default.

```bash
rwget --rwget-timeout-stage1=30000 https://example.com/
```

#### `--rwget-timeout-stage2=MS`

Stage 2 timeout in milliseconds.

**Default**: `15000` (15 seconds)

```bash
rwget --rwget-timeout-stage2=30000 https://example.com/
```

#### `--rwget-timeout-stage3=MS`

Stage 3 timeout in milliseconds.

**Default**: `30000` (30 seconds)

```bash
rwget --rwget-timeout-stage3=60000 https://example.com/
```

### Cache Options

#### `--rwget-no-cache`

Disable domain stage caching. Always start at Stage 1.

```bash
rwget --rwget-no-cache https://example.com/
```

#### `--rwget-clear-cache`

Clear the domain stage cache and exit.

```bash
rwget --rwget-clear-cache
```

### Daemon Options

#### `--rwget-daemon=MODE`

Control daemon behavior.

| Mode | Description |
|------|-------------|
| `auto` | Start daemon when needed (default) |
| `on` | Always use daemon |
| `off` | Never use daemon (Stage 1 only) |

```bash
rwget --rwget-daemon=on https://example.com/
```

### Information Options

#### `--rwget-version`

Print rwget version and exit.

```bash
rwget --rwget-version
```

#### `--rwget-help`

Print help message and exit.

```bash
rwget --rwget-help
```

#### `--rwget-completions=SHELL`

Generate shell completions.

| Shell | Value |
|-------|-------|
| Bash | `bash` |
| Zsh | `zsh` |
| Fish | `fish` |
| PowerShell | `powershell` |

```bash
rwget --rwget-completions=bash
```

## File Locations

| File | Location | Description |
|------|----------|-------------|
| Stage cache | `~/.cache/rwget/stage-cache.json` | Domain → stage mapping |
| Profiles | `~/.local/share/rwget/profiles/` | Browser profile definitions |
| Chromium | `~/.local/share/rwget/chromium/` | Chrome for Testing installation |

## Configuration Precedence

1. Command-line flags (highest priority)
2. Environment variables
3. Default values (lowest priority)
