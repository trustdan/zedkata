# Kata Agent Panel & ACP Demo Workspace

This workspace demonstrates how Zed integrates with conversational AI models and external Agent Client Protocol (ACP) agents.

## Workspace Layout

- `.zed/settings.json` — Pre-configures `agent_servers` for both `zedkata-agent` and a lightweight mock ACP agent script.
- `src/main.rs` — An entry-point command line tool calculating dataset statistics.
- `src/stats.rs` — Mathematical statistical routines (`mean`, `median`, `percentile`) with deliberate edge-case opportunities for agent refactoring.
- `tests/stats_test.rs` — Integration test suite validating statistical outcomes.
- `scripts/mock_acp_agent.py` — A standalone stdio JSON-RPC script demonstrating ACP framing and handshake mechanics without external API dependencies.

## Key Exercises & Demonstrations

1. **Toggle Agent Panel:** Press `ctrl-?` (Windows/Linux) or `cmd-?` (macOS), or run `agent: toggle focus`.
2. **Select Agent:** Use the agent dropdown selector at the top of the panel to select your configured custom agent (`zedkata` or `mock-agent`) or local model (LM Studio / Ollama).
3. **Context Mention:** In the prompt editor, type `@file src/stats.rs` to attach the source context.
4. **Prompt Refactoring:** Ask:
   > "Refactor `calculate_percentile` in `@file src/stats.rs` to return `Result<f64, &'static str>` and guard against empty inputs."
5. **Review Diffs:** Inspect the proposed multi-file or inline diff, review modifications, and accept or reject.
