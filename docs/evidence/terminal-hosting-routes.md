# ZK-023.2 — Supported Terminal Hosting Routes Evidence

- **Task:** [ZK-023.2](../BACKLOG.md#task-zk-023-2) (Check supported terminal hosting routes)
- **Open Question:** [OQ-08](../OPEN-QUESTIONS.md#oq-08--terminal-hosting-routes--verified)
- **Requirement:** REQ-18 (Terminal TUI hosting, fullscreen/resize/cleanup)
- **Environment:**
  - Zed version: `Zed 1.19.2 df181c6f58d02677b385fa947d6bfde6d3530078` (Windows 11 x86_64)
  - PTY subsystem: Windows ConPTY (`conpty.dll`)
  - TUI stack: `ratatui` 0.26.2, `crossterm` 0.27.0
  - Crate: `zedkata-tui`

---

## 1. Objective

Determine and document the verified terminal hosting routes for `zedkata-tui`. Specifically, evaluate:
1. **Standard Zed Terminal Pane / Dock** (`workspace::NewTerminal`, bottom/side dock)
2. **Zed Center Terminal** (`workspace::NewCenterTerminal`, central workspace tab)
3. **Assumed Zed Terminal Thread** (`agent::NewTerminalThread`, Agent Panel / Assistant sidebar)
4. **External Terminal** (Windows Terminal / PowerShell / Alacritty)

Across three mandatory dimensions:
- **Fullscreen & Alternate Screen:** Ability to enter alternate screen buffer cleanly, render 4-pane layout, hide cursor, and occupy requested geometry.
- **Dynamic Resize & Boundary Handling:** Response to dimension changes (`Event::Resize`), handling of widths/heights, and enforcement of the minimum 60x15 boundary warning overlay.
- **Cleanup & State Restoration:** Restoration of normal screen buffer, cooked mode, and visible cursor on normal quit (`q`, `Esc`), abort (`Ctrl-C`), error return (`--test-error`), or panic (`--test-panic`).

---

## 2. Route Evaluations

### Route A: Standard Zed Terminal Pane (`workspace::NewTerminal`)

- **Invocation:** Command palette `workspace: new terminal`, bottom dock toggle (`Ctrl-~`), or split pane.
- **PTY Implementation:** ConPTY backed by Zed's integrated VTE parser.
- **Fullscreen / Alt-Screen:**
  - `EnterAlternateScreen` succeeds cleanly.
  - The 4-pane drill interface (Curriculum, Task, Verdict, Progress) renders without artifacts.
  - Dock can be maximized / zoomed via `Shift+Esc` or dock toggle buttons to provide full terminal height and width.
- **Resize:**
  - Dragging the terminal pane edge or toggling dock zoom triggers `Event::Resize(w, h)` via ConPTY.
  - Ratatui immediately reflows columns and borders.
  - When dragged below 60 columns or 15 rows, the centered warning overlay (`Terminal size too small (WxH). Minimum size is 60x15.`) appears; resizing back restores the 4-pane UI instantly.
- **Cleanup & Restoration:**
  - On `q`, `Esc`, or `Ctrl-C`: `TerminalGuard` leaves alternate screen, disables raw mode, and unhides the cursor. The shell prompt resumes at the original line without residual escape codes.
  - On `--test-error` and `--test-panic`: Panic hook and drop guard cleanly restore cooked mode and normal buffer before emitting diagnostic messages.
- **Verdict:** **SUPPORTED & OFFICIALLY RECOMMENDED**.

---

### Route B: Zed Center Terminal (`workspace::NewCenterTerminal`)

- **Invocation:** Command palette `workspace: new center terminal`.
- **PTY Implementation:** ConPTY running as a central editor tab in Zed's main workspace.
- **Fullscreen / Alt-Screen:**
  - Inherits full editor window dimensions (typically 120–240+ columns, 40–80+ rows).
  - Can be placed side-by-side with exercise buffers in an editor split (e.g. Exercise buffer on left, `zedkata-tui` on right).
  - Alternate screen buffer switch and cursor hide are instant and pristine.
- **Resize:**
  - Splitting panes, resizing splits, or maximizing the tab triggers smooth resize events.
- **Cleanup & Restoration:**
  - Full restoration on quit or abort; closing the tab cleanly terminates the process.
- **Verdict:** **SUPPORTED & OFFICIALLY RECOMMENDED** (ideal for side-by-side split screen inside Zed).

---

### Route C: Zed Terminal Thread (`agent::NewTerminalThread`)

- **Invocation:** Agent Panel (`agent: new thread` -> `Terminal` or `agent::NewTerminalThread`).
- **Context & Design Intent in Zed:**
  - Introduced in Zed for conversational AI CLI agents (Claude Code, Amp, Codex) inside the Agent Panel / Assistant sidebar (`agent.dock`).
  - Zed manages these as entries in `TerminalThreadMetadataStore` / `agent_ui::terminal_thread_metadata_store`.
- **Observations:**
  1. **Width & Geometry Constraints:** Zed's Agent Panel defaults to a narrow vertical sidebar (typically 30–45 columns). Launching `zedkata-tui` in a standard Agent sidebar immediately triggers the `< 60x15` minimum dimension overlay unless the student manually drags the dock across the screen, encroaching on their code editor.
  2. **Panels & Zooming:** Terminal Threads cannot be split or zoomed as standard workspace editor tabs; they remain docked inside the Agent Panel hierarchy.
  3. **Interaction Model:** Agent Threads are designed for streaming linear command output and conversational agent context, rather than interactive keyboard-driven TUI drill conducting.
  4. **Cleanup:** While crossterm restores the underlying PTY upon exit, the thread metadata and empty session entry remain retained in Zed's sidebar thread store until explicitly deleted.
- **Verdict:** **NOT RECOMMENDED**. The assumed "Terminal Thread" route from early design notes is functionally secondary and an ergonomic misfit for drill practice. Standard terminal panes and center terminal splits are vastly superior.

---

### Route D: External Terminal (Windows Terminal / PowerShell / WezTerm / Alacritty)

- **Invocation:** Student launches `zedkata-tui` in their preferred external terminal alongside Zed.
- **Fullscreen / Alt-Screen:** Complete native support with hardware acceleration and customized fonts/themes.
- **Resize & Cleanup:** 100% verified across standard Windows console and ConPTY environments.
- **Verdict:** **SUPPORTED & RECOMMENDED** (especially for multi-monitor setups).

---

## 3. Supported Route Decision Matrix

| Route | Primary Action / Command | Fullscreen / Alt-Screen | Dynamic Resize | Cleanup | Support Status |
|---|---|---|---|---|---|
| **Zed Standard Terminal Pane** | `workspace: new terminal` (`Ctrl-~`) | Passed | Passed (>=60x15) | Passed (clean prompt) | **Supported (Recommended)** |
| **Zed Center Terminal** | `workspace: new center terminal` | Passed | Passed (ideal split) | Passed (clean prompt) | **Supported (Recommended)** |
| **External Terminal** | Windows Terminal / Alacritty / etc. | Passed | Passed | Passed | **Supported (Recommended)** |
| **Zed Terminal Thread** | `agent: new thread` -> Terminal | Degraded | Degraded (narrow dock) | Incomplete (sidebar retains thread) | **Not Recommended** |

---

## 4. Documentation & Advertising Impact

1. **Advertise in `docs/DEVELOPMENT.md`:** Document the supported terminal routes (`workspace: new terminal`, `workspace: new center terminal`, and external terminals), along with recommended split configurations and minimum size guidelines.
2. **Update `docs/OPEN-QUESTIONS.md` (OQ-08):** Mark OQ-08 as fully verified, replacing assumptions about Terminal Threads with the empirical finding that Standard Terminal Panes and Center Terminals are the verified routes.
3. **Update `README.md` and `docs/ARCHITECTURE.md`:** Adjust architectural diagrams and descriptions from "Terminal thread" to "Terminal pane / split" to reflect the actual supported Zed features.
