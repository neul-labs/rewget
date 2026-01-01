# Compliance

Compatibility with wget is the primary goal. rwget is expected to pass a golden test suite that compares `rwget` output against the pinned `wget_engine`.

## Strict mode

When no `--rwget-*` flags are used:

- stdout and stderr match exactly.
- exit codes are identical.
- files and logs created by wget match exactly.

## Golden suite goals

The suite should validate that rwget is indistinguishable from wget in normal use, including:

- Command-line flag handling and ordering.
- Progress and logging output in TTY and non-TTY contexts.
- File output and metadata behavior.
- Recursion and timestamping behavior.

## Minimum golden suite

1. Single file download
2. Redirect handling (`--max-redirect`)
3. Output paths (`-O`, `-P`, `--content-disposition`)
4. Authentication (basic, bearer header)
5. Cookies (`--save-cookies`, `--load-cookies`, `--keep-session-cookies`)
6. Proxy flags
7. Recursion basics (`-r`, `-l 1`, `--spider`)
8. Timestamping (`-N`) and conditional GETs
9. Logging (`-o logfile`, `-q`, `-nv`)

## Harness expectations

- Each test runs wget and rwget with identical inputs and environment.
- Output comparisons are byte-for-byte for stdout, stderr, and log files.
- File tree comparisons verify paths, sizes, and timestamps.
- Tests should run against a controlled fixture server to avoid flakiness.

## Platform matrix

- Linux is the initial reference platform.
- Additional platforms should be added only after strict mode is stable on Linux.
- Each platform must have a pinned wget binary for the compliance baseline.

## Gate

No rwget-specific features should ship until strict mode passes the suite on supported platforms.
