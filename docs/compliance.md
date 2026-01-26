# Compliance

Compatibility with wget is the primary goal. rewget should produce identical output to wget when downloads succeed, regardless of which fallback stage was used.

## Compatibility modes

### Default mode (with fallback)

When rewget succeeds at any stage:

- Downloaded files match what wget would produce for the final URL.
- Exit code is 0 on success.
- Fallback messages go to stderr (suppressible with `--rewget-quiet`).

### Strict mode (`--rewget-no-fallback`)

When `--rewget-no-fallback` is used:

- stdout and stderr match wget exactly.
- Exit codes are identical to wget.
- Files and logs created match wget exactly.
- No automatic retries; single-attempt behavior.

Use strict mode for:
- Scripting that depends on exact wget behavior
- CI/CD pipelines
- Compatibility testing

## Golden suite goals

The suite validates two things:

1. **Strict mode**: rewget with `--rewget-no-fallback` is indistinguishable from wget.
2. **Fallback mode**: rewget produces correct files when fallback stages are used.

### Strict mode validation

- Command-line flag handling and ordering
- Progress and logging output in TTY and non-TTY contexts
- File output and metadata behavior
- Recursion and timestamping behavior

### Fallback mode validation

- Correct file content after Stage 2/3 success
- Cookie jar correctness after preflight
- Redirect chain handling through fallback
- Recursive downloads with mixed stage successes

## Minimum golden suite

### Strict mode tests (with `--rewget-no-fallback`)

1. Single file download
2. Redirect handling (`--max-redirect`)
3. Output paths (`-O`, `-P`, `--content-disposition`)
4. Authentication (basic, bearer header)
5. Cookies (`--save-cookies`, `--load-cookies`, `--keep-session-cookies`)
6. Proxy flags
7. Recursion basics (`-r`, `-l 1`, `--spider`)
8. Timestamping (`-N`) and conditional GETs
9. Logging (`-o logfile`, `-q`, `-nv`)

### Fallback mode tests

1. Stage 2 success after Stage 1 403
2. Stage 3 success after Stage 2 failure
3. Recursive download with per-page fallback
4. Cookie accumulation across fallback stages
5. Timeout handling per stage
6. Body pattern detection triggering fallback
7. `--rewget-quiet` suppresses fallback messages

## Harness expectations

- Strict mode tests run wget and rewget with identical inputs; compare byte-for-byte.
- Fallback tests use a fixture server that returns 403/challenge pages initially.
- File tree comparisons verify paths, sizes, and content hashes.
- Tests should run against controlled fixture servers to avoid flakiness.

## Platform matrix

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | Primary | Reference platform |
| Linux arm64 | Supported | Raspberry Pi, cloud ARM |
| macOS x86_64 | Supported | Intel Macs |
| macOS arm64 | Supported | Apple Silicon |
| Windows x86_64 | Supported | Via WSL2 or native |

Each platform must have a pinned wget binary for the compliance baseline.

## Gate

No rewget release until:
- Strict mode passes 100% on all supported platforms
- Fallback mode passes on Linux (reference platform)
