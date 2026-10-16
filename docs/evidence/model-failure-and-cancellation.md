# ZK-031.3 — Model Failure and Cancellation Handling Evidence

- **Task:** [ZK-031.3](../BACKLOG.md#task-zk-031-3) (Handle model failure and cancellation)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-27](../REQUIREMENTS.md#L65)
- **Architecture Reference:** [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-agent/src/config.rs` (`timeout_seconds` resolution across environment, `config.toml`, and defaults)
  - `crates/zedkata-agent/src/provider.rs` (`ProviderError` failure taxonomy, user-facing error formatting, async cancellation/timeout execution in `ConfiguredProvider`)
  - `crates/zedkata-agent/src/server.rs` (ACP server concurrency with `tokio::spawn`, `session/cancel` notification routing, clean shutdown via `session/close`)
  - `crates/zedkata-agent/src/main.rs` (CLI error presentation with student reassurance)
  - `crates/zedkata-agent/tests/failure_and_cancellation_tests.rs` (7 automated integration tests)

---

## 1. Executive Summary

Task **ZK-031.3** delivers robust failure handling and cancellation mechanisms for `zedkata-agent` under [REQ-27](../REQUIREMENTS.md#L65) and [ADR 0005](../adr/0005-model-providers-and-credentials.md).

All acceptance criteria are satisfied:
1. **Model Timeout Handling:**
   - Provider calls are bounded by a configurable timeout (`timeout_seconds`, default 15s; overrideable via `ZEDKATA_TIMEOUT_SECONDS` or `config.toml`).
   - If the provider does not respond within the allocated budget, the in-flight future is immediately dropped, preventing lingering tasks or hung worker threads.
   - Produces a clear, student-facing message with actionable troubleshooting tips (e.g. checking local model runners, internet connectivity, or configuring offline degraded mode) while explicitly reassuring the student that their session and progress are intact.
2. **Provider Rejection Handling:**
   - Explicitly handles HTTP 4xx/5xx status codes, rate limits (`HTTP 429`), authentication errors (`HTTP 401`/`403`), and connection rejections without looping or unbounded retries.
   - Surfaces targeted guidance (e.g. invalid API key, quota exhaustion, local runner not started).
3. **Malformed Model Response Handling:**
   - Detects unparseable, truncated, or structurally malformed model outputs gracefully without panicking or corrupting state.
4. **ACP Cancellation (`session/cancel`):**
   - Implements `CancelNotification` (`session/cancel`) on `AgentServer`.
   - Spawns turn processing in detached Tokio tasks so that the ACP transport loop immediately receives notifications while an explanation is computing.
   - Upon receiving `session/cancel`, the in-flight work is aborted via `tokio::sync::watch` cancellation channels, and the server responds to `session/prompt` with `StopReason::Cancelled` according to the ACP specification.
   - No lingering background tasks, orphaned handles, or memory leaks remain.
   - The session remains fully registered and available for subsequent prompts.
5. **Session and Progress Invariance:**
   - Across all error and cancellation scenarios, `session.json` and student progress files remain completely untouched and uncorrupted.

---

## 2. Error Taxonomy and User-Facing Formatting

The `ProviderError` enum is expanded to explicitly model all failure and cancellation modes:

| Error Variant | Root Cause | Student-Facing Guidance | State Impact |
|---|---|---|---|
| `Timeout { seconds, message }` | Provider took longer than `timeout_seconds` | Explains timeout budget; advises checking local model server or switching to `degraded` mode | Intact |
| `Rejection { status_code, message }` | HTTP 4xx/5xx, invalid API key, rate limit | Advises verifying API credentials or waiting for quota reset | Intact |
| `MalformedResponse(message)` | Model returned unparseable or truncated payload | Advises retrying prompt or adjusting model temperature/context | Intact |
| `Cancelled(message)` | ACP client sent `session/cancel` | Confirms request cancellation; responds with `StopReason::Cancelled` | Intact |
| `MissingSession(message)` | No active drill in `session.json` | Guides student to launch exercise in Zed | Intact |
| `CorruptSession(message)` | Bad JSON on disk | Advises starting fresh drill to regenerate session | Intact |

All error variants format into structured, calming Markdown via `ProviderError::to_user_message()` ensuring student confidence and operational transparency.

---

## 3. Automated Verification

The test suite in `crates/zedkata-agent/tests/failure_and_cancellation_tests.rs` covers all acceptance criteria:

```text
running 7 tests
test test_malformed_model_response_produces_clear_message_and_intact_session ... ok
test test_provider_rejection_produces_clear_message_and_intact_session ... ok
test test_provider_timeout_produces_clear_message_and_intact_session ... ok
test test_acp_prompt_turn_provider_rejection_returns_clean_user_chunk ... ok
test test_acp_prompt_turn_timeout_returns_end_turn_with_clean_user_chunk ... ok
test test_acp_cancellation_terminates_turn_with_stop_reason_cancelled ... ok
test test_acp_session_close_during_in_flight_prompt_cancels_cleanly ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
```

### Full Workspace Test Verification
```text
test result: ok. 223 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Formatting & Linting Verification
- `cargo fmt --check`: Passed (code 0).
- `cargo clippy --workspace --all-targets`: Passed (code 0, 0 warnings).

---

## 4. Acceptance Criteria Checklist

- [x] **Timeout:** Timed-out requests drop in-flight tasks, produce clear user-facing messages with troubleshooting advice, and leave session and progress intact.
- [x] **Provider Rejection:** Status codes and network rejection errors return actionable guidance with zero unbounded retries.
- [x] **Malformed Response:** Unparseable payloads produce clean error chunks without panics or data corruption.
- [x] **ACP Cancellation:** Client `session/cancel` notifications abort in-flight turns cleanly, emit `StopReason::Cancelled`, and leave the session ready for future prompts.
- [x] **No lingering tasks:** Work futures terminate immediately upon cancellation or timeout; closing sessions cleans up all associated channels and handles.
- [x] **Platform Installation:** Installed `zedkata-agent` v0.1.0 to PATH (`~/.cargo/bin/zedkata-agent.exe`).

Task **ZK-031.3** is verified and complete. Unblocks Epic **ZK-032** (Generated practice).
