# zedkata

An interactive, drill-based course for Zed's vim mode and editor features — run from a
TUI inside Zed's own terminal, with exercises performed in real Zed buffers.

**Status: v0.1.0. All core curriculum, TUI conductor, LSP observer, ACP agent, and platform builds are implemented and validated.**

See [Installation & Troubleshooting Guide](docs/installation.md) for full setup instructions.

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

Empirical verification in Phase 0 confirmed that Zed sends granular per-edit `didChange` notifications, making technique inference reliable. See [the backlog](docs/BACKLOG.md).

## Architecture

```
┌────────────────────────── Zed ───────────────────────────┐
│                                                          │
│  Editor pane                    Terminal pane / tab      │
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

## Implemented Crates & Components

| Component | Path | Role |
|---|---|---|
| `zedkata-core` | `crates/zedkata-core` | Lesson format, verification rules, session state, scoring |
| `zedkata-lsp` | `crates/zedkata-lsp` | Language server; observes `didChange`, emits inline feedback |
| `zedkata-tui` | `crates/zedkata-tui` | ratatui front end; curriculum navigation and progress |
| `zedkata-agent` | `crates/zedkata-agent` | ACP agent for the Zed agent panel; explanations & generated drills |
| `extension` | `extension/` | Zed extension (Rust → WASM) that registers the LSP and `.kata` language |

## Why Rust

ACP has official SDKs for Rust, TypeScript, Python, Kotlin, and Java — not Go. Zed
extensions must be Rust compiled to WebAssembly. See [ADR 0002](docs/adr/0002-rust-over-go.md).

## Repository map

| Location | Contents |
|---|---|
| [docs/installation.md](docs/installation.md) | Complete installation and troubleshooting guide |
| [CLAUDE.md](CLAUDE.md) | Working rules for AI agents |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Proposed components, state contract, and lesson format |
| [docs/BACKLOG.md](docs/BACKLOG.md) | Phased roadmap and task progress |
| [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) | Source-to-task coverage and requirement verification |
| [docs/OPEN-QUESTIONS.md](docs/OPEN-QUESTIONS.md) | Assumptions and recorded evidence |
| [docs/adr/](docs/adr/) | Numbered architecture decisions |
| [docs/plans/](docs/plans/) | Plans for repository changes |
| [docs/evidence/](docs/evidence/) | Verifications and calibration evidence |
| [lessons/](lessons/) | External TOML curriculum lessons and fixtures |
| [extension/](extension/) | Production Zed extension manifest and source |
| [scripts/](scripts/) | Release packaging and verification scripts |

## Getting started

1. **Install Zedkata and the Extension:**
   Follow the detailed [Installation & Troubleshooting Guide](docs/installation.md) to set up the binaries and install the Zed extension.

2. **Launch the TUI Conductor:**
   ```bash
   zedkata-tui
   ```

3. **Practice in Real Zed Buffers:**
   Select any unlocked drill (e.g. `modal-00-baseline`). Zedkata opens the working buffer in Zed at the exact line and column. Edit the buffer, observe instant LSP feedback diagnostics, and advance through the curriculum.

## License

Zedkata is dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))

at your option.

