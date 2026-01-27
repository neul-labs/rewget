# FAQ

Frequently asked questions about rewget.

## General

### What is rewget?

rewget is a wget-compatible wrapper that automatically retries blocked requests with browser emulation. It's a drop-in replacement for wget that handles bot detection automatically.

### Is rewget a replacement for wget?

rewget wraps wget - it doesn't replace it. All wget functionality remains available. rewget adds automatic fallback for when wget gets blocked.

### Do I need to change my existing wget scripts?

No. rewget accepts all wget options unchanged. Your existing scripts will work with `rewget` instead of `wget`. The only difference is automatic fallback when blocked.

### How is this different from curl?

Both wget and curl are blocked by the same bot detection systems. rewget specifically handles these blocks by falling back to browser emulation techniques. You could use rewget with wget and still get blocked by curl on the same site.

### Is rewget legal to use?

rewget is a legitimate tool for accessing web content. However, you should:

- Respect websites' Terms of Service
- Honor robots.txt (wget does this by default)
- Not use rewget for unauthorized access
- Comply with local laws regarding web scraping

## Technical

### How does rewget bypass bot detection?

rewget uses three stages:

1. **Plain wget** - Fast, works for most sites
2. **TLS impersonation** - Mimics browser TLS/HTTP2 fingerprints
3. **JavaScript execution** - Real browser for JS challenges

Most bot detection uses TLS fingerprinting, which Stage 2 bypasses.

### Does rewget solve CAPTCHAs?

No. rewget cannot solve:

- reCAPTCHA
- hCaptcha
- Visual CAPTCHAs

It can solve:

- Cloudflare "checking your browser" challenges
- JavaScript-only verification
- Simple cookie-based challenges

### Why does Stage 3 need Chromium?

Stage 3 runs a real browser to execute JavaScript challenges. Many bot detection systems require JavaScript execution to generate tokens or cookies. There's no way to bypass this without an actual browser.

### How big is the Chromium download?

Approximately 150MB. It's downloaded once on first Stage 3 use and stored locally.

### Can I use my own browser instead of Chromium?

Currently, no. rewget uses Chrome for Testing, which is a version specifically designed for automation.

### Does rewget work with wget2?

Yes. Use `--rewget-engine=wget2` to use wget2 instead of wget.

### What's the overhead of using rewget?

- **Stage 1**: Zero overhead (passes through to wget)
- **Stage 2**: ~100ms on first request (daemon startup), negligible after
- **Stage 3**: 2-10 seconds (browser startup + page load)

Most requests complete at Stage 1 with no overhead.

## Usage

### Why am I still getting blocked?

Some sites have very aggressive protection that even browsers struggle with:

1. Try a different profile: `--rewget-profile=firefox_136`
2. Try Stage 3 with longer wait: `--rewget-js --rewget-js-wait=delay:10000`
3. Some sites may block all automated access

### How do I know which stage succeeded?

Use debug mode:

```bash
rewget --rewget-debug https://example.com/
```

Output shows which stage succeeded.

### Can I make rewget always use Stage 2/3?

Yes:

```bash
# Always start at Stage 2
rewget --rewget-fallback-stage=2 https://example.com/

# Always use Stage 3
rewget --rewget-js https://example.com/
```

### How do I disable fallback?

```bash
rewget --rewget-no-fallback https://example.com/
```

This makes rewget behave exactly like wget.

### Why does rewget remember what worked for a domain?

The domain cache speeds up subsequent requests. If Stage 2 worked for example.com once, future requests skip Stage 1. The cache expires after 7 days.

Clear it with `--rewget-clear-cache` if needed.

### Can I use rewget in shell scripts?

Yes. rewget is designed to be scripting-friendly:

```bash
#!/bin/bash
# Download all files from protected site
for url in $(cat urls.txt); do
    rewget --rewget-quiet "$url"
done
```

Use `--rewget-no-fallback` if you need predictable timing.

## Profiles

### What are browser profiles?

Profiles contain fingerprint data (TLS settings, HTTP/2 parameters, headers) that make rewget look like a specific browser version.

### How often are profiles updated?

Browser fingerprints change with each browser version. Run `--rewget-update-profiles` periodically to get the latest profiles.

### Can I create my own profiles?

Yes. Create a JSON file in `~/.local/share/rewget/profiles/` following the profile format. See the [Profiles](profiles.md) page for details.

### Why are profiles signed?

To prevent malicious profile injection. The signature ensures profiles come from the official rewget distribution. You can skip verification with `--rewget-no-verify`, but only do this with trusted sources.

## Performance

### Is rewget slower than wget?

For sites that don't block wget: No difference (Stage 1 passes through).

For blocked sites: Yes, because fallback takes time. But without rewget, the download would fail entirely.

### How can I speed up Stage 3?

1. Use a specific wait condition instead of networkidle:
   ```bash
   rewget --rewget-js --rewget-js-wait=selector:#content https://example.com/
   ```

2. Reduce timeout if the page loads fast:
   ```bash
   rewget --rewget-js --rewget-timeout-stage3=10000 https://example.com/
   ```

3. Pre-download Chromium so first use doesn't have to:
   ```bash
   rewget --rewget-download-chromium
   ```

### Does the daemon stay running?

The daemon stays running to handle subsequent requests efficiently. It will eventually shut down after an idle timeout.

## Security

### Is rewget safe to use?

Yes. rewget:

- Doesn't collect or transmit personal data
- Uses official Google Chrome builds
- Verifies profile signatures
- Runs browser in sandboxed mode

### Does rewget send my data anywhere?

No. All network requests go directly to the target server. rewget doesn't have telemetry or analytics.

### Can websites detect rewget?

Stage 2 is very difficult to detect (matches real browser fingerprints). Stage 3 runs a real browser so it's also hard to detect, though some advanced systems may detect headless mode.

### Is it safe to use --rewget-no-verify?

Only if you trust the profile source. This disables signature verification, which could allow malicious profiles to be installed.

## Platform Support

### Does rewget work on Windows?

Yes. rewget supports Windows x64. Some features use PowerShell for Chromium download.

### Does rewget work on macOS?

Yes. rewget supports both Intel (x86_64) and Apple Silicon (arm64) Macs.

### Does rewget work on ARM Linux?

Yes. rewget supports aarch64 (ARM64) Linux.

### Does rewget work in Docker?

Yes, but you'll need to install Chromium dependencies for Stage 3. See [Troubleshooting](troubleshooting.md) for required packages.
