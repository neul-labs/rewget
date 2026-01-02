# Quick Start

Get up and running with rwget in 5 minutes.

## Basic Download

rwget works exactly like wget:

```bash
# Download a file
rwget https://example.com/file.tar.gz

# Download with output filename
rwget -O myfile.tar.gz https://example.com/file.tar.gz

# Download to specific directory
rwget -P ./downloads/ https://example.com/file.tar.gz
```

## Automatic Fallback in Action

When a site blocks your request, rwget automatically retries:

```bash
$ rwget https://protected-site.com/data.json

[rwget] 403 Forbidden - retrying with impersonation...
[rwget] Success at Stage 2 (chrome_131)
```

The fallback happens automatically - no configuration needed.

## Common Scenarios

### Download from Protected Sites

Most protected sites work automatically:

```bash
rwget https://cloudflare-protected.com/file.zip
```

### Force Browser Mode

Skip straight to JavaScript preflight for heavily protected sites:

```bash
rwget --rwget-js https://heavily-protected.com/
```

### Strict Mode (No Fallback)

For scripts where you want predictable behavior:

```bash
rwget --rwget-no-fallback https://example.com/file.tar.gz
```

This exits with wget's original error code if blocked.

### Quiet Mode

Suppress rwget status messages:

```bash
rwget --rwget-quiet https://example.com/file.tar.gz
```

### Debug Mode

See detailed information about what's happening:

```bash
rwget --rwget-debug https://example.com/file.tar.gz
```

## Using wget Options

All wget options work unchanged:

```bash
# Resume interrupted download
rwget -c https://example.com/large-file.iso

# Limit download speed
rwget --limit-rate=1M https://example.com/file.tar.gz

# Set user agent (Stage 1 only)
rwget --user-agent="MyApp/1.0" https://example.com/

# Follow redirects
rwget --max-redirect=5 https://example.com/

# Download recursively
rwget -r -l 2 https://example.com/docs/
```

## Check Available Profiles

See what browser profiles are available:

```bash
$ rwget --rwget-list-profiles

Available browser profiles:

  chrome_131 - Chrome 131 on Windows
    Browser: Chrome 131.0.0.0
    Platform: Windows
    TLS: 17 cipher suites, GREASE: yes
    HTTP/2: 6 settings

  firefox_136 - Firefox 136 on Windows
    Browser: Firefox 136.0
    ...
```

## Use a Specific Profile

Request impersonation with a specific browser:

```bash
rwget --rwget-profile=firefox_136 https://example.com/
```

## View Cache Status

See which domains have cached stages:

```bash
# The cache is stored at ~/.cache/rwget/stage-cache.json
cat ~/.cache/rwget/stage-cache.json
```

Clear the cache:

```bash
rwget --rwget-clear-cache
```

## Summary of Key Flags

| Flag | Description |
|------|-------------|
| `--rwget-no-fallback` | Disable fallback, behave like wget |
| `--rwget-quiet` | Suppress rwget messages |
| `--rwget-debug` | Enable debug output |
| `--rwget-js` | Force JavaScript preflight |
| `--rwget-profile=NAME` | Use specific browser profile |
| `--rwget-list-profiles` | List available profiles |
| `--rwget-clear-cache` | Clear domain stage cache |

## Next Steps

- [Usage Guide](usage.md) - Detailed usage information
- [Configuration](configuration.md) - All configuration options
- [How It Works](how-it-works.md) - Technical details
