# Architecture

## High-level flow

1. `rwget` parses its own `--rwget-*` flags.
2. Stage 1: Run `wget_engine` and capture response.
3. If Stage 1 fails with a triggering condition, proceed to Stage 2 (impersonation).
4. If Stage 2 fails, proceed to Stage 3 (JS preflight via daemon).
5. On success at any stage, stream output and exit.

```
┌─────────────────────────────────────────────────────────────────┐
│                         rwget URL                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Stage 1: Plain wget                                            │
│  - Run wget_engine directly                                     │
│  - Check exit code and response                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                    success?  ├──────────────────────► done
                              │ no
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Stage 2: Impersonation (wget-impersonate)                      │
│  - Browser TLS fingerprint (JA3/JA4)                            │
│  - Browser HTTP/2 fingerprint                                   │
│  - Browser header order                                         │
│  - Extract cookies + final URL, replay with wget                │
└─────────────────────────────────────────────────────────────────┘
                              │
                    success?  ├──────────────────────► done
                              │ no
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Stage 3: JS Preflight (rwgetd)                                 │
│  - Full browser session (Chromium)                              │
│  - Solve challenges (Cloudflare, CAPTCHAs)                      │
│  - Export cookies + final URL, replay with wget                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                      done (success or final failure)
```

## Components

### rwget (shim)

- Parses only `--rwget-*` flags.
- Leaves all other arguments intact and in order.
- Preserves environment and working directory.
- Removes `--rwget-*` flags before invoking wget.

### rwgetd (daemon)

- Executes wget jobs as a service.
- Owns warm resources (browser pool, cookie storage).
- Streams stdout/stderr to match wget output.
- Manages per-profile state for preflight runs.

### wget_engine

- Pinned wget binary used for compliance.
- The source of truth for strict behavior.

## Failure detection

Fallback is triggered by two mechanisms:

### Status code detection (fast path)

Checked immediately from wget's exit code and server response headers:

| Code | Meaning | Action |
|------|---------|--------|
| 403 | Forbidden | Trigger fallback |
| 429 | Too Many Requests | Trigger fallback |
| 503 | Service Unavailable | Trigger fallback |
| 520-529 | Cloudflare errors | Trigger fallback |

### Body pattern detection (soft blocks)

Some sites return `200 OK` with a challenge page. rwget buffers small responses and scans for known patterns:

| Pattern | Indicates |
|---------|-----------|
| `cf-browser-verification` | Cloudflare JS challenge |
| `_cf_chl_opt` | Cloudflare challenge |
| `Please enable JavaScript` | Generic JS requirement |
| `Checking your browser` | Browser verification |
| `<noscript>` with redirect | JS-required redirect |
| `Pardon Our Interruption` | Distil Networks |
| `press & hold` | Cloudflare Turnstile |

Body detection only runs when:
- Response status is 200
- Content-Type is `text/html`
- Response size < 100KB (challenge pages are small)

Configurable via `--rwget-fallback-patterns` (add custom patterns) or `--rwget-no-body-detection` (disable).

## Execution paths

### Default path (with fallback)

In default mode, rwget spawns wget as a child process to capture the response:

1. `rwget` spawns `wget_engine` as a subprocess (not exec).
2. Captures exit code and, for small HTML responses, buffers content.
3. If success, streams buffered output and exits.
4. If failure detected, proceeds to Stage 2 (no output emitted yet).
5. Stage 2/3 run via daemon, stream output on success.

```
rwget
  ├─ spawn wget_engine (Stage 1)
  │    └─ capture exit code + response
  ├─ if blocked: spawn/connect rwgetd
  │    ├─ Stage 2: impersonation request
  │    └─ Stage 3: JS preflight
  └─ stream successful output to caller
```

### Strict path (`--rwget-no-fallback`)

With `--rwget-no-fallback`, rwget uses exec for zero overhead:

- `rwget` replaces itself with `wget_engine` using `exec()`.
- Signal handling and terminal state are inherited from the caller.
- No response capture, no fallback, identical to running wget directly.

### Daemon path (`--rwget-daemon=on`)

When daemon is forced, all requests route through `rwgetd`:

- `rwget` sends a request containing argv, environment delta, cwd, and TTY metadata.
- `rwgetd` spawns `wget_engine` and streams stdout/stderr back to the client.
- The daemon returns the exit code as produced by the engine.
- Fallback still applies unless `--rwget-no-fallback` is also set.

## IPC

The daemon provides a small RPC surface:

- `ExecWget(argv, env_delta, cwd, tty_info, stdio_mode)`
- `Status`
- `Warm`
- `Shutdown`

Transport uses nng to support request/response plus streaming stdout/stderr.

### Stream handling

- stdout and stderr are streamed separately, preserving byte order within each stream.
- When attached to a TTY, the daemon must preserve progress output behavior.
- Buffering should be minimal and should not alter line or carriage-return semantics.

### Signal handling

- Ctrl-C and termination signals should be propagated to the running job.
- The daemon should reflect the same exit status as a local exec run.

## State and isolation

- Cookie jars are stored per profile namespace to avoid cross-job leaks.
- Preflight artifacts (cookies, final URLs) are scoped to a single job.
- Daemon state must not modify wget behavior unless explicitly requested.

## Domain stage cache

rwget caches which stage succeeded for each domain to skip unnecessary retries.

### Cache behavior

```
First request to protected.example.com:
  Stage 1 → 403 → Stage 2 → Success
  Cache: protected.example.com → Stage 2

Second request to protected.example.com:
  Skip Stage 1, start at Stage 2 → Success
```

### Cache storage

Location: `~/.cache/rwget/stage-cache.json`

```json
{
  "protected.example.com": {
    "stage": 2,
    "profile": "chrome_120",
    "cached_at": 1704067200,
    "expires": 1704672000
  }
}
```

### Cache policy

| Event | Action |
|-------|--------|
| Stage 2/3 success | Cache domain → stage mapping |
| Stage 1 success | No caching needed |
| Cached stage fails | Clear cache entry, retry from Stage 1 |
| Cache entry > 7 days | Expire, retry from Stage 1 |

### Flags

- `--rwget-no-cache`: Ignore cache, always start at Stage 1
- `--rwget-clear-cache`: Delete cache file

## Engine selection

rwget supports multiple wget implementations:

| Engine | Binary | Notes |
|--------|--------|-------|
| wget | `wget_engine` | GNU Wget 1.x, default |
| wget2 | `wget2_engine` | GNU Wget2, HTTP/2 support |

### Selection priority

1. `--rwget-engine=wget2` flag (highest)
2. `RWGET_ENGINE=wget2` environment variable
3. Config file setting
4. Default: `wget`

### Compatibility notes

- wget and wget2 have slightly different CLI flags
- Golden tests run against both engines separately
- Some wget2 features (HTTP/2) may affect fingerprinting

## Impersonation layer (Stage 2)

The impersonation layer mimics browser TLS and HTTP fingerprints without running a full browser. This is faster than Stage 3 and handles most bot detection systems.

**Execution**: Stage 2 runs inside `rwgetd` (the daemon). The daemon is spawned inline on first Stage 2 request and kept alive for subsequent requests.

### TLS fingerprinting

Browser detection relies on the TLS Client Hello message. Key fields to match:

| Field | Description |
|-------|-------------|
| Cipher suites | Order and selection of ciphers |
| Extensions | SNI, ALPN, supported versions, key share |
| Curves | Elliptic curves and order |
| Signature algorithms | Signing algorithm preferences |
| GREASE | Random values Chrome uses to test server compliance |

The combined fingerprint is often expressed as JA3 or JA4 hash.

### HTTP/2 fingerprinting

HTTP/2 connections expose additional fingerprint surface:

| Field | Description |
|-------|-------------|
| SETTINGS frame | Initial window size, max streams, etc. |
| WINDOW_UPDATE | Flow control behavior |
| PRIORITY frames | Stream priority (deprecated but still sent) |
| Header order | Order of pseudo-headers and regular headers |
| HPACK | Dynamic table size and indexing behavior |

### Implementation approach

Built in Rust as part of `rwgetd`:

```
┌──────────────────────────────────────────────────────────────┐
│  rwget-impersonate (Rust)                                    │
├──────────────────────────────────────────────────────────────┤
│  rustls + custom ClientConfig                                │
│  - BoringSSL-derived cipher suites                           │
│  - Browser extension order                                   │
│  - GREASE support                                            │
├──────────────────────────────────────────────────────────────┤
│  hyper + h2 (patched)                                        │
│  - Chrome/Firefox SETTINGS values                            │
│  - Browser header order                                      │
│  - Browser-like flow control                                 │
├──────────────────────────────────────────────────────────────┤
│  Profile database                                            │
│  - chrome_120, chrome_121, ...                               │
│  - firefox_121, firefox_122, ...                             │
│  - safari_17, edge_120, ...                                  │
└──────────────────────────────────────────────────────────────┘
```

### Profile updates

Browser fingerprints change with each release. The profile database should be:
- Versioned independently from rwget releases
- Updateable via `rwget --rwget-update-profiles`
- Shipped with last 3 versions of each major browser

### Fallback to Stage 3

Impersonation fails when:
- Site requires actual JavaScript execution
- Site uses CAPTCHA or proof-of-work challenges
- Site checks additional browser APIs (canvas, WebGL, etc.)

## JS preflight routing (Stage 3)

- JS preflight always runs inside `rwgetd`.
- Uses headless Chromium via `chromiumoxide` or `headless_chrome` crate.
- Browser pool is kept warm for subsequent requests.
- If `--rwget-js` is provided, `rwget` skips directly to Stage 3.
