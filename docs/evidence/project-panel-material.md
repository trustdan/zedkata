# ZK-027.2 — Project Panel Material & Netrw Navigation Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `zed-02-project-panel` (`[[walkthrough]] id = "project-panel-walkthrough"`)  
**Walkthrough Document:** [`docs/walkthroughs/project-panel.md`](../walkthroughs/project-panel.md)  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/project_panel/`](../../lessons/fixtures/walkthroughs/project_panel/)  
**Prerequisites:** `["operators-01-edit"]`  
**Requirements:** REQ-25, REQ-29  

---

## 1. Primary Source Verification & Keybinding Audit (OQ-15)

Audited against official Zed documentation (`https://zed.dev/docs/project-panel`, `https://zed.dev/docs/vim`), local Zed 1.19.2 installation, and Zed's native keymap `assets/keymaps/vim.json` (under context `"ProjectPanel && not_editing"`):

| Action / Intent | Vim / Netrw Shortcut | Global / Default Shortcut | Zed Action Identifier | Audit Finding & Status |
|:---|:---:|:---:|:---|:---:|
| **Toggle Panel Focus** | `ctrl-shift-e` | `ctrl-shift-e` / `cmd-shift-e` | `project_panel::ToggleFocus` | **VERIFIED**. Focuses project panel from any editor pane. |
| **Return Focus to Editor** | `escape` | — | `vim::ToggleProjectPanelFocus` | **VERIFIED**. Seamlessly returns focus to active editor pane from panel. |
| **Move Selection Down** | `j` / `down` | `down` | `vim::MenuSelectNext` | **VERIFIED**. Advances tree selection cursor down. |
| **Move Selection Up** | `k` / `up` | `up` | `vim::MenuSelectPrevious` | **VERIFIED**. Advances tree selection cursor up. |
| **Collapse Directory** | `h` | `left` | `project_panel::CollapseSelectedEntry` | **VERIFIED**. Collapses directory, or jumps to parent directory if already collapsed. |
| **Expand Directory** | `l` | `right` | `project_panel::ExpandSelectedEntry` | **VERIFIED**. Expands directory or focuses first child entry. |
| **Select Parent** | `-` | — | `project_panel::SelectParent` | **VERIFIED**. Classic netrw minus shortcut; moves focus to parent directory. |
| **Jump to Top** | `g g` | `home` | `menu::SelectFirst` | **VERIFIED**. Jumps selection to the root/first entry in tree. |
| **Jump to Bottom** | `G` (`shift-g`) | `end` | `menu::SelectLast` | **VERIFIED**. Jumps selection to last entry in tree. |
| **Next / Prev Directory** | `}` / `{` | — | `project_panel::SelectNextDirectory` | **VERIFIED**. Skips files and jumps between directories. |
| **Open in Permanent Tab** | `enter` / `t` | `enter` / double-click | `project_panel::OpenPermanent` | **VERIFIED**. Opens file in permanent tab and moves focus into editor. |
| **Preview File** | `p` / `space` | `space` / single-click | `project_panel::Open` | **VERIFIED**. Opens file in temporary preview tab without permanent tab allocation. |
| **Open in Vertical Split** | `v` | — | `project_panel::OpenSplitVertical` | **VERIFIED**. Opens file into a new vertical split pane (side-by-side). |
| **Open in Horizontal Split** | `o` | — | `project_panel::OpenSplitHorizontal` | **VERIFIED**. Opens file into a new horizontal split pane (stacked). |
| **New File** | `%` | `ctrl-n` / `cmd-n` | `project_panel::NewFile` | **VERIFIED**. Classic netrw new file prompt inside active folder. |
| **New Directory** | `d` | `alt-ctrl-n` / `alt-cmd-n` | `project_panel::NewDirectory` | **VERIFIED**. Classic netrw new folder prompt inside active folder. |
| **Rename Entry** | `R` (`shift-r`) | `F2` | `project_panel::Rename` | **VERIFIED**. Opens inline rename prompt with stem pre-selected. |
| **Delete Entry** | `D` (`shift-d`) | `ctrl-delete` | `project_panel::Delete` | **VERIFIED**. Prompts confirmation dialog to permanently delete entry. |
| **Move to Trash** | `delete` | `cmd-backspace` | `project_panel::Trash` | **VERIFIED**. Moves entry to system trash (recoverable). |
| **Undo File Operation** | `ctrl-z` | `cmd-z` / `ctrl-z` | `project_panel::Undo` | **VERIFIED**. Undoes file creation, folder creation, rename, or trash. |
| **Compare Marked Files** | `z d` | `alt-d` | `project_panel::CompareMarkedFiles` | **VERIFIED**. When 2 files are selected, opens side-by-side diff view. |
| **Search in Directory** | `/` | `ctrl-alt-shift-f` | `project_panel::NewSearchInDirectory` | **VERIFIED**. Opens search bar scoped to active directory. |
| **Next / Prev Git Change** | `] c` / `[ c` | — | `project_panel::SelectNextGitEntry` | **VERIFIED**. Jumps tree selection to next/previous modified git file. |
| **Next / Prev Diagnostic** | `] d` / `[ d` | — | `project_panel::SelectNextDiagnostic` | **VERIFIED**. Jumps tree selection to next/previous file with errors/warnings. |
| **Reveal in File Explorer** | `x` | `alt-ctrl-r` | `project_panel::RevealInFileManager` | **VERIFIED**. Reveals selected entry in native OS file manager. |
| **Open with System App** | `s` | — | `workspace::OpenWithSystem` | **VERIFIED**. Opens entry in default external application. |

---

## 2. Observability Boundaries & Outcome Classification (REQ-25 / REQ-29)

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*), all project panel workflows were evaluated against Zed's observer architecture:

1. **Navigation and Focus are Pure GPUI Transitions:**
   - Pressing `ctrl-shift-e`, `<escape>`, `j`, `k`, `h`, `l`, `-`, `g g`, `G`, `}` or `{` updates GPUI focus and selection highlight within the tree widget.
   - These actions produce **zero buffer modifications** and zero LSP `textDocument/didChange` deltas.
   - Zed exposes no external IPC or extension API to query panel tree focus or selection index (OQ-04, REQ-10).
2. **Splits and Previews Affect Editor Layout, Not Document Content:**
   - Opening files with `v` (vertical split), `o` (horizontal split), `enter`, or `p` (preview) alters workspace layout and creates editor panes, but generates no incremental edit deltas.
3. **File Operations Mutate the Filesystem:**
   - Creating files (`%`), creating directories (`d`), renaming (`R`), and deleting (`D`) execute filesystem operations on disk.
   - They do not emit LSP editing events within the student's `.kata` drill buffer.
   - Evaluating filesystem mutations would require an OS-level filesystem watcher, conflicting with the single-buffer modal editing model.
4. **Classification:**
   - Project panel navigation and netrw workflows are formally classified as an **unscored guided walkthrough** (`[[walkthrough]] id = "project-panel-walkthrough"` in `lessons/zed-02-project-panel.toml` and `docs/walkthroughs/project-panel.md`) with self-reported verification criteria rather than false automated drill scores.

---

## 3. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-25, REQ-29):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Modal users migrating to Zed. Explains how Zed natively honors `netrw` conventions (`j`/`k`, `h`/`l`, `-`, `%`, `d`, `v`, `o`) in its modern GUI file tree. | **PASS** |
| **One Concept per Unit** | Walkthrough recipes isolate individual workflows: focus handoff, netrw tree traversal, multi-pane split layout, file creation/rename/undo, and diagnostics/diff comparisons. | **PASS** |
| **Outcome-Focused Wording** | Prompts and walkthrough scenarios state concrete outcomes (e.g. *"Open file in vertical split pane without touching mouse"*, *"Create new file inside directory using %"*). | **PASS** |
| **Non-Drillable Classification** | Strictly classified as an unscored guided walkthrough (`[[walkthrough]] id = "project-panel-walkthrough"`). | **PASS** |
| **Honest Dependencies** | Placed in Module 7 (`zed-02-project-panel`), which depends honestly on `["operators-01-edit"]`. | **PASS** |
| **Prose Budget** | Lesson prose is 59 words (budget: <80 words). Full technical tables and walkthrough scenarios reside in `docs/walkthroughs/project-panel.md`. | **PASS** |

---

## 4. Concrete Reproducible Walkthrough Scenarios

All scenarios verified using sample workspace fixture [`lessons/fixtures/walkthroughs/project_panel/`](../../lessons/fixtures/walkthroughs/project_panel/) in Zed 1.19.2:

1. **Scenario 1: Focus Toggling & Editor Handoff:**
   - Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd>: focus immediately shifts to the project panel on the left dock.
   - Press <kbd>Escape</kbd>: focus immediately returns to the active editor pane.
2. **Scenario 2: Netrw Tree Navigation & Folding:**
   - `j`/`k` moves through items; `l` expands `src/` and `src/utils/`; `h` collapses `src/utils/` and jumps up to `src/`; `-` jumps to workspace root; `G` jumps to `tests/integration_test.rs`.
3. **Scenario 3: Multi-Pane Split Layouts:**
   - In panel, cursor on `src/lib.rs` -> press `v`: opens `src/lib.rs` in a vertical split pane to the right.
   - Cursor on `src/utils/format.rs` -> press `o`: opens `format.rs` in a horizontal split pane below.
   - Cursor on `README.md` -> press `p`: opens preview tab without creating permanent tab clutter.
4. **Scenario 4: Netrw File Creation, Rename & Undo:**
   - Inside `src/utils/`, press `%` -> type `cache.rs` + <kbd>Enter</kbd>: file created and opened.
   - Inside `src/`, press `d` -> type `models` + <kbd>Enter</kbd>: directory created.
   - On `cache.rs`, press `shift-r` -> rename to `store.rs`.
   - Press `ctrl-z`: undoes rename, restoring `cache.rs`.
   - Press `shift-d`: confirms deletion and removes file cleanly.
5. **Scenario 5: Diff Comparison & Diagnostic Navigation:**
   - Select `src/lib.rs` and `src/main.rs` with <kbd>Shift</kbd>+`j` -> press `z d`: opens side-by-side visual diff.
   - `] d` / `[ d` jumps across files with compiler errors/warnings in project tree.
   - `] c` / `[ c` jumps across files with git modifications.

---

## 5. Automated Verification Matrix

- **Unit and Schema Test:**
  - `test_project_panel_lesson_schema_and_walkthrough` in `crates/zedkata-core/tests/lesson_loading.rs`:
    - Validates `zed-02-project-panel` TOML parsing, module number (`Some(7)`), `draft = true`, requires `["operators-01-edit"]`.
    - Validates all key concepts (`"ctrl-shift-e"`, `"j"`, `"k"`, `"h"`, `"l"`, `"v"`, `"o"`, `"p"`, `"%"`, `"d"`, `"shift-r"`, `"shift-d"`, `"-"`).
    - Validates walkthrough metadata (`id = "project-panel-walkthrough"`).
    - Validates fixture directory and files on disk (`Cargo.toml`, `README.md`, `src/main.rs`, `src/lib.rs`, `src/utils/mod.rs`, `src/utils/format.rs`, `tests/integration_test.rs`).
    - Validates catalog acyclicity and prerequisite satisfaction via `catalog.validate_catalog()`.
    - Confirms runnable curriculum catalog remains the 5 admitted modules.
- **Workspace Test Suite:**
  - `cargo test`: all workspace tests pass cleanly across all crates (0 failures, 0 errors).
  - Linter & formatting: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt -- --check` pass with 0 warnings.
