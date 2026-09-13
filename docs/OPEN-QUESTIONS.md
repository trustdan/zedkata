# Open questions

Every claim this project depends on, tagged. Nothing here is settled by memory or by an
AI's confident tone — only by current Zed docs, Zed source, or an observation reproduced
on this machine.

**Re-verify anything older than ~3 months.** Zed ships weekly.

Legend: **VERIFIED** (with date + source) · **ASSUMED** (load-bearing, unchecked) ·
**OPEN** (actively blocking)

---

## OQ-01 — `didChange` granularity · VERIFIED (2026-09-13)

Checked 2026-09-13 on Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`) on Windows.

Zed sends discrete, incremental `didChange` notifications with UTF-16 ranges without
coalescing or debouncing into full-document replacements under normal typing pace.

**Empirical evidence:**
- **Operator deletion (`dw`)**: Consistently emitted as a single notification deleting the
  exact word span `(1,17)–(1,23)` (Cases 01 & 02; sequence 32 and sequence 40).
- **Character deletion (Backspaces)**: Emitted as 5 separate notifications arriving
  ~180 ms apart, each deleting 1 character at decrementing column offsets (Case 03;
  sequences 52, 54, 56, 58, 60).
- Retained raw JSON-RPC captures: [`tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl),
  [`tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl),
  and [`tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl).
- See detailed report in [matrix results](evidence/2026-09-13-matrix-results.md) and
  gate decision in [ADR 0003](adr/0003-phase-0-feasibility-gate.md).

Blocks: unblocked. Technique scoring, `par_edits`, and `forbid` rules are supported.

---

## OQ-02 — Registering a custom language server · VERIFIED (documented dev route)

Checked 2026-09-13 against current unversioned official docs. Local Zed is 1.19.2
(`df181c6f58d02677b385fa947d6bfde6d3530078`).

The [development guide](https://zed.dev/docs/extensions/developing-extensions)
documents installing a local dev extension without publishing it. The
[language guide](https://zed.dev/docs/extensions/languages#language-servers) documents
language metadata, manifest server registration, and a Rust `language_server_command`
implementation returning a command with arguments and environment.

The [settings guide](https://zed.dev/docs/configuring-languages) covers existing
language associations and server overrides; it does not establish a settings-only
route for a new language/server pair. Use the documented dev-extension route.

**Runtime checked 2026-09-13:** the dev extension loaded in Zed 1.19.2 and launched
the probe. Captured initialize/open/change/close and outgoing diagnostics; Dan
confirmed the visible diagnostic. See [connection evidence](evidence/2026-09-13-connection.md).

**Not established:** whether another settings-only route exists.

---

## OQ-03 — Diagnostic round-trip latency · VERIFIED (2026-09-13)

Checked 2026-09-13 on Zed 1.19.2.
- Instrumented server dispatch time after `didChange` receipt is 79 µs to 1.04 ms.
- Visible rendering of inline diagnostics confirmed on hover in Zed editor (ZK-003 connection check).
- Passes the <300 ms channel gate and <100 ms target. Inline diagnostics remain a viable primary feedback channel.

---

## OQ-04 — Extension API cannot observe buffer/cursor/selections · VERIFIED

Checked 2026-09-13. Zed's extension surface covers languages, grammars, language
servers, debug adapters, themes, icon themes, snippets, and MCP servers. There is an
open feature request (zed-industries/zed discussion #53131, Apr 2026) asking for an API
to observe buffer lifecycle events with cursor and selection awareness; it has one
comment and no commitment.

Zed's own extension blog post states there is no support for modifying the UI to create
new panels, making arbitrary HTTP requests, or arbitrary filesystem access.

**Treat as permanent.** All design flows from this.

---

## OQ-05 — Fake-LSP-as-observer is a workable seam · VERIFIED (by precedent)

Checked 2026-09-13. A Zed Discord-presence extension was built by shipping a language
server that reported which files the user had open. Confirms Zed will speak LSP to an
arbitrary binary and that the channel carries document lifecycle information.

Does **not** confirm granularity — that is OQ-01.

---

## OQ-06 — Extensions must be Rust → WASM · VERIFIED

Checked 2026-09-13, zed.dev extension pages. Functional parts are written in Rust,
compiled to WebAssembly, and the repo is linked as a submodule in
`zed-industries/extensions`.

---

## OQ-07 — Custom ACP agents are a supported install path · VERIFIED

Checked 2026-09-13. Zed supports External Agents over ACP, configured under
`agent_servers` in `settings.json` with `"type": "custom"` plus command, args, and env.
There is also an ACP Registry, opened with `zed: acp registry`.

---

## OQ-08 — Terminal Threads can host a TUI · VERIFIED

Checked 2026-09-13. Zed's External Agents docs direct users to Terminal Threads when
they want to run a CLI or TUI directly in a terminal-backed thread.

Not yet verified: whether a full-screen ratatui app behaves correctly there
(alt-screen, resize, mouse capture). Test in Phase 2.

---

## OQ-09 — ACP Rust SDK maturity · VERIFIED

Checked 2026-09-13. `agent-client-protocol` reached 1.0.0 in June 2026 alongside the
TypeScript SDK. Official SDKs exist for Rust, TypeScript, Python, Kotlin, and Java.
No Go SDK.

---

## OQ-10 — Can we place the student's cursor? · ASSUMED

The `zed` CLI is assumed to accept `path:line:col` to open a file at a position, which
the TUI would use to start an exercise. Unverified against current CLI docs.

---

## OQ-11 — Undo semantics on the wire · ASSUMED

Assumed that an undo produces a `didChange` that reverses the prior delta, and can
therefore be detected and excluded from the edit count. If undo is invisible or appears
as a full-document sync, `par_edits` is trivially gameable. Check during Phase 0.

---

## OQ-12 — Does Zed's vim mode still lack anything we plan to teach? · ASSUMED

Registers and macros were missing early on and have since landed, but the long tail of
operators and motions has never been audited against this curriculum. Before writing
Module 3, diff the planned concept list against Zed's default vim keymap file.
