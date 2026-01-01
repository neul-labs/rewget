# rwget

rwget is a wget-compatible wrapper with an optional daemon. By default, it behaves exactly like wget and passes through all arguments unchanged. Optional `--rwget-*` flags enable acceleration and preflight layers while keeping wget's file and logging semantics.

## Goals

- Exact wget behavior by default (stdout, stderr, exit code, and files).
- Opt-in enhancements only; no surprises when you do not use `--rwget-*` flags.
- Streaming output preserved byte-for-byte.
- A clean separation between the wrapper, daemon, and the wget engine.

## Components

- `rwget`: CLI shim that accepts all wget flags plus `--rwget-*` options.
- `rwgetd`: daemon that runs wget jobs, manages warm resources, and streams output.
- `wget_engine`: a pinned wget binary used as the compliance baseline.

## Quick usage

Behavior identical to wget:

```bash
rwget https://example.com/file.tar.gz
```

Enable daemon routing:

```bash
rwget --rwget-daemon=on https://example.com/file.tar.gz
```

Preflight with a browser session before replaying with wget:

```bash
rwget --rwget-js --rwget-js-wait=networkidle https://example.com/
```

## Documentation

- `docs/overview.md`
- `docs/architecture.md`
- `docs/cli.md`
- `docs/daemon.md`
- `docs/compliance.md`

## Status

This repository currently focuses on design and documentation. Implementation details may evolve, but the compatibility goals should remain stable.
