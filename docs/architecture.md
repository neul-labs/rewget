# Architecture

## High-level flow

1. `rwget` parses its own `--rwget-*` flags.
2. If no rwget options are used, it execs `wget_engine` directly.
3. If daemon routing is enabled, `rwget` sends an RPC to `rwgetd`.
4. `rwgetd` runs the job, streams stdout/stderr, and returns the exit code.

## Components

### rwget (shim)

- Parses only `--rwget-*` flags.
- Leaves all other arguments intact.
- Preserves environment and working directory.

### rwgetd (daemon)

- Executes wget jobs as a service.
- Owns warm resources (browser pool, cookie storage).
- Streams stdout/stderr to match wget output.

### wget_engine

- Pinned wget binary used for compliance.
- The source of truth for strict behavior.

## IPC

The daemon provides a simple RPC protocol:

- `ExecWget(argv, env_delta, cwd, tty_info, stdio_mode)`
- `Status`
- `Warm`
- `Shutdown`

The transport is message-based and supports byte-for-byte streaming of both stdout and stderr. Signal handling and exit status should match the local exec path.
