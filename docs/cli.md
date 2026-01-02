# CLI

rwget accepts all standard wget flags. Additional options are namespaced with `--rwget-*` and are ignored by the underlying engine.

## General rules

- `--rwget-*` flags can appear anywhere in the argument list.
- All other flags and positional arguments are passed to wget unchanged.
- By default, rwget automatically retries with progressive fallback when wget encounters blocking responses.
- If rwget fails before invoking the engine, it should emit an error on stderr and exit with a non-zero status.

## Automatic fallback

By default, rwget uses a three-stage fallback strategy:

1. **Stage 1**: Plain wget request
2. **Stage 2**: Impersonation preflight (browser-like headers and TLS fingerprint)
3. **Stage 3**: Full JS preflight (real browser session)

Fallback is triggered by these status codes (configurable):
- `403` Forbidden
- `429` Too Many Requests
- `503` Service Unavailable
- `520-529` Cloudflare-specific errors

For recursive downloads (`-r`), fallback applies to **all requests**, not just the root URL. Each request that fails with a triggering status code will retry through the stages.

When fallback occurs, rwget prints a message to stderr:

```
[rwget] 403 Forbidden - retrying with impersonation...
[rwget] 403 Forbidden - retrying with JS preflight...
[rwget] Success at Stage 2 (impersonation)
```

Use `--rwget-quiet` to suppress these messages.

## rwget options

### Fallback control

- `--rwget-no-fallback`
  - Disables automatic fallback. rwget will behave exactly like wget and fail immediately on blocking responses.
  - Use this for scripting when you need predictable, single-attempt behavior.

- `--rwget-fallback-codes <codes>`
  - Comma-separated list of HTTP status codes that trigger fallback.
  - Default: `403,429,503,520,521,522,523,524,525,526,527,528,529`
  - Example: `--rwget-fallback-codes=403,429` to only retry on 403 and 429.

- `--rwget-fallback-stage <1|2|3>`
  - Start fallback at a specific stage instead of Stage 1.
  - `1`: Plain wget (default starting point)
  - `2`: Start with impersonation preflight
  - `3`: Start with JS preflight
  - Useful when you know a site requires browser emulation.

- `--rwget-fallback-patterns <patterns>`
  - Comma-separated list of additional body patterns that trigger fallback.
  - Patterns are matched as substrings in HTML responses.
  - Example: `--rwget-fallback-patterns="Access Denied,Please wait"`.

- `--rwget-no-body-detection`
  - Disable body pattern detection. Only use status codes for fallback.
  - Useful when downloading HTML files that may contain challenge-like text.

- `--rwget-quiet`
  - Suppress rwget fallback messages. Only wget output is shown.
  - Does not affect `--rwget-debug`.

### Timeout control

- `--rwget-timeout-stage1 <ms>`
  - Timeout for Stage 1 (plain wget). Default: inherits wget's timeout settings.

- `--rwget-timeout-stage2 <ms>`
  - Timeout for Stage 2 (impersonation preflight). Default: 15000 (15 seconds).

- `--rwget-timeout-stage3 <ms>`
  - Timeout for Stage 3 (JS preflight). Default: 30000 (30 seconds).
  - JS-heavy sites may need higher values.

### Daemon control

- `--rwget-daemon <auto|on|off>`
  - `auto` (default): starts the daemon inline when fallback reaches Stage 2 or 3.
  - `on`: forces daemon routing for all requests, including Stage 1.
  - `off`: disables persistent daemon; each Stage 2/3 request spawns a fresh process (slower, no warm pool).

### Preflight options

- `--rwget-js`
  - Forces JavaScript-enabled preflight (Stage 3) from the start.
  - Spawns daemon inline if not already running.

- `--rwget-js-wait <domcontentloaded|networkidle|selector:...>`
  - Controls how long the browser waits before exporting cookies and URLs.
  - Only applies to Stage 3 (JS preflight).

### Impersonation profiles

- `--rwget-profile <name>`
  - Use a specific browser profile for Stage 2 impersonation.
  - Examples: `chrome_120`, `firefox_121`, `safari_17`, `edge_120`.
  - Default: latest Chrome profile.
  - Shorthand: `--rwget-profile=chrome` uses latest Chrome version.

- `--rwget-update-profiles`
  - Download latest browser fingerprint profiles from the update server.
  - Profiles are stored in `~/.config/rwget/profiles/`.

- `--rwget-list-profiles`
  - List all available impersonation profiles.

### Engine selection

- `--rwget-engine <wget|wget2>`
  - Select which wget engine to use for downloads.
  - Default: `wget`.
  - Can also be set via `RWGET_ENGINE` environment variable.

### Stage caching

- `--rwget-no-cache`
  - Disable domain stage caching. Always start at Stage 1.
  - Useful for testing or when site behavior has changed.

- `--rwget-clear-cache`
  - Clear the stage cache and exit.
  - Cache location: `~/.cache/rwget/stage-cache.json`.

### Chromium management

- `--rwget-download-chromium`
  - Pre-download Chromium for Stage 3 JS preflight.
  - Chromium is normally downloaded on first Stage 3 use (~150MB).

- `--rwget-chromium-path`
  - Print the path to the Chromium installation and exit.

### Profile verification

- `--rwget-verify-profile <name>`
  - Verify a profile's fingerprints against test servers.
  - Checks TLS (JA3/JA4), HTTP/2 (Akamai), and header order.
  - Useful for validating custom or updated profiles.

### Debugging

- `--rwget-debug`
  - Enables verbose diagnostic output for rwget, fallback stages, and daemon routing.
  - Shows per-stage timing, cookies obtained, and final URL after redirects.

## Behavioral notes

- Automatic fallback applies to every request, including recursive downloads.
- Cookies obtained during fallback are merged into the cookie jar for subsequent requests.
- Fallback preserves wget output paths and logging formats.
- When `--rwget-debug` is enabled, each fallback attempt is logged to stderr.

## Examples

Download with automatic fallback (default behavior):

```bash
rwget https://example.com/file.tar.gz
```

Disable fallback for scripting:

```bash
rwget --rwget-no-fallback https://example.com/file.tar.gz
```

Only trigger fallback on 403 errors:

```bash
rwget --rwget-fallback-codes=403 https://example.com/file.tar.gz
```

Start directly with browser emulation (skip Stage 1):

```bash
rwget --rwget-fallback-stage=2 https://example.com/file.tar.gz
```

Force JS preflight from the start:

```bash
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/
```

Recursive download with fallback on all pages:

```bash
rwget -r -l 2 https://example.com/
```

Debug mode to see fallback progression:

```bash
rwget --rwget-debug https://protected-site.com/file.tar.gz
```

Quiet mode (suppress fallback messages):

```bash
rwget --rwget-quiet https://protected-site.com/file.tar.gz
```

Custom per-stage timeouts:

```bash
rwget --rwget-timeout-stage2=10000 --rwget-timeout-stage3=60000 https://slow-site.com/
```
