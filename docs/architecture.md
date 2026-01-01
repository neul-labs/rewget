# Architecture

## High-level flow

1. `rwget` parses its own `--rwget-*` flags.
2. If no rwget options are used, it execs `wget_engine` directly.
3. If daemon routing is enabled, `rwget` sends an RPC to `rwgetd`.
4. `rwgetd` runs the job, streams stdout/stderr, and returns the exit code.
5. Optional preflight can run before the wget replay in daemon mode.

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

## Execution paths

### Direct exec path

- `rwget` replaces itself with `wget_engine` using `exec`.
- Signal handling and terminal state are inherited from the caller.

### Daemon path

- `rwget` sends a request containing argv, environment delta, cwd, and TTY metadata.
- `rwgetd` spawns `wget_engine` and streams stdout/stderr back to the client.
- The daemon returns the exit code as produced by the engine.
- The daemon may be auto-started inline on first use and auto-shutdown after an idle timeout.

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

## JS preflight routing

- JS preflight always runs inside `rwgetd`.
- If `--rwget-js` is provided, `rwget` must use the daemon path.
