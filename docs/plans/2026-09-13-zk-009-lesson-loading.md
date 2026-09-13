# ZK-009 Lesson Loading Implementation Plan

**Date:** 2026-09-13  
**Status:** In execution. Approved by user.

## Scope

Implement typed TOML lesson parsing, schema/rule validation, catalog dependency resolution, and fixture path resolution in `zedkata-core`. Add the first runnable standalone lesson (`motions-01-word`) with real `.start.kata` and `.goal.kata` fixtures, while keeping `motions-03-textobjects` marked as an uncalibrated draft excluded from the runnable catalog.

## Tasks

1. **ZK-009.1: Parse and validate lesson schema**
   - Core types: `Lesson`, `Exercise`, `Walkthrough`, `ForbidRules`.
   - Error types in `crates/zedkata-core/src/error.rs` using `thiserror`.
   - Strict validation: non-empty slug IDs, edit budgets (`par_edits <= max_edits`), supported forbid rules (`backspace_run`), rejection of unobservable or unknown forbid rules (like `visual_line_select`) with file/field context.
2. **ZK-009.2: Resolve fixture paths and catalog dependencies**
   - `Catalog` struct in `crates/zedkata-core/src/catalog.rs`.
   - Fixture path resolution relative to the lesson file directory (independent of process working directory).
   - Validation for missing fixtures, duplicate IDs, missing prerequisites, and prerequisite cycles.
   - Filtering of draft lessons: `catalog.runnable_lessons()` vs `catalog.all_lessons()`.
3. **ZK-009.3: Add one runnable standalone lesson**
   - `lessons/motions-01-word.toml`: standalone, no prerequisites, concise prose under 80 words.
   - Real fixtures in `lessons/fixtures/motions-01/`.
   - Test loading through core from a different working directory.

## Validation

- `cargo test --workspace --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --locked --all-targets -- -D warnings`
- `cargo +1.85.0 check --workspace --locked`
- `cargo +1.85.0 test --workspace --locked`
