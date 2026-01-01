# CLI

rwget accepts all standard wget flags. Additional options are namespaced with `--rwget-*` and are ignored by the underlying engine.

## General rules

- `--rwget-*` flags can appear anywhere in the argument list.
- All other flags and positional arguments are passed to wget unchanged.
- If no `--rwget-*` flags are present, rwget behaves exactly like wget.
- If rwget fails before invoking the engine, it should emit an error on stderr and exit with a non-zero status.

## rwget options

- `--rwget-daemon <auto|on|off>`
  - `auto` uses the daemon when available and safe for strict output streaming.
  - `auto` may start the daemon inline on first use and it may auto-shutdown after idle.
  - `on` forces daemon routing and starts the daemon inline if needed, failing if it cannot be reached.
  - `off` always execs the engine locally.

- `--rwget-impersonate <profile>`
  - Runs a browser-like preflight to obtain the final URL and cookies, then replays with wget.
  - Example profiles: `chrome`, `firefox`.

- `--rwget-js`
  - Performs a JavaScript-enabled preflight before replaying with wget.
  - Requires the daemon; if `--rwget-daemon=off` is set, rwget should error.

- `--rwget-js-wait <domcontentloaded|networkidle|selector:...>`
  - Controls how long the browser waits before exporting cookies and URLs.

- `--rwget-js-timeout <ms>`
  - Timeout for the JS preflight phase.

- `--rwget-debug`
  - Enables verbose diagnostic output for rwget and daemon routing.

## Behavioral notes

- Preflight applies to the root URL(s) provided on the command line.
- For recursive downloads, only the root is preflighted; recursion uses the exported cookies.
- `--rwget-impersonate` and `--rwget-js` are explicit opt-ins and should not change wget output paths or logging formats.

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

Impersonation preflight for a single URL:

```bash
rwget --rwget-impersonate=chrome https://example.com/
```

Preflight with recursion:

```bash
rwget --rwget-js -r -l 1 https://example.com/
```
