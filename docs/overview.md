# Overview

rwget is a drop-in wrapper around wget with an optional daemon. The default path is a direct exec of the bundled wget engine with arguments unchanged. Additional features are explicitly opt-in via `--rwget-*` flags.

## Terminology

- Engine: the pinned wget binary that defines canonical behavior.
- Shim: the `rwget` CLI that parses `--rwget-*` flags only.
- Daemon: `rwgetd`, a service process that executes wget jobs.
- Transport: the IPC layer used between `rwget` and `rwgetd` (nng).
- Inline daemon: a foreground `rwgetd` process spawned by `rwget` and tied to the current run.
- Preflight: a browser-like request or JS-enabled navigation to obtain cookies and a final URL.
- Replay: running wget with the original arguments and exported state.
- Strict mode: the default behavior when no `--rwget-*` flags are used.

## Compatibility invariants

When no `--rwget-*` flags are present:

- stdout and stderr match the engine output exactly.
- exit code matches the engine exit code.
- created files, timestamps, and logs match the engine behavior.
- `.wgetrc` and `--config` semantics are unchanged.

## Operating modes

- Direct exec: `rwget` execs `wget_engine` in-place.
- Daemon exec: `rwget` delegates to `rwgetd` and streams output. The daemon can be started inline on first use.
- Preflight + replay: a preflight collects cookies and a final URL, then wget handles file writing and recursion. JS preflight runs in the daemon and requires the inline daemon path.

## Opt-in layers

rwget adds two optional layers:

- Impersonation preflight: uses a browser-like request profile to obtain a final URL, headers, and cookies, then replays with wget for file semantics.
- JS preflight: uses a real browser session to solve challenges and export cookies before replaying with wget.

Both layers are intended to preserve wget's behavior for recursion, output files, and logging while improving fetch success on complex sites.

## Preflight semantics

- Preflight applies to the root URL(s) provided to rwget.
- Recursion uses the exported cookies and the original wget arguments.
- Preflight should not alter wget flags, output paths, or timestamping behavior.
- JS preflight requires the daemon and is not available in direct exec mode.

## Cookie handling

- Cookies supplied via wget flags remain valid for the replay.
- Preflight cookies are merged with any existing cookie jar.
- In conflicts, the most specific cookie (domain + path + name) should win for the replay.

## Engine selection

The baseline is a pinned GNU Wget binary. If a second engine is supported (for example, Wget2), it is a separate compliance target and must be explicitly selected and tested.

## Implementation direction

- Rust is the primary implementation language for `rwget` and `rwgetd`.
- nng is the IPC transport for request and streaming semantics.
