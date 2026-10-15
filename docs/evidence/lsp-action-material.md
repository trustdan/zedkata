# ZK-027.1 — LSP-Backed Action Material & Walkthrough Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `zed-01-lsp-actions` (`[[walkthrough]] id = "lsp-actions-walkthrough"`)  
**Walkthrough Document:** [`docs/walkthroughs/lsp-actions.md`](../walkthroughs/lsp-actions.md)  
**Sample Fixture:** [`lessons/fixtures/walkthroughs/lsp/sample_service.rs`](../../lessons/fixtures/walkthroughs/lsp/sample_service.rs)  
**Prerequisites:** `["operators-01-edit"]`  
**Requirements:** REQ-25, REQ-29  

---

## 1. Primary Source Verification & Keybinding Audit (OQ-14)

Audited against official Zed documentation (`https://zed.dev/docs/vim.md#language-server`, `https://zed.dev/docs/languages`), local Zed 1.19.2 installation, and `crates/zedkata-lsp`:

| Feature / Action | Vim Shortcut in Zed | Alternative / Standard Shortcut | Zed Command Identifier | Audit Finding & Status |
|:---|:---:|:---:|:---|:---:|
| **Rename Symbol** | `c d` | `space r`, `F2` | `editor::Rename` | **VERIFIED**. Mnemonic *"change definition"*. Triggers inline rename prompt and highlights all occurrences with linked edit cursors. |
| **Code Actions** | `g .` | `space .`, `ctrl-x ctrl-l` | `editor::ToggleCodeActions` | **VERIFIED**. Opens quick-fix suggestions popup for diagnostic/symbol under cursor. |
| **Go to Definition** | `g d` | `F12` | `editor::GoToDefinition` | **VERIFIED**. Jumps directly to definition. |
| **Definition in Split** | `<ctrl-w> g d` | — | `editor::GoToDefinitionSplit` | **VERIFIED**. Splits active pane and opens definition in the new split. |
| **Go to Declaration** | `g D` | — | `editor::GoToDeclaration` | **VERIFIED**. Jumps to explicit header/declaration. |
| **Go to Type Definition** | `g y` | — | `editor::GoToTypeDefinition` | **VERIFIED**. Jumps to type/struct/enum definition. |
| **Go to Implementation** | `g I` | — | `editor::GoToImplementation` | **VERIFIED**. Jumps to trait implementation block. |
| **Find All References** | `g A` | `shift-F12` | `editor::FindAllReferences` | **VERIFIED**. Opens multibuffer excerpt view listing all references across workspace. |
| **Next Diagnostic** | `g ]` or `] d` | `F8` | `editor::GoToDiagnostic` | **VERIFIED**. Jumps forward to next warning or compiler error. |
| **Previous Diagnostic** | `g [` or `[ d` | `shift-F8` | `editor::GoToPrevDiagnostic` | **VERIFIED**. Jumps backward to previous warning or compiler error. |
| **Hover Documentation** | `g h` | `K`, `space k` | `editor::ShowHover` | **VERIFIED**. Opens markdown signature and docstring popup. |
| **Symbol Outline in File** | `g s` | `ctrl-t` | `outline::Toggle` | **VERIFIED**. Opens fuzzy search modal over file outline symbols. |
| **Symbol in Project** | `g S` | `ctrl-T` | `project_symbols::Toggle` | **VERIFIED**. Opens fuzzy search modal over workspace symbol index. |
| **Diagnostics Panel** | `:clist` | — | `diagnostics::Deploy` | **VERIFIED**. Vim ex-command opening the workspace diagnostics panel. |

---

## 2. Observability Boundaries & Outcome Classification (REQ-25 / REQ-29)

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*), all LSP-backed actions are evaluated against the observer architecture:

1. **Non-Mutating Inspection Actions (`g d`, `g D`, `g y`, `g I`, `g A`, `g ]`, `g [`, `g h`, `g s`, `g S`, `:clist`):**
   - Cursor jumps and UI popovers emit **zero buffer modifications** and zero LSP `textDocument/didChange` deltas.
   - Zed exposes no API to observe cursor coordinates, focus, or open popovers (OQ-04, REQ-10).
   - Attempting to score inspection actions as buffer drills is structurally impossible.
2. **Mutating Refactoring Actions (`c d` symbol rename, `g .` code actions):**
   - While mutating actions modify buffer text, they **strictly require an active semantic language server** (e.g. `rust-analyzer`, `pyright`, `gopls`) implementing `renameProvider` or `codeActionProvider`.
   - In student drill buffers (`*.kata`), the only registered language server is `zedkata-lsp` (per `extension/extension.toml`). `zedkata-lsp` is an exercise conductor and observer; it does not implement semantic language providers. Pressing `c d` or `g .` in `.kata` files yields *"No rename provider available"* or no actions.
   - Real language files (`.rs`, `.py`) run against external toolchains that vary by student machine and are unobserved by `zedkata-lsp`.
3. **Classification:**
   - All normal-mode LSP actions are formally classified as **Guided Walkthroughs** (`[[walkthrough]] id = "lsp-actions-walkthrough"`) with self-reported verification rather than false automated drill scores.

---

## 3. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-25, REQ-29):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Modal users adopting Zed. Focuses on what Zed adds that Vim never had: seamless Normal-mode bindings (`c d`, `g .`, `g ]`, `g d`, `g h`, `g A`) for language server intelligence. | **PASS** |
| **One Concept per Unit** | Walkthrough recipes isolate individual workflows: atomic symbol rename, quick-fix code actions, inline diagnostic navigation, definition/split navigation, hover documentation, and diagnostics panel. | **PASS** |
| **Outcome-Focused Wording** | Prompts and walkthrough scenarios state explicit, verifiable outcomes (e.g. *"Rename method across multiple call sites in a single refactor"*, *"Navigate to next compiler error with single keypress"*). | **PASS** |
| **Non-Drillable Classification** | Strictly classified as an unscored guided walkthrough (`[[walkthrough]] id = "lsp-actions-walkthrough"` in `lessons/zed-01-lsp-actions.toml` and `docs/walkthroughs/lsp-actions.md`). | **PASS** |
| **Honest Dependencies** | Placed in Module 6 (`zed-01-lsp-actions`), which depends honestly on `["operators-01-edit"]`. | **PASS** |
| **Prose Budget** | Lesson prose is 48 words (budget: <80 words). Full conceptual explanations, keymaps, and recipes are organized in dedicated walkthrough documentation. | **PASS** |

---

## 4. Concrete Reproducible Walkthrough Scenarios

All scenarios manually verified using sample fixture [`lessons/fixtures/walkthroughs/lsp/sample_service.rs`](../../lessons/fixtures/walkthroughs/lsp/sample_service.rs) in Zed 1.19.2:

1. **Scenario 1: Multi-Site Symbol Rename (`c d`):**
   - Cursor on `record_metric` definition (line 49).
   - Press `c d`: inline input appears, linked edits highlight references at lines 73, 74, 75.
   - Enter `ingest_metric` + <kbd>Enter</kbd>: definition and all 3 call sites update simultaneously.
2. **Scenario 2: Quick-Fix Code Actions (`g .`):**
   - Cursor on struct or unused import.
   - Press `g .`: context quick-fix popup displays available code transformations.
   - In Insert mode, `ctrl-x ctrl-l` opens the same code action menu.
3. **Scenario 3: Inline Diagnostic Navigation (`g ]` / `g [`) and Panel (`:clist`):**
   - Intentional type mismatch on line 73 renders inline error block directly beneath line.
   - Pressing `g ]` or `] d` jumps directly to the error line.
   - Typing `:clist` in Normal mode opens the workspace diagnostics list panel.
   - Pressing `u` cleanly restores baseline buffer.
4. **Scenario 4: Definition Jumping (`g d`, `<ctrl-w> g d`) and Type Definition (`g y`):**
   - Cursor on `TelemetryClient` in `initialize_client`: pressing `g y` jumps to `pub struct TelemetryClient` on line 28.
   - Cursor on `new`: pressing `<ctrl-w> g d` automatically creates an adjacent split pane and opens `pub fn new()` inside it.
5. **Scenario 5: Hover Documentation (`g h`) and References (`g A`):**
   - Cursor on `compute_average`: pressing `g h` opens markdown popover showing docstring and signature.
   - Cursor on `MetricLevel`: pressing `g A` opens Multibuffer listing all references across the project.

---

## 5. Automated Verification Matrix

- **Unit and Schema Test:**
  - `test_lsp_actions_lesson_schema_and_walkthrough` in `crates/zedkata-core/tests/lesson_loading.rs`:
    - Validates `zed-01-lsp-actions` TOML parsing, module number (`Some(6)`), `draft = true`, requires `["operators-01-edit"]`.
    - Validates all key concepts (`"c d"`, `"g ."`, `"g ]"`, `"g ["`, `"g d"`, `"g h"`, `"g A"`, `":clist"`).
    - Validates walkthrough metadata (`id = "lsp-actions-walkthrough"`).
    - Validates fixture file existence and contents on disk (`sample_service.rs`).
    - Validates catalog acyclicity and prerequisite satisfaction via `catalog.validate_catalog()`.
    - Confirms runnable curriculum catalog remains the 5 admitted modules.
- **Workspace Test Suite:**
  - `cargo test`: all 162 workspace tests pass cleanly across all crates (0 failures, 0 errors).
  - Linter & formatting: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt -- --check` pass with 0 warnings.
