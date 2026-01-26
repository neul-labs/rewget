# Impersonation Layer

The impersonation layer (Stage 2) mimics browser TLS and HTTP/2 fingerprints without running a full browser. This document specifies the fingerprinting techniques and profile format.

## Why fingerprinting matters

When an HTTP client connects to a TLS-enabled server, it reveals identifying information through:

1. **TLS Client Hello** - Cipher suites, extensions, curves, and their ordering
2. **HTTP/2 SETTINGS frame** - Connection parameters like window size
3. **HTTP headers** - Order, casing, and values

Bot detection systems (Cloudflare, Akamai, PerimeterX) compare these fingerprints against known browser profiles. Mismatches trigger challenges or blocks.

## TLS Fingerprinting

### JA3 (Legacy)

JA3 concatenates these Client Hello fields, then MD5 hashes the result:

```
SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurveFormats
```

Example JA3 string (before hashing):
```
771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24,0
```

**Limitation**: Chrome now randomizes extension order, making JA3 unreliable for Chrome detection.

### JA4 (Current Standard)

JA4 addresses JA3's limitations by:
- Sorting extensions alphabetically before hashing
- Including ALPN values (h2, http/1.1)
- Distinguishing TCP vs QUIC
- Providing human-readable prefix

Format: `{protocol}{version}{SNI}{ciphers}_{extensions}_{signature_algos}`

Example: `t13d1516h2_8daaf6152771_d8a2da3f94cd`

Breakdown:
- `t` = TCP (vs `q` for QUIC)
- `13` = TLS 1.3
- `d` = domain SNI present
- `15` = 15 ciphers
- `16` = 16 extensions
- `h2` = ALPN includes HTTP/2

### Key TLS Fields

| Field | Chrome 120 | Firefox 121 |
|-------|------------|-------------|
| TLS Version | 1.3 | 1.3 |
| Cipher count | 16 | 17 |
| Extension count | 18 | 15 |
| GREASE | Yes | No |
| Post-quantum | Yes (X25519Kyber768) | No |
| ECH | Yes | Yes |

### GREASE (Generate Random Extensions And Sustain Extensibility)

Chrome injects random values (0x0a0a, 0x1a1a, etc.) into cipher suites and extensions to test server compliance. These must be included for accurate Chrome impersonation.

```
GREASE values: 0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a,
               0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a,
               0xaaaa, 0xbaba, 0xcaca, 0xdada, 0xeaea, 0xfafa
```

## HTTP/2 Fingerprinting

### SETTINGS Frame

The first HTTP/2 frame reveals client identity through default values:

| Setting ID | Name | Chrome | Firefox | Go stdlib |
|------------|------|--------|---------|-----------|
| 1 | HEADER_TABLE_SIZE | 65536 | 65536 | 4096 |
| 2 | ENABLE_PUSH | 0 | 0 | 0 |
| 3 | MAX_CONCURRENT_STREAMS | 1000 | 100 | - |
| 4 | INITIAL_WINDOW_SIZE | 6291456 | 131072 | 65535 |
| 5 | MAX_FRAME_SIZE | 16384 | 16384 | 16384 |
| 6 | MAX_HEADER_LIST_SIZE | 262144 | 65536 | - |

### WINDOW_UPDATE Frame

Sent after SETTINGS to increase flow control window:

| Client | WINDOW_UPDATE value |
|--------|---------------------|
| Chrome | 15663105 |
| Firefox | 12517377 |
| Safari | 10485760 |

### Pseudo-Header Order

HTTP/2 pseudo-headers have implementation-specific ordering:

| Client | Order |
|--------|-------|
| Chrome | `:method`, `:authority`, `:scheme`, `:path` |
| Firefox | `:method`, `:path`, `:authority`, `:scheme` |
| Safari | `:method`, `:scheme`, `:path`, `:authority` |

### PRIORITY Frames

Firefox sends PRIORITY frames for unopened streams. Chrome deprecated PRIORITY in favor of PRIORITY_UPDATE (RFC 9218).

### Akamai Fingerprint Format

```
SETTINGS|WINDOW_UPDATE|PRIORITY|PSEUDO_HEADER_ORDER
```

Example Chrome fingerprint:
```
1:65536;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p
```

Example Firefox fingerprint:
```
1:65536;3:100;4:131072;5:16384|12517377|3:0:0:201,5:0:0:101|m,p,a,s
```

## Header Fingerprinting

### Header Order

Browsers send headers in specific orders. Python's `requests` library alphabetizes headers—an immediate red flag.

Chrome typical order:
```
:method: GET
:authority: example.com
:scheme: https
:path: /
sec-ch-ua: "..."
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "..."
upgrade-insecure-requests: 1
user-agent: Mozilla/5.0 ...
accept: text/html,...
sec-fetch-site: none
sec-fetch-mode: navigate
sec-fetch-user: ?1
sec-fetch-dest: document
accept-encoding: gzip, deflate, br
accept-language: en-US,en;q=0.9
```

### Client Hints

Chrome sends `sec-ch-ua-*` headers that must match the claimed browser version:

```
sec-ch-ua: "Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120"
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "macOS"
```

## Profile Format

Profiles are stored as JSON in `~/.config/rewget/profiles/`:

```json
{
  "name": "chrome_120",
  "browser": "chrome",
  "version": "120.0.6099.109",
  "os": "macos",

  "tls": {
    "version": "1.3",
    "ciphers": [
      "TLS_AES_128_GCM_SHA256",
      "TLS_AES_256_GCM_SHA384",
      "TLS_CHACHA20_POLY1305_SHA256",
      "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
      "..."
    ],
    "extensions": [
      "server_name",
      "extended_master_secret",
      "renegotiation_info",
      "supported_groups",
      "ec_point_formats",
      "session_ticket",
      "application_layer_protocol_negotiation",
      "status_request",
      "signature_algorithms",
      "signed_certificate_timestamp",
      "key_share",
      "psk_key_exchange_modes",
      "supported_versions",
      "compress_certificate",
      "encrypted_client_hello",
      "..."
    ],
    "curves": ["X25519", "P-256", "P-384"],
    "signature_algorithms": ["ecdsa_secp256r1_sha256", "rsa_pss_rsae_sha256", "..."],
    "grease": true,
    "post_quantum": true,
    "alpn": ["h2", "http/1.1"],
    "ja3": "771,4865-4866-4867-...",
    "ja4": "t13d1516h2_8daaf6152771_d8a2da3f94cd"
  },

  "http2": {
    "settings": {
      "HEADER_TABLE_SIZE": 65536,
      "ENABLE_PUSH": 0,
      "MAX_CONCURRENT_STREAMS": 1000,
      "INITIAL_WINDOW_SIZE": 6291456,
      "MAX_FRAME_SIZE": 16384,
      "MAX_HEADER_LIST_SIZE": 262144
    },
    "window_update": 15663105,
    "pseudo_header_order": ["method", "authority", "scheme", "path"],
    "priority_frames": false,
    "akamai_fingerprint": "1:65536;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p"
  },

  "headers": {
    "order": [
      "sec-ch-ua",
      "sec-ch-ua-mobile",
      "sec-ch-ua-platform",
      "upgrade-insecure-requests",
      "user-agent",
      "accept",
      "sec-fetch-site",
      "sec-fetch-mode",
      "sec-fetch-user",
      "sec-fetch-dest",
      "accept-encoding",
      "accept-language"
    ],
    "values": {
      "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
      "sec-ch-ua": "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"",
      "sec-ch-ua-mobile": "?0",
      "sec-ch-ua-platform": "\"macOS\"",
      "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
      "accept-encoding": "gzip, deflate, br",
      "accept-language": "en-US,en;q=0.9"
    }
  },

  "metadata": {
    "captured_at": "2024-01-15T12:00:00Z",
    "source": "browserleaks.com",
    "verified": true
  }
}
```

## Profile Update Mechanism

### Update Server

Profiles are distributed from a central server:

```
https://profiles.rewget.dev/v1/
├── index.json           # List of all available profiles
├── chrome/
│   ├── chrome_120.json
│   ├── chrome_121.json
│   └── chrome_122.json
├── firefox/
│   ├── firefox_121.json
│   └── firefox_122.json
└── signatures/
    └── index.json.sig   # Ed25519 signature
```

### Index Format

```json
{
  "version": 2,
  "updated_at": "2024-01-15T12:00:00Z",
  "profiles": [
    {
      "name": "chrome_120",
      "browser": "chrome",
      "version": "120.0.6099.109",
      "os": ["macos", "windows", "linux"],
      "sha256": "abc123...",
      "size": 4096
    }
  ],
  "recommended": {
    "chrome": "chrome_122",
    "firefox": "firefox_122",
    "safari": "safari_17"
  }
}
```

### Security

All profile updates are signed with Ed25519:

1. Server signs `index.json` with private key
2. Public key is embedded in rewget binary
3. Client verifies signature before applying updates
4. Individual profile files are verified via SHA-256 from signed index

```rust
// Embedded in rewget binary
const PROFILE_PUBLIC_KEY: &[u8] = b"...";
```

### Update Flow

```
rewget --rewget-update-profiles

1. Fetch https://profiles.rewget.dev/v1/index.json
2. Fetch https://profiles.rewget.dev/v1/signatures/index.json.sig
3. Verify Ed25519 signature against embedded public key
4. Compare local profiles against remote index
5. Download new/updated profiles
6. Verify SHA-256 of each downloaded file
7. Atomic rename into ~/.config/rewget/profiles/
```

### Offline Bundling

The rewget binary bundles the 3 most recent profiles for each major browser as a fallback when updates are unavailable.

## Testing Strategy

### Fingerprint Capture

Capture real browser fingerprints using:

1. **TLS**: Run a local TLS server that logs Client Hello
2. **HTTP/2**: Use nginx with fingerprinting module
3. **Online tools**: browserleaks.com, scrapfly.io

### Verification Tools

```bash
# Verify TLS fingerprint
rewget --rewget-debug --rewget-profile=chrome_120 https://scrapfly.io/web-scraping-tools/ja3-fingerprint

# Verify HTTP/2 fingerprint
rewget --rewget-debug --rewget-profile=chrome_120 https://browserleaks.com/http2

# Verify headers
rewget --rewget-debug --rewget-profile=chrome_120 https://httpbin.org/headers
```

### Automated Testing

CI pipeline runs against fingerprint verification services:

```yaml
fingerprint-tests:
  - name: "Chrome 120 JA3"
    profile: chrome_120
    url: https://check.ja3.zone/
    expect:
      ja3_match: true

  - name: "Chrome 120 HTTP/2"
    profile: chrome_120
    url: https://browserleaks.com/http2
    expect:
      akamai_fingerprint: "1:65536;3:1000;4:6291456;6:262144|15663105|0|m,a,s,p"
```

### Local Verification Server

rewget ships with a local verification tool:

```bash
rewget --rewget-verify-profile chrome_120

TLS Fingerprint:
  JA3: 771,4865-4866-4867-... ✓
  JA4: t13d1516h2_8daaf6152771_d8a2da3f94cd ✓

HTTP/2 Fingerprint:
  SETTINGS: 1:65536;3:1000;4:6291456;6:262144 ✓
  WINDOW_UPDATE: 15663105 ✓
  Pseudo-headers: m,a,s,p ✓

Headers:
  Order: ✓
  Client hints: ✓

Overall: PASS
```

### Regression Testing

When updating profiles:

1. Capture fingerprint from real browser (Selenium/Playwright)
2. Generate profile JSON
3. Run rewget with new profile against verification services
4. Compare fingerprints
5. Gate release on 100% match

### Known Limitations

Some fingerprint aspects cannot be replicated:

| Aspect | Replicable | Notes |
|--------|------------|-------|
| TLS cipher suites | ✓ | Requires patched TLS library |
| TLS extensions | ✓ | Ordering controllable |
| GREASE | ✓ | Random but valid values |
| HTTP/2 SETTINGS | ✓ | Configurable |
| HTTP/2 PRIORITY | ✓ | Can be emulated |
| Header order | ✓ | Manual ordering |
| Navigator JS APIs | ✗ | Requires real browser (Stage 3) |
| Canvas fingerprint | ✗ | Requires real browser (Stage 3) |
| WebGL fingerprint | ✗ | Requires real browser (Stage 3) |

Sites checking JS-based fingerprints will require Stage 3 (full browser).

## Implementation Notes

### Rust Crates

| Component | Crate | Notes |
|-----------|-------|-------|
| TLS | `rustls` | Fork with custom ClientConfig |
| HTTP/2 | `h2` | Fork for SETTINGS control |
| HTTP client | `hyper` | Uses custom h2 |
| Crypto | `ring` | BoringSSL-compatible |
| Signing | `ed25519-dalek` | Profile verification |

### rustls Modifications

Required changes to rustls for accurate fingerprinting:

1. Custom cipher suite ordering
2. Custom extension ordering
3. GREASE injection
4. Configurable signature algorithms
5. Post-quantum key exchange support

### h2 Modifications

Required changes to h2 crate:

1. Configurable SETTINGS frame values
2. Custom WINDOW_UPDATE values
3. PRIORITY frame emission
4. Pseudo-header ordering control

## References

- [JA3 - Salesforce](https://github.com/salesforce/ja3)
- [JA4+ - FoxIO](https://github.com/FoxIO-LLC/ja4)
- [Akamai HTTP/2 Fingerprinting Whitepaper](https://blackhat.com/docs/eu-17/materials/eu-17-Shuster-Passive-Fingerprinting-Of-HTTP2-Clients-wp.pdf)
- [BrowserLeaks HTTP/2](https://browserleaks.com/http2)
- [curl-impersonate](https://github.com/lwthiker/curl-impersonate)
- [Scrapfly JA3 Tool](https://scrapfly.io/web-scraping-tools/ja3-fingerprint)
