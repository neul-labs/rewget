# Daemon

`rwgetd` is a long-running process that executes wget jobs and manages warm resources. It is optional and only used when `rwget` opts in via `--rwget-daemon` or other rwget features.

## Responsibilities

- Execute `wget_engine` with a provided argv and environment delta.
- Stream stdout and stderr back to the client in the same order and format as local execution.
- Manage a browser pool for JS preflight.
- Store and isolate cookie jars per profile namespace.

## Streaming guarantees

- Output is streamed byte-for-byte with minimal buffering.
- Exit codes are returned exactly as produced by the engine.
- Signal propagation should match local process behavior.

## Warm resources

`rwgetd` can keep browser instances warm to reduce startup latency for JS or impersonation preflight. Warm state should never change the behavior of the underlying wget replay.
