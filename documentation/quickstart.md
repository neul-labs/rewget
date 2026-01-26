# Quick Start

Get up and running with rewget in 5 minutes.

## Basic Download

rewget works exactly like wget:

```bash
# Download a file
rewget https://example.com/file.tar.gz

# Download with output filename
rewget -O myfile.tar.gz https://example.com/file.tar.gz

# Download to specific directory
rewget -P ./downloads/ https://example.com/file.tar.gz
```

## Automatic Fallback in Action

When a site blocks your request, rewget automatically retries:

```bash
$ rewget https://protected-site.com/data.json

[rewget] 403 Forbidden - retrying with impersonation...
[rewget] Success at Stage 2 (chrome_131)
```

The fallback happens automatically - no configuration needed.

## Common Scenarios

### Download from Protected Sites

Most protected sites work automatically:

```bash
rewget https://cloudflare-protected.com/file.zip
```

### Force Browser Mode

Skip straight to JavaScript preflight for heavily protected sites:

```bash
rewget --rewget-js https://heavily-protected.com/
```

### Strict Mode (No Fallback)

For scripts where you want predictable behavior:

```bash
rewget --rewget-no-fallback https://example.com/file.tar.gz
```

This exits with wget's original error code if blocked.

### Quiet Mode

Suppress rewget status messages:

```bash
rewget --rewget-quiet https://example.com/file.tar.gz
```

### Debug Mode

See detailed information about what's happening:

```bash
rewget --rewget-debug https://example.com/file.tar.gz
```

## Using wget Options

All wget options work unchanged:

```bash
# Resume interrupted download
rewget -c https://example.com/large-file.iso

# Limit download speed
rewget --limit-rate=1M https://example.com/file.tar.gz

# Set user agent (Stage 1 only)
rewget --user-agent="MyApp/1.0" https://example.com/

# Follow redirects
rewget --max-redirect=5 https://example.com/

# Download recursively
rewget -r -l 2 https://example.com/docs/
```

## Check Available Profiles

See what browser profiles are available:

```bash
$ rewget --rewget-list-profiles

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
rewget --rewget-profile=firefox_136 https://example.com/
```

## View Cache Status

See which domains have cached stages:

```bash
# The cache is stored at ~/.cache/rewget/stage-cache.json
cat ~/.cache/rewget/stage-cache.json
```

Clear the cache:

```bash
rewget --rewget-clear-cache
```

## Summary of Key Flags

| Flag | Description |
|------|-------------|
| `--rewget-no-fallback` | Disable fallback, behave like wget |
| `--rewget-quiet` | Suppress rewget messages |
| `--rewget-debug` | Enable debug output |
| `--rewget-js` | Force JavaScript preflight |
| `--rewget-profile=NAME` | Use specific browser profile |
| `--rewget-list-profiles` | List available profiles |
| `--rewget-clear-cache` | Clear domain stage cache |

## Next Steps

- [Usage Guide](usage.md) - Detailed usage information
- [Configuration](configuration.md) - All configuration options
- [How It Works](how-it-works.md) - Technical details
