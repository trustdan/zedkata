# ZK-032.2 — Validate and Store Generated Assets Evidence

- **Task:** [ZK-032.2](../BACKLOG.md#task-zk-032-2) (Validate and store generated assets)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-12](../REQUIREMENTS.md#L50), [REQ-28](../REQUIREMENTS.md#L66), [REQ-31](../REQUIREMENTS.md#L69)
- **Architecture Reference:** [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-core/src/catalog.rs` (`default_generated_dir`, `default_data_dir`, `MergeSummary`, `Catalog::merge_from_dir`, `Catalog::insert_preserving_existing`, `Catalog::load_with_generated`)
  - `crates/zedkata-core/src/error.rs` (`CoreError::DataPathError`)
  - `crates/zedkata-agent/src/storage.rs` (`GeneratedAssetError`, `StoredAssetPaths`, `verify_path_within_root`, `validate_candidate`, `store_candidate_assets`)
  - `crates/zedkata-agent/src/generator.rs` (`CandidateExercise::validate`, `CandidateExercise::validate_and_store`, `CandidateExercise::validate_and_store_with_catalog`)
  - `crates/zedkata-agent/src/config.rs` (`AgentConfig.generated_dir`, `UserConfigFile.generated_dir`, `resolved_generated_dir()`, resolution hierarchy)
  - `crates/zedkata-agent/src/main.rs` (CLI `--store` and `--out-dir` support on `generate` subcommand)
  - `crates/zedkata-agent/tests/asset_validation_and_storage_tests.rs` (10 automated integration tests covering all acceptance criteria)

---

## 1. Executive Summary

Task **ZK-032.2** enforces strict structural validation, path traversal prevention, missing fixture detection, ID collision protection, atomic disk persistence, and catalog merging for generated practice assets.

All acceptance criteria are satisfied:
1. **Reject Malformed TOML/Rules:**
   - Validates generated `raw_toml` against `zedkata_core::lesson::Lesson::from_toml`.
   - Rejects TOML syntax errors, duplicate keys, or structural parsing failures as `GeneratedAssetError::MalformedToml`.
   - Rejects invalid rules as `GeneratedAssetError::InvalidRules`:
     - Par edits = 0 or missing.
     - Par edits exceeding max edits (`par_edits > max_edits`).
     - Zero-based cursor coordinates (must be 1-based line >= 1, col >= 1).
     - Empty task goal, lesson title, or prose.
     - Non-draft lessons with unobservable forbid rules (e.g. `visual_line_select`).
     - Inconsistencies between Candidate metadata and generated TOML fields (IDs, fixture paths).
2. **Reject Missing Fixtures:**
   - Validates fixture buffers before touching disk:
     - Rejects empty or whitespace-only start buffer as `GeneratedAssetError::MissingFixture`.
     - Rejects empty or whitespace-only goal buffer as `GeneratedAssetError::MissingFixture`.
     - Rejects identical start and goal buffers (degenerate zero-edit drills) as `GeneratedAssetError::MissingFixture`.
   - Verifies fixture paths end with `.kata`.
3. **Reject Paths Escaping the Generated Content Root (Path Traversal):**
   - Enforces `verify_path_within_root` on all asset paths (TOML filename, start fixture, goal fixture).
   - Rejects relative path traversal attempts containing `..` (`Component::ParentDir`).
   - Rejects absolute paths (leading `/`, `\`, or Windows drive letters).
   - Rejects root or prefix components.
   - Validates `lesson_id` contains only lowercase ASCII alphanumeric characters and hyphens, forbidding directory separators or `.`/`..`.
4. **ID Collisions Preserve Existing Lessons:**
   - Checks `<root_dir>/<lesson_id>.toml` before write; if the file already exists on disk, returns `Err(GeneratedAssetError::IdCollision)` and leaves the existing file completely untouched and unaltered.
   - Checks existing `Catalog` if provided; if `lesson_id` already exists, returns `Err(GeneratedAssetError::IdCollision)` and preserves the existing lesson entry.
   - In catalog merging (`Catalog::merge_from_dir`): if a generated lesson ID collides with an existing lesson, the existing lesson is strictly preserved, and the duplicate is recorded in `summary.skipped_collisions`.
5. **Valid Assets Write Together to Configured User-Data Directory:**
   - Standard platform resolution layout under ADR 0005:
     - Development: `lessons/generated/` (or via `ZEDKATA_GENERATED_DIR`)
     - Windows: `%LOCALAPPDATA%\zedkata\lessons\generated\` (fallback `%USERPROFILE%\AppData\Local\zedkata\lessons\generated\`)
     - Linux: `$XDG_DATA_HOME/zedkata/lessons/generated/` (fallback `~/.local/share/zedkata/lessons/generated/`)
     - macOS: `~/Library/Application Support/zedkata/lessons/generated/`
   - Multi-file atomic persistence:
     - Writes `.kata.tmp` for start and goal fixtures, and `.toml.tmp` for lesson TOML.
     - Atomically renames temporary files to target paths.
     - Cleans up all newly written files if any write or rename fails, preventing orphaned or partial assets.
     - Transitions candidate status to `CandidateStatus::Persisted`.
     - Returns `StoredAssetPaths` containing paths to the written TOML, start fixture, and goal fixture.
   - The stored directory is immediately loadable by `Catalog::load_from_dir`, fixture paths resolve cleanly, and `Catalog::validate_runnable()` passes without errors.

---

## 2. Directory Layout & Platform Resolution

```
<configured_generated_dir>/
├── <lesson_id>.toml
└── fixtures/
    └── <lesson_id>/
        ├── <exercise_id>.start.kata
        └── <exercise_id>.goal.kata
```

### Precedence Hierarchy:
1. `ZEDKATA_GENERATED_DIR` environment variable.
2. `ZEDKATA_DATA_DIR` environment variable (`<data_dir>/lessons/generated`).
3. User `config.toml` (`generated_dir = "..."`).
4. Platform standard user data path (`%LOCALAPPDATA%` on Windows, `$XDG_DATA_HOME` on Linux, `~/Library/Application Support` on macOS).

---

## 3. Automated Test Suite

A dedicated integration test suite in `crates/zedkata-agent/tests/asset_validation_and_storage_tests.rs` covers all acceptance criteria:

| Test Case | Acceptance Criteria Verified | Result |
|---|---|---|
| `test_reject_malformed_toml` | Rejects malformed TOML syntax with `GeneratedAssetError::MalformedToml` | PASSED |
| `test_reject_invalid_rules` | Rejects 0 par edits, par > max, 0-based cursor, and unobservable forbid rules | PASSED |
| `test_reject_missing_or_identical_fixtures` | Rejects empty start buffer, empty goal buffer, and identical buffers | PASSED |
| `test_reject_path_traversal_escaping_content_root` | Rejects `..`, absolute paths, and traversing IDs escaping content root | PASSED |
| `test_id_collision_preserves_existing_lesson_on_disk` | ID collision on disk returns `IdCollision` and preserves existing file intact | PASSED |
| `test_id_collision_with_existing_catalog_preserves_existing_lesson` | ID collision with loaded catalog preserves existing lesson entry | PASSED |
| `test_valid_assets_write_together_atomically` | Valid candidate writes all 3 assets atomically; catalog loads and verifies | PASSED |
| `test_catalog_merge_preserves_existing_on_collision` | `Catalog::merge_from_dir` adds new lessons and preserves existing on collision | PASSED |
| `test_default_generated_dir_resolution` | Tests `ZEDKATA_GENERATED_DIR` and `ZEDKATA_DATA_DIR` environment resolution | PASSED |
| `test_cli_generate_store_flag` | CLI `zedkata-agent generate --concept dw --store --out-dir <dir>` writes assets | PASSED |

### Workspace Verification Summary:
- **`cargo test`**: All 243 unit and integration tests across all workspace crates pass cleanly.
- **`cargo fmt --check`**: Formatting is 100% compliant.
- **`cargo clippy --workspace --all-targets -- -D warnings`**: Zero warnings or errors.
- **`cargo install --path crates/zedkata-agent --force`**: Successfully reinstalled `zedkata-agent` v0.1.0 to PATH.

---

## 4. Verification Transcript Excerpts

```
$ cargo test --test asset_validation_and_storage_tests
running 10 tests
test test_default_generated_dir_resolution ... ok
test test_reject_malformed_toml ... ok
test test_reject_missing_or_identical_fixtures ... ok
test test_reject_path_traversal_escaping_content_root ... ok
test test_id_collision_preserves_existing_lesson_on_disk ... ok
test test_reject_invalid_rules ... ok
test test_id_collision_with_existing_catalog_preserves_existing_lesson ... ok
test test_valid_assets_write_together_atomically ... ok
test test_catalog_merge_preserves_existing_on_collision ... ok
test test_cli_generate_store_flag ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```
$ zedkata-agent generate --concept dw --store --out-dir temp_gen_test
### Candidate Practice Drill: Generated Practice: Delete word (dw) (gen-dw-1792113896)
...
[zedkata-agent] Validated and stored generated exercise in 'temp_gen_test':
  TOML:    temp_gen_test\gen-dw-1792113896.toml
  Start:   temp_gen_test\fixtures/gen-dw-1792113896/delete-redundant-param.start.kata
  Goal:    temp_gen_test\fixtures/gen-dw-1792113896/delete-redundant-param.goal.kata
```

---

## 5. Conclusion

Task **ZK-032.2** is complete and verified. Unblocks **ZK-032.3** (Select and complete generated practice).
