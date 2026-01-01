# CLI

rwget accepts all standard wget flags. Additional options are namespaced with `--rwget-*` and are ignored by the underlying engine.

## rwget options

- `--rwget-daemon <auto|on|off>`
  - `auto` uses the daemon when available and safe for strict output streaming.
  - `on` forces daemon routing.
  - `off` always execs the engine locally.

- `--rwget-impersonate <profile>`
  - Runs a browser-like preflight to obtain the final URL and cookies, then replays with wget.
  - Example profiles: `chrome`, `firefox`.

- `--rwget-js`
  - Performs a JavaScript-enabled preflight before replaying with wget.

- `--rwget-js-wait <domcontentloaded|networkidle|selector:...>`
  - Controls how long the browser waits before exporting cookies and URLs.

- `--rwget-js-timeout <ms>`
  - Timeout for the JS preflight phase.

- `--rwget-debug`
  - Enables verbose diagnostic output for rwget and daemon routing.

## Examples

Fetch a file normally:

```bash
rwget https://example.com/file.tar.gz
```

Use the daemon when available:

```bash
rwget --rwget-daemon=auto https://example.com/file.tar.gz
```

Enable JS preflight:

```bash
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/
```
