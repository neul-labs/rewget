# Daemon

`rewgetd` is a long-running process that handles Stage 2 (impersonation) and Stage 3 (JS preflight) requests. It manages warm resources like browser pools and TLS sessions.

## When the daemon is used

| Stage | Daemon required? |
|-------|------------------|
| Stage 1 (plain wget) | No - runs directly |
| Stage 2 (impersonation) | Yes - custom TLS/HTTP2 client |
| Stage 3 (JS preflight) | Yes - browser pool |

The daemon is spawned **inline** on first Stage 2/3 request and kept alive for subsequent requests.

## Responsibilities

- Execute impersonation requests with browser-like TLS/HTTP2 fingerprints.
- Manage headless browser pool for JS preflight.
- Execute `wget_engine` replay after successful preflight.
- Stream stdout and stderr back to the client.
- Store and isolate cookie jars per profile namespace.

## Lifecycle

- The daemon is started inline on first fallback to Stage 2 or 3.
- Inline means `rewgetd` runs as a child process, not a system service.
- The daemon auto-shuts down after an idle timeout (default: 5 minutes).
- `--rewget-daemon=on` forces all requests through the daemon.
- `--rewget-daemon=off` disables persistent daemon; Stage 2/3 still work but spawn fresh processes.

## Job execution

### Stage 2 (Impersonation)

1. Daemon receives URL and profile name from `rewget`.
2. Performs request with browser TLS/HTTP2 fingerprint.
3. Extracts cookies and final URL (after redirects).
4. Spawns `wget_engine` with cookies injected.
5. Streams wget output back to client.

### Stage 3 (JS Preflight)

1. Daemon receives URL and wait conditions from `rewget`.
2. Ensures Chromium is available (lazy download if needed).
3. Allocates browser from warm pool (or spawns new).
4. Navigates browser to URL, waits for condition.
5. Exports cookies and final URL.
6. Spawns `wget_engine` with cookies injected.
7. Streams wget output back to client.
8. Returns browser to pool.

### Chromium lazy download

Chromium is downloaded on first Stage 3 use:

```
~/.local/share/rewget/chromium/
├── chrome-linux64/          # Extracted browser
├── version.txt              # e.g., "120.0.6099.109"
└── download.lock            # Prevents concurrent downloads
```

Download source: Chrome for Testing (official Google builds)
Size: ~150MB compressed, ~450MB extracted

Pre-download with: `rewget --rewget-download-chromium`

### Isolation

- Environment deltas are applied on top of the daemon's base environment.
- Preflight artifacts are scoped to a single job and must not leak across runs.
- Each job runs in the caller's working directory.

## Streaming guarantees

- Output is streamed byte-for-byte with minimal buffering.
- Exit codes are returned exactly as produced by the engine.
- Signal propagation (Ctrl-C) terminates the current job.

## Warm resources

| Resource | Purpose | Pool size |
|----------|---------|-----------|
| TLS sessions | Reuse connections for Stage 2 | Per-domain |
| Browser instances | Reduce cold start for Stage 3 | 2 (default) |
| Cookie jars | Persist across requests | Per-profile |

- Cookie storage is namespaced by profile to avoid cross-site contamination.
- Warm state should never change wget replay behavior.
- Pool sizes configurable via daemon config file.

## Observability

- `--rewget-debug` enables verbose routing and protocol logs.
- Daemon logs to `~/.local/share/rewget/daemon.log` when running.
- The daemon should not write to stdout/stderr to avoid polluting wget output.
