# How It Works

This page explains the technical details of how rwget bypasses bot detection.

## The Bot Detection Problem

Modern websites use multiple techniques to detect and block automated tools:

### TLS Fingerprinting

Every TLS client has a unique "fingerprint" based on:

- Supported cipher suites and their order
- TLS extensions and their order
- Supported elliptic curves
- Signature algorithms

wget's TLS fingerprint is distinctly different from browsers, making it easy to detect.

### HTTP/2 Fingerprinting

HTTP/2 connections reveal:

- SETTINGS frame values (window size, max streams, etc.)
- Pseudo-header order (`:method`, `:path`, etc.)
- Header compression behavior

wget2 uses different HTTP/2 settings than browsers.

### JavaScript Challenges

Some sites require JavaScript execution to:

- Solve Cloudflare's "checking your browser" challenge
- Generate required cookies or tokens
- Complete CAPTCHAs

wget can't execute JavaScript.

## rwget's Three-Stage Solution

### Stage 1: Plain wget

```
┌─────────────────┐     ┌─────────────────┐
│     rwget       │────▶│      wget       │────▶ Server
│  (passthrough)  │     │                 │
└─────────────────┘     └─────────────────┘
```

rwget first tries plain wget. This is the fastest option and works for most sites.

**Detection**: Checks exit code and response body for:
- HTTP 403, 429, 503, 520-529
- Cloudflare challenge pages
- Known bot detection signatures

### Stage 2: TLS Impersonation

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     rwget       │────▶│     rwgetd      │────▶│     rquest      │────▶ Server
│                 │ IPC │    (daemon)     │     │  (impersonate)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

rwget sends the request to the daemon, which uses `rquest` with browser emulation:

1. **TLS Handshake**: Mimics browser's exact cipher suite order, extensions, curves
2. **HTTP/2 Connection**: Uses browser's SETTINGS values and header order
3. **Headers**: Sends browser-appropriate Accept, Accept-Language, etc.

The server sees what looks like a real browser.

### Stage 3: JavaScript Preflight

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     rwget       │────▶│     rwgetd      │────▶│    Chromium     │────▶ Server
│                 │ IPC │    (daemon)     │     │   (headless)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
                                                 ┌─────────────────┐
                                                 │    Cookies +    │
                                                 │    Response     │
                                                 └─────────────────┘
```

For sites requiring JavaScript:

1. Headless Chromium navigates to the URL
2. Waits for challenges to complete (networkidle, selector, or delay)
3. Extracts cookies from the browser session
4. Returns response body or uses cookies for wget retry

## Component Architecture

### rwget (CLI)

The main binary that users interact with:

- Parses `--rwget-*` flags
- Passes remaining flags to wget
- Manages fallback logic
- Communicates with daemon via IPC

### rwgetd (Daemon)

Background process that handles Stage 2/3:

- Spawned automatically on first fallback
- Manages TLS impersonation via rquest
- Controls headless Chromium for JS preflight
- Handles multiple concurrent requests

### Communication

rwget and rwgetd communicate via nng (nanomsg-next-gen):

```json
// Request
{
  "id": "uuid",
  "command": "stage2",
  "url": "https://example.com/",
  "profile": "chrome_131",
  "timeout": 15000
}

// Response
{
  "id": "uuid",
  "success": true,
  "status": 200,
  "headers": {...},
  "body": "base64-encoded"
}
```

## Domain Stage Caching

To avoid repeating failed stages, rwget caches successful stages per domain:

```json
// ~/.cache/rwget/stage-cache.json
{
  "protected.example.com": {
    "stage": 2,
    "profile": "chrome_131",
    "expires": 1704672000
  },
  "js-heavy.example.com": {
    "stage": 3,
    "expires": 1704672000
  }
}
```

Cache entries expire after 7 days.

## TLS Fingerprint Details

### Chrome 131 Fingerprint

```
Cipher Suites (in order):
  TLS_AES_128_GCM_SHA256
  TLS_AES_256_GCM_SHA384
  TLS_CHACHA20_POLY1305_SHA256
  ECDHE-ECDSA-AES128-GCM-SHA256
  ECDHE-RSA-AES128-GCM-SHA256
  ...

Extensions:
  server_name (0)
  extended_master_secret (23)
  session_ticket (35)
  signature_algorithms (13)
  supported_versions (43)
  psk_key_exchange_modes (45)
  key_share (51)
  ...

Curves:
  x25519
  secp256r1
  secp384r1

GREASE: Enabled (random values in cipher suites and extensions)
```

### How Impersonation Works

1. **rquest** is a Rust HTTP client with impersonation support
2. It uses BoringSSL configured to match browser fingerprints
3. HTTP/2 frames are sent with browser-specific settings
4. The result is indistinguishable from a real browser at the network level

## JavaScript Execution Details

### Chromium Management

- Uses "Chrome for Testing" (official Google builds)
- Downloaded on first Stage 3 use (~150MB)
- Stored in `~/.local/share/rwget/chromium/`
- Runs in headless mode with anti-detection flags

### Challenge Resolution

1. Browser navigates to URL
2. Cloudflare/etc. JavaScript executes
3. Challenge cookies are set
4. Page loads or redirects
5. rwget extracts final response

### Wait Conditions

- **networkidle**: Waits for no network requests for 500ms
- **selector:CSS**: Waits for element to appear in DOM
- **delay:MS**: Waits fixed time (simple but reliable)

## Performance Characteristics

| Stage | Latency | Success Rate |
|-------|---------|--------------|
| Stage 1 | ~0ms overhead | 70-80% of sites |
| Stage 2 | ~100ms first request | 95%+ of TLS-fingerprinting sites |
| Stage 3 | 2-10s | JS challenge sites |

Most downloads complete at Stage 1 with zero overhead. The fallback chain only activates when needed.

## Security Considerations

### What rwget Does

- Mimics browser network behavior
- Executes JavaScript in isolated browser
- Respects robots.txt (wget behavior)

### What rwget Doesn't Do

- Bypass CAPTCHAs requiring human interaction
- Circumvent legal access controls
- Violate terms of service (user responsibility)

### Profile Verification

Remote profile updates are signed with Ed25519. The public key is embedded in rwget, preventing malicious profile injection.
