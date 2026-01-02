# Browser Profiles

Browser profiles are the key to Stage 2's effectiveness. They contain all the fingerprint data needed to impersonate a real browser.

## What's in a Profile?

Each profile contains:

- **TLS Configuration**: Cipher suites, extensions, curves, ALPN
- **HTTP/2 Settings**: SETTINGS frame values, window sizes, header order
- **HTTP Headers**: User-Agent, Accept, Accept-Language, etc.
- **Browser Metadata**: Name, version, platform

## Available Profiles

rwget ships with 6 built-in profiles:

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
rwget --rwget-list-profiles
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
rwget --rwget-profile=firefox_136 https://example.com/
```

### Verify Profile Details

```bash
rwget --rwget-verify-profile=chrome_131
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
rwget --rwget-update-profiles
```

Output:
```
[rwget] Fetching profiles from: https://rwget.dev/profiles/v1/index.json
[rwget] Updated: chrome_132, firefox_137
[rwget] Added: chrome_133
[rwget] Unchanged: 4 profiles
[rwget] Total: 7 profiles
```

### Custom Profile Source

Use your own profile server:

```bash
rwget --rwget-profile-url=https://my-server.com/profiles.json --rwget-update-profiles
```

### Skip Verification

For testing only:

```bash
rwget --rwget-no-verify --rwget-update-profiles
```

## Profile Selection Logic

When no profile is specified, rwget selects automatically:

1. **Cached profile**: If the domain has a cached successful profile, use it
2. **Chrome latest**: Try the newest Chrome profile first
3. **Rotation**: On failure, try other profiles in order

## Profile File Format

Profiles are stored as JSON in `~/.local/share/rwget/profiles/`:

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

Profile updates are signed with Ed25519. rwget verifies signatures before applying updates.

The public key is embedded in rwget. Updates from unofficial sources will fail verification unless `--rwget-no-verify` is used.

### Privacy Considerations

- Profiles don't contain any personal information
- They're based on default browser configurations
- No tracking or telemetry data is included

## Creating Custom Profiles

For advanced users, you can create custom profiles:

1. Create a JSON file following the format above
2. Place it in `~/.local/share/rwget/profiles/`
3. Use it with `--rwget-profile=your_profile_name`

!!! note
    Custom profiles won't be overwritten by `--rwget-update-profiles`.

## Troubleshooting Profiles

### Profile Not Found

```
[rwget] Profile 'unknown_profile' not found
```

List available profiles with `--rwget-list-profiles`.

### Update Failed

```
[rwget] Remote update failed: connection refused
[rwget] Falling back to built-in defaults...
```

Check your network connection. Built-in profiles are restored automatically.

### Verification Failed

```
[rwget] Signature verification failed
```

The profile source may be compromised. Only use `--rwget-no-verify` if you trust the source.
