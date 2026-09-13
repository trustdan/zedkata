# ZK-010 Goal Verification Implementation Plan

**Date:** 2026-09-13  
**Status:** In execution. Approved by user.

## Scope

Implement goal buffer comparison and attempt outcome evaluation in `zedkata-core` under `crates/zedkata-core/src/verification.rs`.

## Tasks

1. **ZK-010.1: Compare goal buffers**
   - Newline normalization: standardize `\r\n` and `\r` to `\n` to prevent OS line-ending mismatches.
   - Exact and nonmatching comparison.
   - Whitespace: preserve significant indentation and spacing; normalize trailing newlines.
   - Unicode: multi-byte UTF-8, emojis, and non-ASCII character handling.
   - Empty content: empty buffer matching empty goal; empty vs non-empty mismatch.
   - `diff_buffers`: generate actionable line/column and expected/actual differences.
2. **ZK-010.2: Return explicit attempt outcomes**
   - `AttemptOutcome` enum: `Unfinished`, `Passed`, `Failed`, and `AlreadyFinalized`.
   - Distinguish unfinished from pass and finalized failure with actionable failure diagnostics.
   - Prevent duplicate completions / history additions on repeated updates for the same finalized attempt.

## Validation

- `cargo test --workspace --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo +1.85.0 check --workspace --locked`
- `cargo +1.85.0 test --workspace --locked`
