# Agent Panel & Agent Client Protocol (ACP)

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `zed-05-agent-panel` (`[[walkthrough]] id = "agent-panel-walkthrough"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-25, REQ-26, REQ-29  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/agent_panel/`](../../lessons/fixtures/walkthroughs/agent_panel/)  
**Prerequisites:** `["zed-04-tasks-and-debugger"]`  

---

## 1. Why the Agent Panel & ACP are Guided Walkthroughs

In ZedKata, drill exercises evaluate student editing technique through Language Server Protocol (`textDocument/didChange`) incremental notifications within a single `.kata` exercise buffer.

However, **the Agent Panel and external Agent Client Protocol (ACP) workflows cannot be represented as scored single-buffer drills** for four architectural reasons:

1. **Agent Conversations Operate in Dedicated GPUI Panels:**
   - The Agent Panel is a docked graphical interface component (toggled via `ctrl-?` or `agent: toggle focus`).
   - Thread history, model selection dropdowns, slash-command pickers, and prompt input boxes reside entirely within Zed's GPUI widget tree and emit **zero LSP `textDocument/didChange` modifications** on `.kata` drill buffers.
2. **ACP Communicates via External JSON-RPC stdio Pipes:**
   - Zed interacts with external agents (such as `zedkata-agent`) across standard input and standard output pipes using the Agent Client Protocol (ACP).
   - Protocol exchanges (handshake `initialize`, prompt delivery, tool calls, and plan reviews) take place on the IPC channel, completely outside document text editing streams.
3. **Absence of Deterministic Goal States:**
   - Standard drills evaluate a single start buffer reaching an exact deterministic goal buffer within a strict Par edit budget.
   - Conversational AI responses are non-deterministic, varying across model providers, temperatures, and system prompts. Furthermore, agent tool executions can modify multiple files or execute shell commands across a repository.
4. **No External Inspection Hooks for Assistant State:**
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed exposes no external API or extension hooks to inspect assistant panel focus, active prompt text, message thread history, or tool approval decisions.

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*) and **REQ-29**, Agent Panel and ACP workflows are formally classified as an **unscored guided walkthrough** with concrete reproducible workspace fixtures, verified step-by-step recipes, and self-reported verification criteria.

---

## 2. Zed Agent Panel Architecture & Verified Semantics

Zed features an integrated conversational assistant designed for code exploration, context-aware Q&A, and autonomous project editing.

### 2.1 Access and Panel Navigation

| Action / Intent | Default Shortcut | macOS Shortcut | Zed Action Identifier |
|:---|:---:|:---:|:---|
| **Toggle Agent Panel** | `ctrl-?` | `cmd-?` | `agent::ToggleFocus` |
| **New Conversation Thread** | `ctrl-n` (in panel) | `cmd-n` (in panel) | `agent::NewThread` |
| **Focus Editor from Panel** | `<escape>` / `ctrl-w` | `<escape>` / `cmd-w` | `pane::FocusActive` |
| **Command Palette Search** | `ctrl-shift-p` | `cmd-shift-p` | `command_palette::Toggle` |

- **Dock Placement:** The Agent Panel typically docks to the right-hand panel rail alongside Collaboration/Chat. It can be dragged to the bottom dock or left dock as desired.
- **Thread History:** Previous conversations are preserved per workspace and can be resumed from the panel header dropdown.

### 2.2 Context Referencing (`@` Mentions)

Zed allows attaching workspace context directly to prompts using the `@` mention trigger:
- **`@file <path>`**: Attaches the full contents of a file (e.g. `@file src/stats.rs`).
- **`@symbol <name>`**: Attaches the definition and docstrings of a specific function, struct, or type across the worktree.
- **`@rules`**: Attaches project instructions and style guidelines (e.g. `AGENTS.md` or `.rules`).
- **Active Editor Selection**: If code is selected in the active editor buffer before focusing the agent panel, Zed automatically highlights the selection as included context.

### 2.3 Proposed Diffs and Tool Approvals

When an agent suggests file modifications:
- Zed renders a visual diff directly within the chat thread or in an excerpt buffer.
- The student can inspect the exact line insertions and deletions.
- Actions:
  - **Keep (`Apply` / `Accept`)**: Writes the proposed diff directly to the target file on disk.
  - **Discard (`Reject`)**: Clears the proposed change without touching the file.

---

## 3. ACP (Agent Client Protocol) Architecture

The **Agent Client Protocol (ACP)** is Zed's standard protocol for connecting external AI agents to the editor over standard I/O (`stdin`/`stdout`).

### 3.1 Registration in `settings.json`

Custom ACP agents are declared in Zed's settings (`~/.config/zed/settings.json` on macOS/Linux, `%APPDATA%\Zed\settings.json` on Windows, or workspace-local `.zed/settings.json`):

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

Once configured, the custom agent appears as an option in the Agent Panel model selector dropdown.

### 3.2 Stdout Framing Hygiene Invariant

Because ACP uses `stdin` and `stdout` for JSON-RPC 2.0 communication:
- **`stdout` Invariant:** Standard output must contain **strictly valid JSON-RPC 2.0 frames**.
- **`stderr` Invariant:** Extraneous debug printouts, panics, or tracing logs sent to `stdout` corrupt the protocol stream and cause Zed to immediately terminate the agent connection.
- **Remedy:** All diagnostics, informational logs, and error traces must write exclusively to `stderr` or dedicated log files (e.g. `$XDG_STATE_HOME/zedkata/agent.log`).

### 3.3 Provider Support Hierarchy ([ADR 0005](../adr/0005-model-providers-and-credentials.md))

ZedKata's agent architecture supports three operational tiers:
1. **Tier 1 — Local / Zero-Cost (Default):** LM Studio (`http://localhost:1234/v1`) or Ollama (`http://localhost:11434/v1`). Enables offline drills without API costs or subscription requirements.
2. **Tier 2 — Big Three Cloud Providers:** Anthropic (`claude-3-5-sonnet`/`haiku`), OpenAI (`gpt-4o`/`mini`), and Google Gemini (`gemini-2.0-flash`).
3. **Tier 3 — Personal Subscription Auth:** ChatGPT Plus/Team session tokens.

### 3.4 Bounded Context Contract ([REQ-27](../../docs/REQUIREMENTS.md#L65))

To eliminate token runaway and prevent hallucinations, `zedkata-agent` extracts attempt context from `session.json`:
- Active lesson ID, task description, par edits budget, and forbidden techniques.
- Recent edit stream bounded strictly to the **last 20 operations or 2 KB diff**.
- Zero credentials or API keys are ever retained in session logs or repository storage.

---

## 4. Verified Walkthrough Recipes

Follow these recipes using the sample fixture workspace at [`lessons/fixtures/walkthroughs/agent_panel/`](../../lessons/fixtures/walkthroughs/agent_panel/).

```bash
# Launch Zed in the sample fixture workspace
zed lessons/fixtures/walkthroughs/agent_panel
```

---

### Recipe 1: Toggling and Navigating the Agent Panel

**Goal:** Open the Agent Panel, inspect its layout, and transition focus smoothly between the editor and assistant.

1. Press `ctrl-?` (Windows/Linux) or `cmd-?` (macOS). Alternatively, open the command palette (`ctrl-shift-p`) and run `agent: toggle focus`.
2. Notice the panel opening in the right dock:
   - Header bar with thread history and agent selector.
   - Main message thread viewport.
   - Multi-line prompt input box at the bottom.
3. Press `ctrl-n` (or click `+ New Thread`) to ensure a fresh, clean conversation thread.
4. Press `escape` or `ctrl-w` to return focus to the code editor.
5. Press `ctrl-?` again to toggle focus back to the prompt input.

*Self-Check:* You can switch back and forth between writing code and chatting with the assistant without using the mouse.

---

### Recipe 2: Configuring Custom ACP Agents in `settings.json`

**Goal:** Understand how external custom agents are registered and verify that Zed discovers them.

1. Inspect the workspace settings file `.zed/settings.json`:
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
       },
       "mock-agent": {
         "type": "custom",
         "command": "python",
         "args": ["scripts/mock_acp_agent.py"],
         "env": {
           "PYTHONUNBUFFERED": "1"
         }
       }
     }
   }
   ```
2. In the Agent Panel header, click the model/agent selector dropdown.
3. Observe the custom entries:
   - Any configured custom servers (e.g. `mock-agent` or `zedkata`) appear alongside standard model providers.
4. Select `mock-agent` (or `zedkata`).
5. Notice that Zed spawns the child process in the background and initiates the ACP JSON-RPC handshake over stdio.

*Self-Check:* Custom agents configured in `agent_servers` populate the agent picker dropdown.

---

### Recipe 3: Context Mentions and Thread Invocations

**Goal:** Attach specific workspace files to prompts using `@` mentions and query the model.

1. Open `src/main.rs` in the editor.
2. Focus the Agent Panel (`ctrl-?`).
3. Type `@file` into the prompt input box. Notice that Zed opens a fuzzy file completion popup.
4. Type `stats` and select `src/stats.rs`. Zed inserts `@file src/stats.rs` into your prompt.
5. Complete your prompt:
   > "Explain the differences in return types between `calculate_mean` and `calculate_percentile` in `@file src/stats.rs`."
6. Press `Enter` (or `ctrl-enter` depending on your keybindings) to send the message.
7. Observe the model's explanation referencing the Option-wrapped mean versus the panicking percentile implementation.

*Self-Check:* File context was attached via `@file` without manually copying and pasting code.

---

### Recipe 4: Reviewing Multi-File Diffs and Tool Actions

**Goal:** Prompt the agent to refactor code and review the generated diff before accepting.

1. Focus the Agent Panel (`ctrl-?`).
2. Type the following prompt:
   > "Refactor `calculate_percentile` in `@file src/stats.rs` so that it returns `Option<f64>` instead of asserting on an empty slice. Update `src/main.rs` to handle the `Option` return safely."
3. Send the prompt.
4. Watch the agent generate proposed modifications.
5. In the message thread, Zed renders an interactive diff view:
   - Red highlight for removed lines (e.g. `assert!(!values.is_empty())`).
   - Green highlight for inserted lines (e.g. `if values.is_empty() { return None; }` and `Some(...)`).
6. Click **Keep** (or press the accept shortcut) to apply the refactor to `src/stats.rs`.
7. Switch to the editor (`escape`) and verify that `src/stats.rs` now contains the applied changes.
8. Run the tests using the command palette or terminal:
   ```bash
   cargo test
   ```

*Self-Check:* You inspected an agent-generated diff and selectively applied it to your project workspace.

---

### Recipe 5: Verifying Connection Health and Stdout Framing Hygiene

**Goal:** Diagnose connection issues and verify stdio logging rules.

1. Inspect `scripts/mock_acp_agent.py`:
   - Note that all `log_debug` messages are written to `sys.stderr`.
   - Note that only `json.dumps(...)` responses are written to `sys.stdout`.
2. Understand what happens if framing is violated:
   - If an agent writes `println!("debugging something")` to standard output, Zed's JSON-RPC parser encounters invalid JSON tokens and drops the connection with a protocol error.
3. If an agent fails to connect or crashes:
   - Check Zed's background log: Command palette -> `zed: open log`.
   - Look for `agent_servers` spawn errors or JSON-RPC parse exceptions.
   - For `zedkata-agent`, check the dedicated log at `$XDG_STATE_HOME/zedkata/agent.log` (Unix) or `%LOCALAPPDATA%\zedkata\agent.log` (Windows).
4. Verify graceful degraded mode:
   - If no LLM endpoint or API key is detected, `zedkata-agent` operates in degraded mode, providing static drill hints and forbidden rule explanations without remote API calls.

*Self-Check:* You understand why stdio framing hygiene is required and where to find diagnostics when an agent connection fails.

---

## 5. Self-Reported Completion Checklist

Because Agent Panel operations and ACP stdio messages cannot be detected by Language Server Protocol observer buffers, mastery is self-reported. Confirm that you can:

- [ ] Open and close the Agent Panel using `ctrl-?` (or `cmd-?`) and navigate back to the editor via keyboard.
- [ ] Configure custom ACP agents in `settings.json` under `agent_servers`.
- [ ] Reference workspace files using `@file` mentions to supply targeted context.
- [ ] Review agent-generated diffs and choose between accepting (`Keep`) or discarding changes.
- [ ] Diagnose agent connection issues by checking stderr output and Zed logs.
