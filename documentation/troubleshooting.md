# Troubleshooting

Solutions to common issues with rwget.

## Installation Issues

### "command not found: rwget"

The rwget binary is not in your PATH.

**Solution:**

=== "Homebrew"

    ```bash
    brew link rwget
    ```

=== "Manual Install"

    Add the installation directory to your PATH:

    ```bash
    export PATH="$PATH:$HOME/.local/bin"
    ```

    Add this line to your `.bashrc` or `.zshrc`.

### "wget not found"

rwget requires wget to be installed.

**Solution:**

=== "Debian/Ubuntu"

    ```bash
    sudo apt install wget
    ```

=== "Fedora"

    ```bash
    sudo dnf install wget
    ```

=== "macOS"

    ```bash
    brew install wget
    ```

### Build Errors

If building from source fails:

```bash
# Ensure Rust is up to date
rustup update

# Install build dependencies (Ubuntu/Debian)
sudo apt install build-essential cmake pkg-config libssl-dev

# Try clean build
cargo clean
cargo build --release
```

## Connection Issues

### "Connection refused" or "Network error"

**Possible causes:**

1. Network connectivity issues
2. Firewall blocking connections
3. VPN/proxy interference

**Solutions:**

```bash
# Test basic connectivity
curl -v https://example.com/

# Try without proxy
unset http_proxy https_proxy

# Check firewall
sudo iptables -L
```

### "SSL certificate problem"

**Solution:**

```bash
# Skip certificate verification (not recommended for production)
rwget --no-check-certificate https://example.com/

# Or update CA certificates
sudo apt update && sudo apt install ca-certificates
```

### Timeout Errors

**Solution:**

Increase timeouts:

```bash
rwget --rwget-timeout-stage1=60000 \
      --rwget-timeout-stage2=60000 \
      --rwget-timeout-stage3=120000 \
      https://slow-site.com/
```

## Fallback Issues

### "403 Forbidden" Despite Fallback

Some sites have very aggressive bot detection that even Stage 3 can't bypass.

**Solutions:**

1. Try a different profile:
   ```bash
   rwget --rwget-profile=firefox_136 https://example.com/
   ```

2. Force Stage 3 with wait:
   ```bash
   rwget --rwget-js --rwget-js-wait=delay:10000 https://example.com/
   ```

3. Check if the site actually allows access (some block all automated access)

### "Stage 2 failed, trying Stage 3..."

This is normal behavior - Stage 2 (impersonation) didn't work, so rwget tries Stage 3 (browser).

If Stage 3 also fails, the site may have additional protections.

### Fallback Not Triggering

rwget only retries on specific status codes. Check your fallback codes:

```bash
# See what codes trigger fallback (default: 403,429,503,520-529)
rwget --rwget-debug https://example.com/

# Add additional codes
rwget --rwget-fallback-codes=403,429,503,500 https://example.com/
```

### Cache Issues

If a site starts blocking after previously working:

```bash
# Clear the domain cache
rwget --rwget-clear-cache

# Disable caching for this request
rwget --rwget-no-cache https://example.com/
```

## Daemon Issues

### "Failed to connect to daemon"

The daemon isn't running or crashed.

**Solutions:**

```bash
# Check if daemon is running
pgrep rwgetd

# Restart by killing and letting rwget respawn
pkill rwgetd
rwget https://example.com/

# Check daemon logs
rwget --rwget-debug https://example.com/
```

### "Daemon startup timeout"

The daemon is taking too long to start.

**Solutions:**

```bash
# Kill any stale daemon
pkill -9 rwgetd

# Remove socket file
rm ~/.cache/rwget/rwgetd.sock

# Try again
rwget https://example.com/
```

### High Memory Usage

The daemon caches browser sessions which can use memory.

**Solution:**

```bash
# Kill daemon to free memory
pkill rwgetd

# It will restart automatically when needed
```

## Chromium Issues

### "Chromium download failed"

**Solutions:**

1. Check internet connectivity
2. Ensure ~150MB disk space is available
3. Try manual download:
   ```bash
   rwget --rwget-download-chromium
   ```

### "Chromium not found" After Download

**Solution:**

```bash
# Check installation path
rwget --rwget-chromium-path

# Re-download if needed
rm -rf ~/.local/share/rwget/chromium/
rwget --rwget-download-chromium
```

### Stage 3 Hangs

The browser might be stuck on a page.

**Solutions:**

```bash
# Use a specific wait condition
rwget --rwget-js --rwget-js-wait=delay:5000 https://example.com/

# Reduce timeout
rwget --rwget-js --rwget-timeout-stage3=15000 https://example.com/
```

### "Chrome failed to start"

**Possible causes:**

1. Missing system dependencies
2. Sandbox issues on Linux

**Solutions:**

=== "Ubuntu/Debian"

    ```bash
    # Install Chrome dependencies
    sudo apt install libnss3 libatk1.0-0 libatk-bridge2.0-0 \
                     libcups2 libdrm2 libxkbcommon0 libxcomposite1 \
                     libxdamage1 libxfixes3 libxrandr2 libgbm1 \
                     libasound2
    ```

=== "Sandbox Issues"

    If you see sandbox errors:
    ```bash
    # This is a workaround - not recommended for security
    export CHROMIUM_FLAGS="--no-sandbox"
    ```

## Profile Issues

### "Profile not found"

**Solution:**

```bash
# List available profiles
rwget --rwget-list-profiles

# Use exact profile name
rwget --rwget-profile=chrome_131 https://example.com/
```

### "Profile update failed"

**Solutions:**

```bash
# Check network connectivity
curl -v https://rwget.dev/profiles/v1/index.json

# Reset to built-in defaults
rwget --rwget-update-profiles
# (Falls back to defaults on failure)
```

### "Signature verification failed"

The profile source may be compromised or the file is corrupted.

**Solutions:**

1. Try updating again:
   ```bash
   rwget --rwget-update-profiles
   ```

2. If using custom URL, verify the source is trustworthy

3. Skip verification (only if you trust the source):
   ```bash
   rwget --rwget-no-verify --rwget-update-profiles
   ```

## Debug Mode

When reporting issues, include debug output:

```bash
rwget --rwget-debug https://example.com/ 2>&1 | tee rwget-debug.log
```

This shows:

- Which stage is being attempted
- HTTP status codes received
- Profile being used
- Timing information
- Error details

## Getting Help

If you can't resolve an issue:

1. Check existing [GitHub Issues](https://github.com/dipankardas011/rwget/issues)

2. Create a new issue with:
   - rwget version (`rwget --rwget-version`)
   - Operating system
   - Debug output (`--rwget-debug`)
   - Steps to reproduce

3. For security issues, email security@rwget.dev instead of creating a public issue.
