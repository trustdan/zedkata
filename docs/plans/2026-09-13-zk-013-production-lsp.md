# ZK-013 Production LSP Observer Implementation Plan

**Date:** 2026-09-13  
**Status:** Complete. All acceptance criteria met and verified.

## Scope

Implement the production Language Server Protocol (LSP) observer in `crates/zedkata-lsp`. This delivers protocol wire framing, lifecycle state management (`initialize`, `initialized`, `shutdown`, `exit`), clean stdout enforcement, UTF-16 aware in-memory document tracking (`didOpen`, `didChange`, `didClose`), incremental edit application and delta recording, and a full regression test suite driven by synthetic protocol tests and real Phase 0 golden captures.

## Tasks

1. **ZK-013.1: Implement LSP lifecycle and capabilities**
   - LSP wire framing (`Content-Length: ...\r\n\r\n{...}`) with clean stdout.
   - Lifecycle state machine: `Uninitialized` -> `Initialized` -> `ShuttingDown` -> `Exited`.
   - Advertise negotiated capabilities: `change: 2` (Incremental) and `positionEncoding: "utf-16"`.
   - Handle unsupported/unimplemented requests by returning empty/null responses without error, per `docs/ARCHITECTURE.md`.
   - Synthetic integration test covering full handshake, pre-init request rejection, unsupported method handling, and clean framing.

2. **ZK-013.2: Track documents and replay ordered changes**
   - In-memory document state tracking (`DocumentTracker`, `DocumentState`).
   - UTF-16 code unit line/column to UTF-8 byte offset translation, handling multi-byte Unicode and 4-byte surrogate pair emojis (`🌍`).
   - Line ending support for both CRLF (`\r\n`) and LF (`\n`).
   - Ordered incremental change application and full document replacement (`range: None`).
   - Version monotonicity tracking, rejecting stale/duplicate versions (`DocumentError::StaleVersion`).
   - Comprehensive unit and integration tests for document tracking and edge cases.

3. **ZK-013.3: Retain a production protocol regression suite**
   - Replay real Phase 0 wire captures from `tests/fixtures/lsp/`:
     - `2026-09-13-connection.jsonl` (smoke test with 10 changes)
     - `2026-09-13-matrix-01-dw.jsonl` (operator deletion `dw`)
     - `2026-09-13-matrix-02-dw-repeat.jsonl` (repeat operator deletion)
     - `2026-09-13-matrix-03-backspace.jsonl` (discrete backspaces)
   - Step-by-step validation asserting that reconstructed buffers and recorded edits match baseline expectations.

## Validation

- `cargo test --workspace --locked`
- `cargo test -p zedkata-lsp --test protocol_lifecycle_tests`
- `cargo test -p zedkata-lsp --test document_tracking_tests`
- `cargo test -p zedkata-lsp --test golden_fixture_regression_tests`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
