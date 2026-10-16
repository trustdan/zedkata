# ZK-029.1 — ACP SDK & Configuration Compatibility Evidence

- **Task:** [ZK-029.1](../BACKLOG.md#task-zk-029-1) (Verify ACP SDK and configuration compatibility)
- **Date:** 2026-10-15
- **Author:** Zedkata Team
- **Status:** Verified

---

## 1. Executive Summary

This audit validates the external Agent Client Protocol (ACP) integration mechanics for Zedkata (`zedkata-agent`), rechecking [OQ-07](../OPEN-QUESTIONS.md#oq-07-custom-acp-agents-are-a-supported-install-path--verified) and [OQ-09](../OPEN-QUESTIONS.md#oq-09-acp-rust-sdk-maturity--verified) against current primary sources.

### Key Conclusions:
1. **Zed Custom Agent Support (OQ-07):** Zed officially supports external custom agents running over stdio via the `agent_servers` block in `settings.json` with `"type": "custom"`. Agents appear automatically in Zed's Agent Panel dropdown (`ctrl-?`).
2. **Official Rust SDK (OQ-09):** The `agent-client-protocol` crate (v3+ on crates.io) provides standard `Agent` / `Client` traits and native `stdio` framing.
3. **Toolchain & MSRV:** `agent-client-protocol` builds cleanly on Rust 1.85 (the workspace MSRV established in ZK-007.1), using Tokio 1.36+ for async execution. The obsolete Rust 1.75 assumption is formally retired.
4. **Output Framing Hygiene:** Standard output (`stdout`) is strictly reserved for JSON-RPC 2.0 protocol frames. All diagnostic logging must go to `stderr` or local log files.
5. **Next Steps Unblocked:** Directly unblocks [ZK-027.5](../BACKLOG.md#task-zk-027-5) (authoring Agent Panel & ACP walkthrough content) and [ZK-029.2](../BACKLOG.md#task-zk-029-2) (provider and credential configuration).

---

## 2. Zed Configuration & Invocation Contract (OQ-07 Recheck)

### Primary Sources
- Zed Documentation: `https://zed.dev/docs/assistant/agent-client-protocol`
- Zed Documentation: `https://zed.dev/docs/assistant/external-agents`
- Zed Settings Schema: `agent_servers` dictionary configuration

### Configuration Format
Custom agents are registered in Zed's user settings (`~/.config/zed/settings.json` on macOS/Linux, `%APPDATA%\Zed\settings.json` on Windows):

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

### UI Integration Points
1. **Agent Panel Access:** Toggled via `ctrl-?` (Windows/Linux) or `cmd-?` (macOS), or command palette `agent: toggle focus`.
2. **Agent Selector:** Custom agents declared under `agent_servers` appear as selectable options in the agent dropdown menu alongside built-in models and Zed AI.
3. **Execution Lifecycle:**
   - Zed spawns `command` with `args` as a child process when an agent conversation thread is opened.
   - Zed handles process termination when the session ends or when Zed exits.
   - Communication operates across bidirectional standard pipes (`stdin` and `stdout`).

---

## 3. ACP Rust SDK Audit (OQ-09 Recheck)

### Primary Sources
- Crates.io: `https://crates.io/crates/agent-client-protocol` (v3+)
- Docs.rs: `https://docs.rs/agent-client-protocol`
- GitHub: `Agent Client Protocol` organization repository

### Crate Architecture & Features
* **Core Crate:** `agent-client-protocol`
* **Essential Feature Flags:** `stdio` (enables Tokio-based asynchronous standard I/O framing adapters).
* **Protocol Framing:** JSON-RPC 2.0 messages serialized over `stdin` / `stdout`.
* **Async Runtime:** Tokio 1.36+ is required for the stdio transport, perfectly matching `zedkata`'s existing Tokio dependency in `crates/zedkata-lsp`.
* **Synchronous Core Isolation:** As mandated by ADR 0002 and REQ-03, `zedkata-core` remains synchronous and runtime-agnostic. Async handling is isolated strictly to `crates/zedkata-agent`.

---

## 4. Stdout Framing & Logging Invariants

Because ACP exchanges protocol packets via stdio:
- **`stdout` Invariant:** Standard output must contain ONLY valid, framed JSON-RPC 2.0 payloads.
- **`stderr` Invariant:** Unformatted debug messages, tracing logs, or panics printed to stdout will cause deserialization failures in Zed and immediately crash the agent connection.
- **Action:**
  - `zedkata-agent` must configure any tracing/logging subscribers (such as `tracing-subscriber`) to write exclusively to `stderr` or to `$XDG_STATE_HOME/zedkata/agent.log`.

---

## 5. Toolchain & MSRV Resolution

In `docs/REQUIREMENTS.md`, the unresolved decision regarding Rust 1.75 vs modern dependencies is addressed:
- ZK-007.1 established MSRV 1.85 because dependencies utilizing `edition2024` and modern Tokio/Ratatui crates require 1.85+.
- `agent-client-protocol` v3+ requires Rust 1.85+ and compiles seamlessly with Rust 2021/2024 editions.
- **Verdict:** Native MSRV 1.85 is formally reaffirmed. Rust 1.75 is permanently superseded.

---

## 6. Downstream Task Handoff

| Task | Status | Role |
|------|--------|------|
| **[ZK-027.5](../BACKLOG.md#task-zk-027-5)** | **Ready** | Author Agent Panel and ACP walkthrough guide & lesson (`zed-05-agent-panel.toml`). Walks the student through opening the panel (`ctrl-?`), configuring `agent_servers`, and conversing with an ACP agent. |
| **[ZK-029.2](../BACKLOG.md#task-zk-029-2)** | **Ready** | Choose LLM provider (Anthropic / OpenAI / local Ollama), configure credentials/env vars, and specify bounded context extraction from `session.json`. |
| **[ZK-030.1](../BACKLOG.md#task-zk-030-1)** | Blocked on 029.2 | Scaffold `crates/zedkata-agent` with `agent-client-protocol` crate and protocol unit tests. |
