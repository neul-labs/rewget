# rwget

rwget is a wget-compatible wrapper with an optional daemon. By default, it behaves exactly like wget and passes through all arguments unchanged. Optional `--rwget-*` flags enable acceleration and preflight layers while keeping wget's file and logging semantics.

## Principles

- Exact wget behavior by default (stdout, stderr, exit code, and files).
- Opt-in enhancements only; no surprises when you do not use `--rwget-*` flags.
- Streaming output preserved byte-for-byte.
- Clear separation between the wrapper, daemon, and the wget engine.

## What rwget is

- A thin shim that accepts every wget option unchanged.
- A compatibility layer that can proxy execution through a daemon.
- A framework for preflight and replay that preserves wget semantics.

## What rwget is not

- A new downloader with its own semantics.
- A replacement for wget configuration or logging formats.
- A wrapper that changes behavior without explicit `--rwget-*` flags.

## Components

- `rwget`: CLI shim that parses only `--rwget-*` flags.
- `rwgetd`: daemon that runs wget jobs, manages warm resources, and streams output.
- `wget_engine`: a pinned wget binary used as the compliance baseline.

## Operating modes

- Direct exec: `rwget` replaces itself with `wget_engine`.
- Daemon exec: `rwget` sends an RPC to `rwgetd` and streams output. The daemon can be started inline on first use.
- Preflight + replay: an optional browser-like preflight exports cookies and a final URL, then `wget_engine` performs the download. JS preflight runs in the daemon.
- The daemon can be started on first use and auto-shutdown after an idle timeout.

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

- `docs/overview.md` - goals, terminology, and behavior at a glance
- `docs/architecture.md` - execution flow and IPC design
- `docs/cli.md` - `--rwget-*` flags and usage patterns
- `docs/daemon.md` - daemon responsibilities and streaming guarantees
- `docs/compliance.md` - strict mode and golden suite expectations

## Status

This repository focuses on design and documentation. Implementation details may evolve, but the compatibility goals should remain stable.

## Implementation notes

- The primary implementation language is Rust.
- IPC uses nng for request/stream transport between `rwget` and `rwgetd`.
