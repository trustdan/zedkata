# zedkata

An interactive, drill-based course for Zed's vim mode and editor features — run from a
TUI inside Zed's own terminal, with exercises performed in real Zed buffers.

**Status: pre-alpha. Nothing works yet. Phase 0 is an unresolved feasibility spike.**

---

## The idea

`vimtutor` and `vim-be-good` work because they run *inside* the editor they teach. Zed
has no equivalent plugin surface, so this takes a different shape:

- The **TUI** is the conductor — curriculum, progress, scoring. It is not an editor.
- The **exercises happen in real Zed buffers**, because the point is to build muscle
  memory in the tool you actually use.
- A **language server** attached to exercise files observes edits and reports back.
- An **ACP agent** handles the conversational parts a scripted drill can't: explaining
  why a motion did what it did, generating a fresh drill against a weak spot, and
  walking through Zed features (agent panel, multibuffer, tasks, debugger) that are
  better demonstrated than checked.

## The hard part

Zed's extension API cannot observe the buffer, the cursor, or selections. There is an
open feature request for exactly that capability; assume it never lands.

This matters because the whole pedagogical value of a vim tutor is verifying *how* you
did something. `ci"` and mouse-select-then-retype produce identical buffers. Without
keystroke access, technique has to be **inferred from LSP `textDocument/didChange`
deltas** — one word-sized deletion looks different on the wire from five backspaces.

Whether that inference is possible at all depends on a single empirical fact nobody has
written down: does Zed send granular per-edit `didChange` notifications, or does it
coalesce them? **Phase 0 exists to answer that question before any real code is
written.** See [the backlog](docs/BACKLOG.md).

## Architecture

```
┌────────────────────────── Zed ───────────────────────────┐
│                                                          │
│  Editor pane                    Terminal thread          │
│  ┌────────────────────┐         ┌────────────────────┐   │
│  │ exercise buffer    │         │ zedkata-tui        │   │
│  │  ↕ LSP             │         │  (ratatui)         │   │
│  └────────┬───────────┘         └─────────┬──────────┘   │
│           │                               │              │
│  Agent panel                              │              │
│  ┌────────────────────┐                   │              │
│  │ zedkata-agent (ACP)│                   │              │
│  └────────┬───────────┘                   │              │
└───────────┼───────────────────────────────┼──────────────┘
            │                               │
     ┌──────▼──────┐                 ┌──────▼──────┐
     │ zedkata-lsp │◄───────────────►│ session.json│
     └─────────────┘   shared state  └─────────────┘
```

Four processes, one shared state file. They do not talk to each other directly.

## Planned crates

| Crate           | Role                                                          |
|-----------------|---------------------------------------------------------------|
| `zedkata-core`  | Lesson format, verification rules, session state, scoring      |
| `zedkata-lsp`   | Language server; observes `didChange`, emits inline feedback   |
| `zedkata-tui`   | ratatui front end; curriculum navigation and progress          |
| `zedkata-agent` | ACP agent for the Zed agent panel                              |
| `zedkata-ext`   | Zed extension (Rust → WASM) that registers the LSP             |

## Why Rust

ACP has official SDKs for Rust, TypeScript, Python, Kotlin, and Java — not Go. Zed
extensions must be Rust compiled to WebAssembly. See [ADR 0002](docs/adr/0002-rust-over-go.md).

## Repository map

| Location | Contents |
|----------|----------|
| [CLAUDE.md](CLAUDE.md) | Working rules for AI agents |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Proposed components, state contract, and lesson format |
| [docs/BACKLOG.md](docs/BACKLOG.md) | Phased roadmap, starting with the feasibility gate |
| [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) | Source-to-task coverage and unresolved decisions |
| [docs/OPEN-QUESTIONS.md](docs/OPEN-QUESTIONS.md) | Assumptions and recorded evidence |
| [docs/adr/](docs/adr/) | Numbered architecture decisions |
| [docs/plans/](docs/plans/) | Plans for repository changes |
| [.claude/agents/](.claude/agents/) | Specialist agent definitions |
| [lessons/](lessons/) | Reference lesson data and content status |

## Getting started

Nothing to run yet. Start with [Phase 0 in the backlog](docs/BACKLOG.md#phase-0--feasibility-spike-gate).
