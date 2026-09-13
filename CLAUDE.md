# CLAUDE.md

Working rules for AI agents in this repository. Read this before touching anything.

---

## What this project is

A drill-based course teaching Zed's vim mode and editor features. A TUI conducts, real
Zed buffers host the exercises, a language server observes the edits, and an ACP agent
handles the conversational parts. See `README.md` for the architecture and
`docs/ARCHITECTURE.md` for the contracts between components.

## The constraint everything bends around

**Zed exposes no API for observing the buffer, the cursor, selections, or keystrokes.**
There is an open feature request for this. Treat it as permanently unavailable.

Consequence: technique verification is *inference from LSP `textDocument/didChange`
deltas*, not observation. This is a proxy. It is defeatable by a determined user. That
is acceptable for a self-directed course and is not a bug to be fixed — do not propose
architectures that assume richer access.

If you find yourself designing something that needs to know where the cursor is, stop
and re-read this section.

## Evidence discipline

Zed ships fast and your training data is stale. This repo has been burned by that
already.

- Every claim about Zed's behavior is tagged **VERIFIED** or **ASSUMED** in
  `docs/OPEN-QUESTIONS.md`. Nothing moves from ASSUMED to VERIFIED without either a
  citation to current Zed docs/source or an observation reproduced on this machine.
- Do not state an unverified Zed capability as fact in code comments, docs, or commit
  messages. Write "assumed" or check it.
- When you check something against the docs, record the date. A verification from three
  months ago is a hypothesis again.
- If a design depends on an ASSUMED item, say so at the point of dependency.

## Hard rules

1. **The TUI is not an editor.** It runs the curriculum and shows progress. If you are
   implementing text editing in the TUI, you have lost the plot — the entire point is
   that the student edits in Zed.
2. **No web UI.** No Electron, no localhost server rendering HTML, no browser step.
3. **Four processes, one state file.** Components communicate through `session.json`
   (schema in `docs/ARCHITECTURE.md`). No sockets, no IPC, no direct coupling between
   the TUI, the LSP, and the agent.
4. **No network calls from `zedkata-lsp` or `zedkata-tui`.** The agent talks to a model;
   nothing else does.
5. **Don't vendor lesson content into code.** Lessons are data files. A lesson change
   must never require a recompile.

## Stack and conventions

- Rust. MSRV 1.85 (verified 2026-09-13 against edition2024 dependencies). Edition 2021.
- `thiserror` in library crates, `anyhow` at binary boundaries.
- `clap` for CLI surfaces.
- `ratatui` + `crossterm` for the TUI.
- Tokio **is** allowed here, unlike some sibling projects — the LSP and ACP agent are
  both genuinely concurrent request/notification workloads. Keep it out of
  `zedkata-core`, which must stay sync and runtime-agnostic.
- No `unwrap()` or `expect()` in library code. Binaries may `expect()` at startup only,
  with a message that says what the operator should do about it.
- Lesson data is TOML. Session state is JSON (other tools read it).

## How to work here

- **Plan before large changes.** For anything spanning more than two files, write a
  short plan file under `docs/plans/` and get it agreed before implementing.
- **Phase 0 is a gate, not a formality.** Do not begin Phase 1 work until the
  `didChange` granularity question is answered and recorded. If the answer is bad, the
  project changes shape — see `docs/BACKLOG.md` for the fallback.
- **Throwaway spikes are throwaway.** The Phase 0 probe may be written in whatever is
  fastest (TypeScript is fine). It does not get refactored into production code; it gets
  deleted once its finding is recorded.
- **Record decisions as ADRs** under `docs/adr/`. Short. One decision per file.
- Prefer deleting an abstraction to generalizing it. This codebase is small and should
  stay that way.

## Specialist agents

`.claude/agents/` holds subagents for recurring work:

| Agent                | Use for                                                    |
|----------------------|------------------------------------------------------------|
| `zed-docs-scout`     | Verifying any claim about Zed behavior against live docs    |
| `lsp-probe`          | Protocol-level investigation and the Phase 0 spike          |
| `curriculum-author`  | Writing and reviewing lesson TOML                           |
| `rust-console`       | TUI and CLI implementation work                             |

## Things that are out of scope

- Teaching vim itself from zero. This assumes basic modal editing and teaches what is
  *different* about Zed's emulation and what Zed adds on top.
- Supporting editors other than Zed.
- Multiplayer, accounts, telemetry, leaderboards.
