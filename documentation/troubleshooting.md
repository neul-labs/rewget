# Troubleshooting

Solutions to common issues with rewget.

## Installation Issues

### "command not found: rewget"

The rewget binary is not in your PATH.

**Solution:**

=== "Homebrew"

    ```bash
    brew link rewget
    ```

=== "Manual Install"

    Add the installation directory to your PATH:

    ```bash
    export PATH="$PATH:$HOME/.local/bin"
    ```

    Add this line to your `.bashrc` or `.zshrc`.

### "wget not found"

rewget requires wget to be installed.

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
rewget --no-check-certificate https://example.com/

# Or update CA certificates
sudo apt update && sudo apt install ca-certificates
```

### Timeout Errors

**Solution:**

Increase timeouts:

```bash
rewget --rewget-timeout-stage1=60000 \
      --rewget-timeout-stage2=60000 \
      --rewget-timeout-stage3=120000 \
      https://slow-site.com/
```

## Fallback Issues

### "403 Forbidden" Despite Fallback

Some sites have very aggressive bot detection that even Stage 3 can't bypass.

**Solutions:**

1. Try a different profile:
   ```bash
   rewget --rewget-profile=firefox_136 https://example.com/
   ```

2. Force Stage 3 with wait:
   ```bash
   rewget --rewget-js --rewget-js-wait=delay:10000 https://example.com/
   ```

3. Check if the site actually allows access (some block all automated access)

### "Stage 2 failed, trying Stage 3..."

This is normal behavior - Stage 2 (impersonation) didn't work, so rewget tries Stage 3 (browser).

If Stage 3 also fails, the site may have additional protections.

### Fallback Not Triggering

rewget only retries on specific status codes. Check your fallback codes:

```bash
# See what codes trigger fallback (default: 403,429,503,520-529)
rewget --rewget-debug https://example.com/

# Add additional codes
rewget --rewget-fallback-codes=403,429,503,500 https://example.com/
```

### Cache Issues

If a site starts blocking after previously working:

```bash
# Clear the domain cache
rewget --rewget-clear-cache

# Disable caching for this request
rewget --rewget-no-cache https://example.com/
```

## Daemon Issues

### "Failed to connect to daemon"

The daemon isn't running or crashed.

**Solutions:**

```bash
# Check if daemon is running
pgrep rewgetd

# Restart by killing and letting rewget respawn
pkill rewgetd
rewget https://example.com/

# Check daemon logs
rewget --rewget-debug https://example.com/
```

### "Daemon startup timeout"

The daemon is taking too long to start.

**Solutions:**

```bash
# Kill any stale daemon
pkill -9 rewgetd

# Remove socket file
rm ~/.cache/rewget/rewgetd.sock

# Try again
rewget https://example.com/
```

### High Memory Usage

The daemon caches browser sessions which can use memory.

**Solution:**

```bash
# Kill daemon to free memory
pkill rewgetd

# It will restart automatically when needed
```

## Chromium Issues

### "Chromium download failed"

**Solutions:**

1. Check internet connectivity
2. Ensure ~150MB disk space is available
3. Try manual download:
   ```bash
   rewget --rewget-download-chromium
   ```

### "Chromium not found" After Download

**Solution:**

```bash
# Check installation path
rewget --rewget-chromium-path

# Re-download if needed
rm -rf ~/.local/share/rewget/chromium/
rewget --rewget-download-chromium
```

### Stage 3 Hangs

The browser might be stuck on a page.

**Solutions:**

```bash
# Use a specific wait condition
rewget --rewget-js --rewget-js-wait=delay:5000 https://example.com/

# Reduce timeout
rewget --rewget-js --rewget-timeout-stage3=15000 https://example.com/
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
rewget --rewget-list-profiles

# Use exact profile name
rewget --rewget-profile=chrome_131 https://example.com/
```

### "Profile update failed"

**Solutions:**

```bash
# Check network connectivity
curl -v https://rewget.dev/profiles/v1/index.json

# Reset to built-in defaults
rewget --rewget-update-profiles
# (Falls back to defaults on failure)
```

### "Signature verification failed"

The profile source may be compromised or the file is corrupted.

**Solutions:**

1. Try updating again:
   ```bash
   rewget --rewget-update-profiles
   ```

2. If using custom URL, verify the source is trustworthy

3. Skip verification (only if you trust the source):
   ```bash
   rewget --rewget-no-verify --rewget-update-profiles
   ```

## Debug Mode

When reporting issues, include debug output:

```bash
rewget --rewget-debug https://example.com/ 2>&1 | tee rewget-debug.log
```

This shows:

- Which stage is being attempted
- HTTP status codes received
- Profile being used
- Timing information
- Error details

## Getting Help

If you can't resolve an issue:

1. Check existing [GitHub Issues](https://github.com/dipankardas011/rewget/issues)

2. Create a new issue with:
   - rewget version (`rewget --rewget-version`)
   - Operating system
   - Debug output (`--rewget-debug`)
   - Steps to reproduce

3. For security issues, email security@rewget.dev instead of creating a public issue.
