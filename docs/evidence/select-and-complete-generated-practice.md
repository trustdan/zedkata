# ZK-032.3 — Select and Complete Generated Practice Evidence

- **Task:** [ZK-032.3](../BACKLOG.md#task-zk-032-3) (Select and complete generated practice)
- **Status:** Complete & Verified
- **Date:** 2026-10-15
- **Requirements:** [REQ-12](../REQUIREMENTS.md#L50), [REQ-28](../REQUIREMENTS.md#L66), [REQ-31](../REQUIREMENTS.md#L69)
- **Architecture Reference:** [ADR 0005: Model Providers and Credentials Architecture](../adr/0005-model-providers-and-credentials.md)
- **Primary Sources & Components:**
  - `crates/zedkata-core/src/lesson.rs` (`Lesson::is_generated()`, identifying `gen-*` prefix)
  - `crates/zedkata-core/src/catalog.rs` (`Catalog::load_with_generated`, `default_generated_dir`, deterministic lesson sorting prioritizing curriculum ahead of generated practice)
  - `crates/zedkata-lsp/src/feedback.rs` (`FeedbackEngine::evaluate_attempt`, uncalibrated technique budget diagnostics)
  - `crates/zedkata-lsp/src/main.rs` (`--generated-dir` CLI flag, platform path discovery)
  - `crates/zedkata-tui/src/app.rs` (`App::load_with_generated`, `App::refresh_state`, external fixture resolution)
  - `crates/zedkata-tui/src/terminal.rs` (`run_tui_app_with_generated`)
  - `crates/zedkata-tui/src/ui.rs` (TUI rendering: `[G]` badge, `[gen]` suffix, task warning banner, uncalibrated par lines, uncalibrated history badges, `[✓]` completion)
  - `crates/zedkata-tui/src/main.rs` (`--generated-dir` CLI flag, `resolve_generated_dir` precedence, headless CLI support)
  - `crates/zedkata-tui/tests/generated_practice_tests.rs` (5 integration tests covering discovery, buffer preparation, rendering, LSP feedback, and complete end-to-end journey)
  - `crates/zedkata-lsp/tests/feedback_integration_tests.rs` (`test_generated_practice_drill_pass_emits_uncalibrated_diagnostic_and_finalizes_session`)

---

## 1. Executive Summary

Task **ZK-032.3** connects generated practice assets produced by `zedkata-agent` into the primary student learning workflow across the TUI, CLI, and LSP observer.

All acceptance criteria are satisfied:
1. **Generated Catalog Discovery Works Outside the Repo:**
   - The TUI and LSP discover generated practice exercises stored in standard platform data paths (`%LOCALAPPDATA%\zedkata\lessons\generated\` on Windows, `$XDG_DATA_HOME/zedkata/lessons/generated/` on Linux, `~/Library/Application Support/zedkata/lessons/generated/` on macOS) or via `--generated-dir <PATH>` or `ZEDKATA_GENERATED_DIR`.
   - Discovery requires zero repository files or git tree mutations.
   - Shipped curriculum lessons take sorting priority, keeping the base curriculum orderly while appending generated practice exercises.
   - Fixture paths for generated exercises (`fixtures/<lesson_id>/<exercise_id>.start.kata` and `.goal.kata`) resolve relative to the external lesson directory without relying on process working directory.
2. **Select, Launch, and Complete a Targeted Generated Exercise:**
   - The student can select a generated drill from the catalog using TUI vim navigation (`j`/`k`, `Tab`) or CLI (`zedkata-tui select <gen_id> 0`).
   - Launching an exercise prepares the buffer in `<exercises_dir>/<lesson_id>/<target_file>` copying from the external start fixture.
   - The student performs vim motions in the buffer.
   - The LSP feedback engine detects goal matching, calculates score, finalizes the session, records history, and marks completion.
   - The TUI displays `[✓]` completion badge once passed.
3. **Uncalibrated Technique Budgets:**
   - Algorithmic heuristics from generated drills are never presented as human-calibrated curriculum standards.
   - Explicitly labeled across all surfaces:
     - **TUI Catalog:** Marked with `[G]` prefix badge and `[gen]` suffix.
     - **TUI Task Pane:** Prominent banner displayed: `NOTE: Targeted generated practice drill (uncalibrated technique budget)`.
     - **TUI Task Details:** Qualified budget line: `Par: X edit(s) (uncalibrated)`.
     - **TUI Progress History:** Qualified score entries: `score 100, uncalibrated`.
     - **Headless CLI Status:** Displays `[generated practice]` and `par X (uncalibrated)`.
     - **LSP Diagnostics:** Diagnostics specify `Passed — X edit(s), par Y (uncalibrated), score 100 (generated practice)`.

---

## 2. Directory Layout & Discovery Precedence

Generated exercises reside strictly outside the git tree:

```
%LOCALAPPDATA%\zedkata\lessons\generated\
├── gen-dw-1792114636.toml
└── fixtures\
    └── gen-dw-1792114636\
        ├── delete-redundant-param.start.kata
        └── delete-redundant-param.goal.kata
```

### Discovery Resolution Precedence:
1. Explicit `--generated-dir <PATH>` CLI argument.
2. `ZEDKATA_GENERATED_DIR` environment variable.
3. Platform standard user data path:
   - Windows: `%LOCALAPPDATA%\zedkata\lessons\generated\`
   - Linux: `$XDG_DATA_HOME/zedkata/lessons/generated/`
   - macOS: `~/Library/Application Support/zedkata/lessons/generated/`

---

## 3. Automated Test Suite

A dedicated integration test suite in `crates/zedkata-tui/tests/generated_practice_tests.rs` and companion test in `crates/zedkata-lsp/tests/feedback_integration_tests.rs` verify all acceptance criteria:

| Test Name | File | Verified Capabilities | Status |
|---|---|---|---|
| `test_generated_catalog_discovery_outside_repo` | `generated_practice_tests.rs` | Discovers lessons outside repo, resolves fixtures correctly, merges into catalog | PASSED |
| `test_select_and_launch_generated_exercise_buffer` | `generated_practice_tests.rs` | Selects generated exercise, prepares buffer in isolated exercise directory | PASSED |
| `test_tui_uncalibrated_budget_rendering_and_completion_badge` | `generated_practice_tests.rs` | Verifies `[G]` badge, uncalibrated banner, par line, and `[✓]` completion rendering | PASSED |
| `test_lsp_feedback_and_scoring_uncalibrated_indication` | `generated_practice_tests.rs` | LSP detects buffer completion, applies par calculation, qualifies uncalibrated diagnostic | PASSED |
| `test_select_and_complete_generated_practice_full_journey` | `generated_practice_tests.rs` | Full lifecycle: select -> launch -> edit -> pass -> TUI state refresh -> completion | PASSED |
| `test_generated_practice_drill_pass_emits_uncalibrated_diagnostic_and_finalizes_session` | `feedback_integration_tests.rs` | LSP observer handles generated practice buffer lifecycle and session finalization | PASSED |

Total workspace test count: **248 passed, 0 failed**.

---

## 4. Live Manual Verification Trace

### 4.1 Asset Generation & Storage
```powershell
PS C:\Users\Dan\zedkata> zedkata-agent generate --concept dw --store
Generating candidate exercise for concept 'dw'...
Generated exercise:
  Lesson ID:   gen-dw-1792114636
  Exercise ID: delete-redundant-param
  Task:        Delete the unused parameter 'redundant' and following comma
  Par edits:   1
  Max edits:   2
  Persisted:   true
  Stored at:   C:\Users\Dan\AppData\Local\zedkata\lessons\generated\gen-dw-1792114636.toml
```

### 4.2 Headless Selection
```powershell
PS C:\Users\Dan\zedkata> zedkata-tui select gen-dw-1792114636 0
Selected exercise 0 ('delete-redundant-param') of lesson 'gen-dw-1792114636'.
Prepared exercise buffer at:
  C:\Users\Dan\AppData\Local\zedkata\exercises\gen-dw-1792114636\delete-redundant-param.kata
```

### 4.3 Headless Status Inspection
```powershell
PS C:\Users\Dan\zedkata> zedkata-tui status
Zedkata Session Status:
  Session File:   "C:\\Users\\Dan\\AppData\\Local\\zedkata\\session.json"
  Schema Version: 1
  Student Start:  2026-09-14 02:24:39.477863600 UTC
  Current Lesson: gen-dw-1792114636 [generated practice]
  Exercise Index: 0
  Buffer Path:    C:\Users\Dan\AppData\Local\zedkata\exercises\gen-dw-1792114636\delete-redundant-param.kata
  Opened At:      2026-10-16 01:37:35.355870400 UTC

Active Attempt:
  Attempt ID:     attempt_1792114655355_4904
  Edit Count:     0
  Verdict:        IN PROGRESS
```

### 4.4 Headless Reset
```powershell
PS C:\Users\Dan\zedkata> zedkata-tui reset
Reset active exercise:
  Lesson ID:    gen-dw-1792114636
  Exercise ID:  delete-redundant-param
  Fresh Buffer: C:\Users\Dan\AppData\Local\zedkata\exercises\gen-dw-1792114636\delete-redundant-param.kata
  New Attempt:  attempt_1792114660232_1052
```

---

## 5. Acceptance Verification Checklist

- [x] **AC 1: Generated catalog discovery works outside the repo**
  - TUI and LSP locate `%LOCALAPPDATA%\zedkata\lessons\generated\` without repository modifications.
  - `--generated-dir` flag and `ZEDKATA_GENERATED_DIR` supported.
  - Fixtures load properly from outside repository directories.
- [x] **AC 2: Select, launch, and complete a targeted generated exercise**
  - Navigation, selection, launching, editing, and completion verified end-to-end.
  - Passing exercise marks completion in session state and renders `[✓]` in catalog.
- [x] **AC 3: Uncalibrated technique budgets**
  - High-visibility banner in TUI task pane.
  - Explicit `(uncalibrated)` par label across TUI, CLI status, and LSP diagnostics.
  - `[G]` and `[gen]` badges identify generated lessons in catalog pane.
