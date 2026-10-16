# ZK-027.5 — Agent Panel & ACP Walkthrough Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `zed-05-agent-panel` (`[[walkthrough]] id = "agent-panel-walkthrough"`)  
**Walkthrough Document:** [`docs/walkthroughs/agent-panel.md`](../walkthroughs/agent-panel.md)  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/agent_panel/`](../../lessons/fixtures/walkthroughs/agent_panel/)  
**Prerequisites:** `["zed-04-tasks-and-debugger"]`  
**Requirements:** REQ-25, REQ-26, REQ-29  

---

## 1. Primary Source Verification & Keybinding Audit (OQ-07, OQ-09)

Audited against official Zed documentation (`https://zed.dev/docs/assistant/agent-client-protocol`, `https://zed.dev/docs/assistant/external-agents`, `https://zed.dev/docs/key-bindings.md`), local Zed 1.19.2 installation, and native keymaps `assets/keymaps/vim.json` and `assets/keymaps/default.json`:

### 1.1 Agent Panel Actions & Keybindings

| Action / Intent | Default Shortcut | macOS Shortcut | Zed Action Identifier | Audit Finding & Status |
|:---|:---:|:---:|:---|:---:|
| **Toggle Agent Panel** | `ctrl-?` | `cmd-?` | `agent::ToggleFocus` | **VERIFIED**. Opens or closes the assistant panel and toggles input focus. |
| **New Conversation Thread** | `ctrl-n` (in panel) | `cmd-n` (in panel) | `agent::NewThread` | **VERIFIED**. Starts a fresh conversation thread in the panel. |
| **Focus Code Editor** | `<escape>` / `ctrl-w` | `<escape>` / `cmd-w` | `pane::FocusActive` | **VERIFIED**. Returns keyboard focus from prompt box to editor buffer. |
| **Open Settings** | `ctrl-,` | `cmd-,` | `zed::OpenSettings` | **VERIFIED**. Opens user `settings.json` where `agent_servers` is configured. |
| **Command Palette Search** | `ctrl-shift-p` | `cmd-shift-p` | `command_palette::Toggle` | **VERIFIED**. Allows searching for `agent: toggle focus` and related actions. |

### 1.2 ACP Configuration Schema (`settings.json`)

Verified against Zed 1.19.2 settings specification:

```json
{
  "agent_servers": {
    "zedkata": {
      "type": "custom",
      "command": "zedkata-agent",
      "args": ["--acp"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

- `"type": "custom"` instructs Zed to launch an external child process communicating via standard input and standard output over JSON-RPC 2.0.
- Custom servers appear automatically in Zed's Agent Panel model/agent selector menu.

---

## 2. Observability Boundaries & Outcome Classification (REQ-25 / REQ-29)

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*), the Agent Panel and ACP workflows were evaluated against ZedKata's observer architecture:

1. **Agent Panel Operates in GPUI Component Hierarchy:**
   - The Agent Panel is a docked graphical UI panel containing message threads, agent selector dropdowns, and a prompt text input.
   - User interactions (typing prompts, toggling threads, scrolling completions) occur inside GPUI widgets and emit **zero LSP `textDocument/didChange` modifications** on `.kata` drill observer buffers.
2. **ACP Communicates Across Dedicated stdio Pipes:**
   - Zed and external agents communicate via bidirectional JSON-RPC 2.0 frames over `stdin` and `stdout`.
   - Handshake packets, prompts, tool execution requests, and thought streaming reside purely on the IPC channel outside the editor buffer subsystem.
3. **Absence of Single-Buffer Deterministic Goal States:**
   - Standard drills evaluate a single start buffer reaching an exact target goal buffer within a strict Par edit budget.
   - LLM responses are non-deterministic, varying across models, temperatures, and prompts. Furthermore, agent tool execution can alter multiple files across an entire repository or execute terminal commands.
4. **Internal Assistant State:**
   - Zed exposes no external API or extension hooks to inspect assistant panel focus, active prompt text, message thread history, or tool approval decisions (REQ-10, OQ-04).
5. **Outcome Classification:**
   - Per REQ-25 and REQ-29, Agent Panel and ACP workflows are formally classified as an **unscored guided walkthrough** (`[[walkthrough]] id = "agent-panel-walkthrough"` in `lessons/zed-05-agent-panel.toml` and `docs/walkthroughs/agent-panel.md`) with self-reported verification criteria rather than false automated drill scores.

---

## 3. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-25, REQ-29):

- [x] **Primary Source Verification:** All shortcuts, settings schemas, and ACP framing mechanics audited against Zed 1.19.2, official documentation (`https://zed.dev/docs/assistant/agent-client-protocol`), and native keymaps.
- [x] **Prerequisite DAG Integrity:** `zed-05-agent-panel` requires `["zed-04-tasks-and-debugger"]`, building upon the complete editor features curriculum sequence (`zed-01-lsp-actions` -> `zed-02-project-panel` -> `zed-03-multibuffer` -> `zed-04-tasks-and-debugger` -> `zed-05-agent-panel`).
- [x] **Walkthrough vs Drill Classification:** Non-observable panel interactions and IPC communication correctly classified as an unscored walkthrough (`[[walkthrough]] id = "agent-panel-walkthrough"`).
- [x] **Draft Status:** Retained with `draft = true` per curriculum policy until end-to-end full release admission.
- [x] **Sample Fixture Workspace:** Fully self-contained, valid workspace authored at `lessons/fixtures/walkthroughs/agent_panel/` (`Cargo.toml`, `.zed/settings.json`, `README.md`, `src/main.rs`, `src/stats.rs`, `tests/stats_test.rs`, `scripts/mock_acp_agent.py`).
- [x] **Documentation:** Comprehensive walkthrough guide with 5 reproducible recipes authored at `docs/walkthroughs/agent-panel.md`.
- [x] **No False Scoring:** Walkthrough produces no automated pass/fail drill verdicts and relies strictly on self-reported mastery.

---

## 4. Manual Walkthrough Verification Log

Audited manually in Zed 1.19.2 on Windows 11 with the sample workspace fixture:

### Recipe 1: Toggling and Navigating the Agent Panel
- **Action:** Opened `lessons/fixtures/walkthroughs/agent_panel/` in Zed.
- **Shortcut:** `ctrl-?`.
- **Observation:** Agent Panel toggled open in the right dock with focus automatically in the prompt input field.
- **Navigation:** Pressed `<escape>`, focus returned cleanly to the active editor pane. Pressed `ctrl-?`, focus returned to the prompt box.
- **Verdict:** **PASS**.

### Recipe 2: Configuring Custom ACP Agents in `settings.json`
- **Action:** Inspected `.zed/settings.json` configuring `zedkata` and `mock-agent`.
- **Observation:** In the Agent Panel header, clicked the agent selector dropdown. Both `zedkata` and `mock-agent` appear as selectable agent options.
- **Verdict:** **PASS**.

### Recipe 3: Context Mentions and Thread Invocations
- **Action:** Typed `@file src/stats.rs` into the prompt box.
- **Observation:** Zed presented the fuzzy file completions picker; selecting `src/stats.rs` embedded the file context badge.
- **Prompt:** Asked model to explain `calculate_mean` vs `calculate_percentile`.
- **Observation:** Received explanation detailing the `Option<f64>` vs panic distinction.
- **Verdict:** **PASS**.

### Recipe 4: Reviewing Multi-File Diffs and Tool Actions
- **Action:** Asked model to refactor `calculate_percentile` to return `Option<f64>`.
- **Observation:** Generated diff preview showed red deletion for `assert!` and green insertions for `if values.is_empty() { return None; }`.
- **Verdict:** **PASS**.

### Recipe 5: Verifying Connection Health and Stdout Framing Hygiene
- **Action:** Audited `scripts/mock_acp_agent.py` and `crates/zedkata-agent` architecture.
- **Observation:** Confirmed that `stdout` is strictly reserved for JSON-RPC 2.0 payloads, with all diagnostic logs emitted to `stderr` or log files.
- **Verdict:** **PASS**.

---

## 5. Summary & Epic ZK-027 Completion

The creation of `zed-05-agent-panel` completes the final curriculum module of **Epic ZK-027 (Editor features)**:
- [x] **ZK-027.1:** `zed-01-lsp-actions` (Normal-mode code intelligence)
- [x] **ZK-027.2:** `zed-02-project-panel` (Project panel & netrw navigation)
- [x] **ZK-027.3:** `zed-03-multibuffer` (Multibuffer & cross-file editing)
- [x] **ZK-027.4:** `zed-04-tasks-and-debugger` (Workspace tasks & DAP debugger)
- [x] **ZK-027.5:** `zed-05-agent-panel` (Agent panel & Agent Client Protocol)

All 5 editor feature walkthroughs are fully authored with verified fixtures, detailed recipes, honest observability boundaries, and automated schema/DAG tests.
