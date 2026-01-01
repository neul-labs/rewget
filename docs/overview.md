# Overview

rwget is a drop-in wrapper around wget with an optional daemon. The default path is a direct exec of the bundled wget engine with arguments unchanged. Additional features are explicitly opt-in via `--rwget-*` flags.

## Compatibility invariants

When no `--rwget-*` flags are present:

- stdout and stderr match the engine output exactly.
- exit code matches the engine exit code.
- created files, timestamps, and logs match the engine behavior.
- `.wgetrc` and `--config` semantics are unchanged.

## Opt-in layers

rwget adds two optional layers:

- Impersonation preflight: uses a browser-like request to obtain a final URL, headers, and cookies, then replays with wget for file semantics.
- JS preflight: uses a real browser session to solve challenges and export cookies before replaying with wget.

Both layers are intended to preserve wget's behavior for recursion, output files, and logging while improving fetch success on complex sites.

## Engine selection

The baseline is a pinned GNU Wget binary. If a second engine is supported (for example, Wget2), it is a separate compliance target and must be explicitly selected.
