# ZK-030.1 — ACP Agent Protocol Scaffolding Evidence

- **Task:** [ZK-030.1](../BACKLOG.md#task-zk-030-1) (Scaffold the ACP agent protocol)
- **Date:** 2026-10-15
- **Status:** Complete & Verified
- **Requirements:** [REQ-26](../REQUIREMENTS.md#L64)

---

## 1. Summary

This artifact records the successful completion of task ZK-030.1. We scaffolded `crates/zedkata-agent` using the official `agent-client-protocol` SDK (v3.3.0) and established the complete Agent Client Protocol (ACP) communication lifecycle over asynchronous byte streams and standard I/O pipes.

The crate satisfies all acceptance criteria:
1. Builds on the pinned workspace MSRV (Rust 1.85+) and Tokio 1.36+ async runtime.
2. CLI binary (`zedkata-agent`) supports `--acp` for launching over stdio and `--help` / `--version` flags.
3. Implements protocol request and notification handlers:
   - `initialize`: Protocol negotiation, agent identity declaration (`zedkata-agent`), capabilities handshake.
   - `session/new`: Unique session instantiation, active registry bookkeeping, returning `SessionId`.
   - `session/prompt`: Turn processing, emitting `session/update` (`agent_message_chunk`) notifications, responding with `stopReason: "end_turn"`.
   - `session/close`: Clean session removal from registry.
4. Unsupported request resilience: Unknown JSON-RPC methods receive error code `-32601 Method not found` while keeping the connection active and healthy.
5. Strict stdout framing hygiene: Standard output is exclusively reserved for valid JSON-RPC 2.0 frames; all diagnostics, warnings, and tracing messages are routed strictly to `stderr`.
6. Clean lifecycle shutdown: Stream closure/EOF terminates the server gracefully with `Ok(())`.

---

## 2. Implemented Architecture

### Crate Structure
```
crates/zedkata-agent/
├── Cargo.toml
├── src/
│   ├── lib.rs        # Public API exports
│   ├── logging.rs    # Strict stderr tracing initialization
│   ├── main.rs       # CLI entrypoint (--acp flag)
│   └── server.rs     # AgentServer implementation & ACP handlers
└── tests/
    └── protocol_lifecycle_tests.rs # In-memory integration lifecycle test suite
```

### Handler Lifecycle Matrix

| Method | Protocol Direction | Behavior | Response / Side Effect |
|--------|---------------------|----------|------------------------|
| `initialize` | Client → Agent | Version check | `InitializeResponse` with protocol version `1` and `zedkata-agent` implementation metadata |
| `session/new` | Client → Agent | Creates active session | Assigns unique ID (`sess-<uuid>`), registers in `AgentServer`, returns `NewSessionResponse` |
| `session/prompt` | Client → Agent | Executes conversation turn | Emits `session/update` notification with `agent_message_chunk` text content; returns `PromptResponse` with `stopReason: "end_turn"` |
| `session/close` | Client → Agent | Closes conversation | Deregisters session from `AgentServer`, returns `CloseSessionResponse` |
| `<unknown>` | Client → Agent | Method dispatch fallback | Returns JSON-RPC error `-32601 Method not found`, retains transport connection |
| `<EOF>` | Client → Agent | Transport hangup | Connection driver exits loop cleanly returning `Ok(())` |

---

## 3. Test Verification

### Protocol Lifecycle Suite (`crates/zedkata-agent/tests/protocol_lifecycle_tests.rs`)
- `test_initialize_lifecycle`: Handshake negotiation, agent metadata assertions, clean EOF termination.
- `test_session_lifecycle_and_prompt_turn`: End-to-end multi-turn scenario (`initialize` → `session/new` → `session/prompt` → `session/close`), verifying session update notification streaming and session registry updates.
- `test_unsupported_request_returns_method_not_found`: Negative testing with bogus JSON-RPC method, verifying `-32601` error response and subsequent connection recovery.
- `test_stdout_framing_hygiene`: Batch of mixed requests asserting every single emitted byte sequence parses as a valid, framed `{"jsonrpc": "2.0", ...}` frame with no stray text.

```
running 4 tests
test test_initialize_lifecycle ... ok
test test_unsupported_request_returns_method_not_found ... ok
test test_stdout_framing_hygiene ... ok
test test_session_lifecycle_and_prompt_turn ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Full Workspace Test Suite
All 170 workspace tests pass:
- `zedkata-core`: 81 tests passed
- `zedkata-lsp`: 33 tests passed
- `zedkata-tui`: 52 tests passed
- `zedkata-agent`: 4 tests passed
- Rustfmt and Clippy: Zero warnings or formatting diffs.

---

## 4. Next Task Handoff

ZK-030.1 directly unblocks:
- **[ZK-030.2](../BACKLOG.md#task-zk-030-2)** — Connect an agent session in Zed: Connect live Zed Agent Panel (`ctrl-?`) to `zedkata-agent --acp`, verifying real-world integration under Zed 1.19.2.
- **[ZK-031.1](../BACKLOG.md#task-zk-031-1)** — Build bounded last-attempt context: Read active drill state from `session.json` and feed bounded edit history into model context.
