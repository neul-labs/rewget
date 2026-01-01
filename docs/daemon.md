# Daemon

`rwgetd` is a long-running process that executes wget jobs and manages warm resources. It is optional and only used when `rwget` opts in via `--rwget-daemon` or other rwget features.

## Responsibilities

- Execute `wget_engine` with a provided argv and environment delta.
- Stream stdout and stderr back to the client in the same order and format as local execution.
- Manage a browser pool for JS or impersonation preflight. JS fallback runs in the daemon.
- Store and isolate cookie jars per profile namespace.

## Lifecycle

- The daemon may be started explicitly or on demand.
- On first use, `rwget` can initialize the daemon automatically and run it inline.
- Inline means `rwgetd` runs as a foreground child process for that invocation, not a background service.
- The daemon should auto-shutdown after an idle timeout to avoid lingering processes.
- `--rwget-daemon=auto` should fall back to local exec if the daemon is unavailable.
- `--rwget-daemon=on` should fail fast if the daemon cannot be reached.

## Job execution

- Each job is executed in its own process with a defined working directory.
- Environment deltas are applied on top of the daemon's base environment.
- Preflight artifacts are scoped to a single job and must not leak across runs.

## Streaming guarantees

- Output is streamed byte-for-byte with minimal buffering.
- Exit codes are returned exactly as produced by the engine.
- Signal propagation should match local process behavior.

## Preflight resources

- A warm browser pool reduces startup latency for JS preflight.
- Cookie storage is namespaced by profile to avoid cross-site contamination.
- Warm state should never change wget replay behavior.

## Observability

- `--rwget-debug` enables verbose routing and protocol logs.
- The daemon should log only when explicitly configured to avoid polluting wget output.
