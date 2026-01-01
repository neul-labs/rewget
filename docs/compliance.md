# Compliance

Compatibility with wget is the primary goal. rwget is expected to pass a golden test suite that compares `rwget` output against the pinned `wget_engine`.

## Strict mode

When no `--rwget-*` flags are used:

- stdout and stderr match exactly.
- exit codes are identical.
- files and logs created by wget match exactly.

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

## Gate

No rwget-specific features should ship until strict mode passes the suite on supported platforms.
