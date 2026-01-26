# Browser Profiles

Browser profiles are the key to Stage 2's effectiveness. They contain all the fingerprint data needed to impersonate a real browser.

## What's in a Profile?

Each profile contains:

- **TLS Configuration**: Cipher suites, extensions, curves, ALPN
- **HTTP/2 Settings**: SETTINGS frame values, window sizes, header order
- **HTTP Headers**: User-Agent, Accept, Accept-Language, etc.
- **Browser Metadata**: Name, version, platform

## Available Profiles

rewget ships with 6 built-in profiles:

| Profile | Browser | Platform | Best For |
|---------|---------|----------|----------|
| `chrome_131` | Chrome 131 | Windows | General use (default) |
| `chrome_130` | Chrome 130 | Windows | Fallback if 131 blocked |
| `firefox_136` | Firefox 136 | Windows | Sites that prefer Firefox |
| `firefox_133` | Firefox 133 | Windows | Older Firefox fingerprint |
| `safari_18` | Safari 18 | macOS | Apple-friendly sites |
| `edge_131` | Edge 131 | Windows | Microsoft Edge fingerprint |

## Using Profiles

### List Available Profiles

```bash
rewget --rewget-list-profiles
```

Output:
```
Available browser profiles:

  chrome_131 - Chrome 131 on Windows
    Browser: Chrome 131.0.0.0
    Platform: Windows
    TLS: 17 cipher suites, GREASE: yes
    HTTP/2: 6 settings

  firefox_136 - Firefox 136 on Windows
    Browser: Firefox 136.0
    Platform: Windows
    TLS: 17 cipher suites, GREASE: no
    HTTP/2: 6 settings

  ...

Total: 6 profiles
```

### Select a Specific Profile

```bash
rewget --rewget-profile=firefox_136 https://example.com/
```

### Verify Profile Details

```bash
rewget --rewget-verify-profile=chrome_131
```

Output:
```
Profile: chrome_131
Description: Chrome 131 on Windows
Version: 1

Browser:
  Name: Chrome
  Version: 131.0.0.0
  Platform: Windows
  User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36...

TLS Settings:
  Versions: TLSv1.2, TLSv1.3
  Cipher Suites: 17 total
    1. TLS_AES_128_GCM_SHA256
    2. TLS_AES_256_GCM_SHA384
    3. TLS_CHACHA20_POLY1305_SHA256
    ...
  Extensions: 16 total
  Curves: x25519, secp256r1, secp384r1
  ALPN: h2, http/1.1
  GREASE: enabled

HTTP/2 Settings:
  HEADER_TABLE_SIZE: 65536
  ENABLE_PUSH: 0
  MAX_CONCURRENT_STREAMS: 1000
  INITIAL_WINDOW_SIZE: 6291456
  MAX_FRAME_SIZE: 16384
  MAX_HEADER_LIST_SIZE: 262144
  Window Update: 15663105
  Pseudo-header Order: :method, :authority, :scheme, :path

Default Headers: 8
  Accept: text/html,application/xhtml+xml,...
  Accept-Language: en-US,en;q=0.9
  ...
```

## Updating Profiles

Browser fingerprints change with each browser version. Keep profiles current:

```bash
rewget --rewget-update-profiles
```

Output:
```
[rewget] Fetching profiles from: https://rewget.dev/profiles/v1/index.json
[rewget] Updated: chrome_132, firefox_137
[rewget] Added: chrome_133
[rewget] Unchanged: 4 profiles
[rewget] Total: 7 profiles
```

### Custom Profile Source

Use your own profile server:

```bash
rewget --rewget-profile-url=https://my-server.com/profiles.json --rewget-update-profiles
```

### Skip Verification

For testing only:

```bash
rewget --rewget-no-verify --rewget-update-profiles
```

## Profile Selection Logic

When no profile is specified, rewget selects automatically:

1. **Cached profile**: If the domain has a cached successful profile, use it
2. **Chrome latest**: Try the newest Chrome profile first
3. **Rotation**: On failure, try other profiles in order

## Profile File Format

Profiles are stored as JSON in `~/.local/share/rewget/profiles/`:

```json
{
  "name": "chrome_131",
  "description": "Chrome 131 on Windows",
  "version": 1,
  "browser": {
    "name": "Chrome",
    "version": "131.0.0.0",
    "platform": "Windows",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)..."
  },
  "tls": {
    "versions": ["TLSv1.2", "TLSv1.3"],
    "cipher_suites": [
      "TLS_AES_128_GCM_SHA256",
      "TLS_AES_256_GCM_SHA384",
      ...
    ],
    "extensions": [0, 23, 65281, ...],
    "curves": ["x25519", "secp256r1", "secp384r1"],
    "alpn": ["h2", "http/1.1"],
    "grease": true
  },
  "http2": {
    "settings": {
      "HEADER_TABLE_SIZE": 65536,
      "ENABLE_PUSH": 0,
      ...
    },
    "window_update": 15663105,
    "pseudo_header_order": [":method", ":authority", ":scheme", ":path"]
  },
  "headers": {
    "Accept": "text/html,application/xhtml+xml,...",
    "Accept-Language": "en-US,en;q=0.9",
    ...
  }
}
```

## Security

### Signature Verification

Profile updates are signed with Ed25519. rewget verifies signatures before applying updates.

The public key is embedded in rewget. Updates from unofficial sources will fail verification unless `--rewget-no-verify` is used.

### Privacy Considerations

- Profiles don't contain any personal information
- They're based on default browser configurations
- No tracking or telemetry data is included

## Creating Custom Profiles

For advanced users, you can create custom profiles:

1. Create a JSON file following the format above
2. Place it in `~/.local/share/rewget/profiles/`
3. Use it with `--rewget-profile=your_profile_name`

!!! note
    Custom profiles won't be overwritten by `--rewget-update-profiles`.

## Troubleshooting Profiles

### Profile Not Found

```
[rewget] Profile 'unknown_profile' not found
```

List available profiles with `--rewget-list-profiles`.

### Update Failed

```
[rewget] Remote update failed: connection refused
[rewget] Falling back to built-in defaults...
```

Check your network connection. Built-in profiles are restored automatically.

### Verification Failed

```
[rewget] Signature verification failed
```

The profile source may be compromised. Only use `--rewget-no-verify` if you trust the source.
