# ADR 0004 — ACP SDK and custom agent integration

**Status:** Accepted
**Date:** 2026-10-15

## Context

Zedkata requires an agent component (`zedkata-agent`) to support learning workflows that cannot be observed or validated via buffer-edit diffs:
1. Explaining student mistakes grounded in actual edit logs (`session.json`).
2. Generating customized practice targeting identified weaknesses.
3. Conducting guided walkthroughs of external editor panels and interactive features (Agent Panel, Multibuffer, Tasks, Debugger, and ACP itself).

Under REQ-03 and ADR 0002, shipped components must use Rust. Zed supports external agents through the Agent Client Protocol (ACP). We need to determine the integration architecture, SDK dependency, toolchain bounds, and communication framing between Zed and `zedkata-agent`.

## Decision

1. **Official ACP Rust SDK:**
   Use the official `agent-client-protocol` crate (`v3+` on crates.io) with the `stdio` feature enabled. Communication occurs over standard input and standard output (`stdin` / `stdout`) using JSON-RPC 2.0 protocol frames.

2. **Dedicated Binary via `agent_servers` Custom Configuration:**
   Ship `zedkata-agent` as a dedicated binary in the workspace (`crates/zedkata-agent`). It registers directly in Zed's `settings.json` under `agent_servers`:
   ```json
   {
     "agent_servers": {
       "zedkata": {
         "type": "custom",
         "command": "zedkata-agent",
         "args": ["--acp"],
         "env": {}
       }
     }
   }
   ```
   Users select `zedkata` from the model/agent selector in Zed's Agent Panel (`ctrl-?` on Linux/Windows, `cmd-?` on macOS).

3. **Strict Stdout Hygiene:**
   Standard output (`stdout`) belongs exclusively to ACP wire framing. All operational logging, diagnostics, and debugging traces must write to `stderr` or a dedicated log file (`$XDG_STATE_HOME/zedkata/agent.log`). Emitting un-framed text on `stdout` corrupts ACP JSON-RPC framing and triggers client-side agent connection termination in Zed.

4. **Async Runtime and Toolchain Boundary:**
   `zedkata-agent` runs on Tokio 1.36+ matching the existing workspace async boundary established in `zedkata-lsp`. The core library (`zedkata-core`) remains 100% synchronous, pure, and runtime-agnostic. The native toolchain is pinned to MSRV 1.85 (Rust 2021 edition), which is compatible with `agent-client-protocol`, Tokio, and modern 2026 dependencies.

5. **Subsequent Decision Handoff:**
   Model provider selection (e.g. Anthropic, OpenAI, or local Ollama), API credential resolution, and generated lesson storage paths are formally isolated into [ZK-029.2](../BACKLOG.md#task-zk-029-2).

## Consequences

- **Positives:**
  - Standardized protocol: Adheres to Zed's official external agent contract; no bespoke RPC schemas needed.
  - Zero UI hacks: Integrates directly into Zed's native GPUI Agent Panel dock without unsupported extension UI APIs.
  - Architectural consistency: Reuses workspace Tokio and logging conventions.
- **Trade-offs & Constraints:**
  - Process isolation: `zedkata-agent` runs in a separate process spawned by Zed; communication with Zedkata's TUI and LSP is mediated exclusively through the local filesystem state (`session.json`).
  - Observability boundary: Agent panel chats are GPUI interactions outside document buffers; instructional agent content must be structured as guided walkthroughs with self-reported verification criteria rather than scored drills.
