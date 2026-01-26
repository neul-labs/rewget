# Overview

rewget is a drop-in wrapper around wget with automatic fallback and an optional daemon. By default, rewget attempts a plain wget request first, then progressively retries with browser emulation when encountering blocking responses (403, 429, etc.). This behavior can be disabled with `--rewget-no-fallback` for scripting.

## Terminology

- Engine: the pinned wget binary that defines canonical behavior.
- Shim: the `rewget` CLI that parses `--rewget-*` flags only.
- Daemon: `rewgetd`, a service process that executes wget jobs and manages browser pool.
- Transport: the IPC layer used between `rewget` and `rewgetd` (nng).
- Inline daemon: a foreground `rewgetd` process spawned by `rewget` on first fallback.
- Fallback: automatic retry with progressive enhancement when wget encounters blocking responses.
- Preflight: a browser-like request or JS-enabled navigation to obtain cookies and a final URL.
- Replay: running wget with the original arguments and exported state.
- Stage: one of three fallback levels (plain wget → impersonation → JS preflight).

## Compatibility invariants

On successful download (regardless of which stage succeeded):

- stdout and stderr match what wget would produce for the final URL.
- exit code is 0 on success, non-zero on failure.
- created files, timestamps, and logs match wget behavior.
- `.wgetrc` and `--config` semantics are unchanged.

With `--rewget-no-fallback`:

- Behavior is identical to running wget directly.
- stdout, stderr, and exit code match the engine exactly.

## Operating modes

- **Stage 1 (Plain wget)**: `rewget` runs `wget_engine` directly. If successful, done. If blocked, proceed to Stage 2.
- **Stage 2 (Impersonation)**: Uses browser-like headers and TLS fingerprint to obtain cookies and final URL, then replays with wget.
- **Stage 3 (JS preflight)**: Runs a real browser session to solve challenges, exports cookies, then replays with wget.

The daemon (`rewgetd`) is started inline when Stage 2 or 3 is needed. It manages a warm browser pool for subsequent requests.

## Fallback stages

rewget automatically progresses through these stages on failure:

- **Impersonation preflight (Stage 2)**: uses a browser-like request profile to obtain a final URL, headers, and cookies, then replays with wget.
- **JS preflight (Stage 3)**: uses a real browser session to solve challenges (Cloudflare, CAPTCHAs) and export cookies before replaying with wget.

Both stages preserve wget's behavior for recursion, output files, and logging while improving fetch success on protected sites.

## Fallback semantics

- Fallback applies to **every request**, including recursive downloads.
- Each blocked request independently retries through the stages.
- Cookies obtained during fallback are accumulated in the session cookie jar.
- Fallback does not alter wget flags, output paths, or timestamping behavior.
- The daemon is spawned inline when Stage 2 or 3 is first needed.
- Fallback messages are printed to stderr (suppressible with `--rewget-quiet`).

## Timeouts

Each stage has an independent timeout:

- **Stage 1**: Inherits wget's timeout settings (`--timeout`, `--connect-timeout`, etc.)
- **Stage 2**: 15 seconds default (`--rewget-timeout-stage2`)
- **Stage 3**: 30 seconds default (`--rewget-timeout-stage3`)

When a stage times out, rewget proceeds to the next stage (if available).

## Cookie handling

- Cookies supplied via wget flags remain valid for the replay.
- Preflight cookies are merged with any existing cookie jar.
- In conflicts, the most specific cookie (domain + path + name) should win for the replay.

## Engine selection

rewget supports two wget implementations:

| Engine | Flag | Notes |
|--------|------|-------|
| GNU Wget | `--rewget-engine=wget` | Default, maximum compatibility |
| GNU Wget2 | `--rewget-engine=wget2` | HTTP/2 support, modern features |

Each engine is a separate compliance target with its own golden tests.

## Domain stage caching

rewget remembers which stage succeeded for each domain:

- First request: Stage 1 fails → Stage 2 succeeds → cache `domain → Stage 2`
- Future requests: Start directly at Stage 2

Cache expires after 7 days. Disable with `--rewget-no-cache`.

## Implementation direction

- Rust is the primary implementation language for `rewget` and `rewgetd`.
- nng is the IPC transport for request and streaming semantics.
