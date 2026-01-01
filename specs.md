Yes — same playbook works for `wget`, and it’s actually *cleaner* because GNU Wget’s surface area is big but generally less “behaviorally finicky” than curl’s trace/progress quirks.

The key is identical:

* **Compliance source of truth:** ship/execute a real `wget` (pinned build)
* **Shim:** `kwget` installed as `wget` (optional), pass-through by default
* **Daemon:** warms heavy stuff + optional layers (impersonation, JS preflight + replay)
* **IPC:** nng, streaming stdout/stderr byte-for-byte

Below is a tight spec for **Kwget** that matches the Kurl model.

---

## Spec: `kwget` (wget drop-in) + `kwgetd` daemon

### 0) Invariants (strict mode default)

When no `--kwget-*` flags (and no env opt-in) are used:

* stdout identical to `wget_engine`
* stderr identical (including progress)
* exit code identical
* files created identical (`-O`, `-o` log, `--content-disposition`, recursive downloads, timestamping)
* `.wgetrc` and `--config` semantics identical

Strict mode must pass a golden suite against the bundled engine.

---

## 1) Components

### 1.1 `kwget` shim

* Intercepts only `--kwget-*`
* Otherwise execs `wget_engine` with argv unchanged
* Optionally routes through daemon for “exec service” acceleration (still identical)

### 1.2 `kwgetd` daemon (nng)

* Runs wget jobs as a service
* Owns Chromium warm pool (for JS preflight)
* Owns cookie jar store + profile namespaces
* Streams stdout/stderr to shim

### 1.3 Wget Engine

Prefer bundled pinned:

* **GNU Wget** (classic)
* optionally support **Wget2** as an alternate engine (but treat as separate compliance target; don’t mix)

---

## 2) IPC (nng)

Same as kurl:

* `ExecWget(argv, env_delta, cwd, tty_info, stdio_mode) -> job_id + stdout/stderr streams + exit`
* `Status`, `Warm`, `Shutdown`

Streaming is critical to preserve wget’s progress/log formatting.

---

## 3) Layering (explicit opt-in only)

### Namespaced flags

* `--kwget-impersonate chrome` (switch engine to wget-impersonation path; see below)
* `--kwget-js` (force JS preflight)
* `--kwget-js-wait <domcontentloaded|networkidle|selector:...>`
* `--kwget-js-timeout <ms>`
* `--kwget-daemon <auto|on|off>`
* `--kwget-debug`

### 3.1 Impersonation layer (realism)

Unlike curl-impersonate, there isn’t a single universally-standard “wget-impersonate”. So the practical approach is:

* If user enables `--kwget-impersonate`, daemon performs a **curl-impersonate fetch** to obtain:

  * final URL
  * response headers
  * cookies
* Then `wget_engine` does the actual download using:

  * cookies (Netscape jar)
  * original wget args
  * possibly `--header` additions if explicitly requested

This keeps wget semantics for file writing / recursion, while using a browser-like front door.

(We can later add a “native” impersonation engine if it emerges, but this works now.)

### 3.2 JS layer: preflight then replay

* Chromium preflight navigates to the URL(s) and resolves challenges
* Export cookies + final resolved URL(s)
* Replay with `wget_engine` so:

  * `-r`, `-l`, `--mirror`, `--timestamping`, `--content-disposition`, etc. behave like wget

Important: JS mode for recursive downloads is tricky:

* v1 scope: JS preflight only for the **root URL**; recursion uses cookies thereafter.
* v2 scope: optionally preflight per-domain or per-top-level navigation only.

---

## 4) Compliance test plan (Kwget)

Golden harness runs **wget_engine** vs **kwget**.

Minimum suite:

1. single file download
2. redirects (`--max-redirect`)
3. `-O`, `-P`, `--content-disposition`
4. auth (basic, bearer header)
5. cookies (`--save-cookies`, `--load-cookies`, `--keep-session-cookies`)
6. proxy flags
7. recursion basics (`-r`, `-l 1`, `--spider`)
8. timestamping (`-N`) and conditional GETs
9. logging (`-o logfile`, `-q`, `-nv`)

Gate: strict suite must be green before any `--kwget-*` features ship.

---

## 5) UX / Installation

* Provide binaries:

  * `kwget` (can be installed as `wget`)
  * `kwgetd`
  * bundled `wget_engine` (and optionally `wget2_engine`)
* Default behavior matches wget exactly.
* Users opt into layers via `--kwget-*`.

---

## 6) Shared daemon vs separate daemons?

You can do either:

### Option A: One daemon for both (`knetd`)

* Single warm Chromium pool
* Shared cookie/profile store
* RPCs: `ExecCurl`, `ExecWget`
* Less overhead, better warm reuse

### Option B: Separate daemons (`kurld`, `kwgetd`)

* Simpler mental model
* Fewer cross-compat constraints

**Recommendation:** one daemon (`knetd`) once kurl is stable; start with separate if you want faster v1.

---

If you want the “everything is one daemon” direction, I’ll write the unified spec (`knetd`) with two frontends (`kurl`, `kwget`) and shared protocol/versioning so you don’t paint yourself into a corner.

