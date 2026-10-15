# ZK-026.2 — Keymap Context Teaching Material & Walkthrough Evidence

**Date:** 2026-10-15  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Lesson:** `operators-01-edit` (`[[walkthrough]] id = "keymap-contexts"`)  
**Walkthrough Document:** `docs/walkthroughs/keymap-contexts.md`  
**Prerequisites:** `["motions-03-textobjects"]`  
**Requirements:** REQ-24, REQ-25, REQ-29  

---

## 1. Primary Source Verification (OQ-13 Audit)

Audited against official Zed documentation (`https://zed.dev/docs/key-bindings.md`, `https://zed.dev/docs/vim.md`) and local Zed 1.19.2:

| Concept / Context | Documented Status | Behavior / Syntax | Verification Reference | Notes |
|:---|:---:|:---|:---|:---|
| **Context Hierarchy** | **VERIFIED** | Tree: `Workspace` -> `Pane`/`Dock` -> `Editor` | `zed.dev/docs/key-bindings.md` | Attributes are node-local. Cross-level scoping requires ancestor operator `>`. |
| `VimControl` | **VERIFIED** | Shorthand alias: `vim_mode == normal \|\| vim_mode == visual \|\| vim_mode == operator` | `zed.dev/docs/vim.md#selecting-the-correct-context` | Standard helper for modal actions across editing modes. |
| `vim_mode == normal` | **VERIFIED** | Matches Normal mode on `Editor` node | `zed.dev/docs/vim.md` | Recommended with `&& !menu` for custom leader keys. |
| `vim_mode == insert` | **VERIFIED** | Matches Insert mode on `Editor` node | `zed.dev/docs/vim.md` | Target for chorded escapes (e.g. `j k` -> `vim::NormalBefore`). |
| `vim_mode == visual` | **VERIFIED** | Matches Visual mode on `Editor` node | `zed.dev/docs/vim.md` | Used for selection-specific surrounds (e.g. `shift-s` -> `vim::PushAddSurrounds`). |
| `vim_mode == operator` | **VERIFIED** | Operator pending state | `zed.dev/docs/vim.md` | Active after pressing `d`, `c`, `y`, `cs`, etc. |
| `vim_operator == <op>` | **VERIFIED** | Specific pending operator identifier | `zed.dev/docs/vim.md#example-configuration` | Used to bind custom delimiters (e.g. `vim_operator == a \|\| vim_operator == i`). |
| `dev: open key context view` | **VERIFIED** | Live diagnostic view of active context hierarchy | `zed.dev/docs/key-bindings.md` | Interactive verification tool for students to inspect active nodes. |
| **Precedence Resolution** | **VERIFIED** | Deeper node wins; later binding wins; 1000ms chord timeout | `zed.dev/docs/key-bindings.md#precedence` | Deterministic resolution rules for overlapping shortcuts. |
| **Observation Boundary** | **VERIFIED** | Client/GPUI internal; no LSP notifications | `CLAUDE.md`, OQ-04, OQ-13 | Configuration changes cannot be scored as drills; must be guided walkthroughs per REQ-25 / REQ-29. |

---

## 2. Curriculum Authoring Checklist

Verified against `.claude/agents/curriculum-author.md` and `docs/REQUIREMENTS.md` (REQ-22, REQ-24, REQ-25, REQ-29):

| Criterion | Implementation | Status |
|:---|:---|:---:|
| **Audience** | Modal users customizing keybindings and modal shortcuts in Zed. Focuses on what is unique to Zed: the GPUI context hierarchy, node-level attribute scoping, and modal context variables (`VimControl`, `vim_mode`, `vim_operator`). | **PASS** |
| **One Concept per Unit** | Each reproducible example targets a distinct configuration outcome: Normal-mode leader save, Insert-mode escape chord, subword navigation scoping, operator text object scoping, and live context tree diagnostics. | **PASS** |
| **Outcome-Focused Wording** | Prompts and walkthrough tasks state explicit, verifiable behavioral outcomes (e.g. "Configure 'space w' to save in Normal mode without triggering in Insert mode or completion menus"). | **PASS** |
| **Non-Drillable Classification** | Strictly classified as an unscored guided walkthrough (`[[walkthrough]] id = "keymap-contexts"` in `operators-01-edit.toml` and `docs/walkthroughs/keymap-contexts.md`) rather than an unobservable scored buffer drill. | **PASS** |
| **Honest Dependencies** | Placed in Module 4 (`operators-01-edit`), which depends honestly on `["motions-03-textobjects"]` where modal editing concepts are fully grounded. | **PASS** |
| **Prose Budget** | Lesson walkthrough prompt is 43 words (budget: <80 words). Full conceptual explanations and walkthrough recipes are organized in dedicated walkthrough documentation. | **PASS** |

---

## 3. Concrete Reproducible Walkthrough Examples

Detailed recipes and expected outcomes recorded in [`docs/walkthroughs/keymap-contexts.md`](../walkthroughs/keymap-contexts.md):

1. **Normal-Mode Leader Save:**
   - **Context:** `"vim_mode == normal && !menu"`
   - **Binding:** `"space w": "workspace::Save"`
   - **Observed Behavior:** In Normal mode, pressing `space w` immediately saves the buffer. In Insert mode, `space w` types `" w"` literally. In completion menus, pressing space does not accidentally save.
2. **Insert-Mode Fast Escape:**
   - **Context:** `"vim_mode == insert"`
   - **Binding:** `"j k": "vim::NormalBefore"`
   - **Observed Behavior:** Typing `j k` quickly in Insert mode drops to Normal mode. Pausing after `j` allows literal `j` insertion after the 1-second chord timeout.
3. **Subword Navigation Scoping:**
   - **Context:** `"VimControl && !menu && vim_mode != operator"`
   - **Bindings:** `"w": "vim::NextSubwordStart"`, `"b": "vim::PreviousSubwordStart"`
   - **Observed Behavior:** Cursor jumps between camelCase subwords on `w`/`b`. Pressing `dw` continues to delete the entire word because operator-pending mode is explicitly excluded (`vim_mode != operator`).
4. **Operator-Scoped Delimiters:**
   - **Context:** `"vim_operator == a || vim_operator == i || vim_operator == cs"`
   - **Binding:** `"q": "vim::AnyQuotes"`
   - **Observed Behavior:** `ciq` modifies inside any quote characters. Normal-mode macro recording (`q`) remains completely unshadowed.
5. **Context Tree Diagnostics:**
   - **Command:** `dev: open key context view`
   - **Observed Behavior:** Inspects active tree hierarchy (`Workspace` -> `Pane` -> `Editor`). Verifies why `"Workspace && vim_mode == normal"` fails (different node levels) while `"Editor && vim_mode == normal"` succeeds.

---

## 4. Automated Verification Matrix

All 161 automated workspace tests pass across all crates:

- `test_operators_lesson_operators_01_edit_fixtures_and_schema` in `crates/zedkata-core/tests/lesson_loading.rs`:
  - Validates `operators-01-edit` TOML schema, `draft = false`, `module = 4`, requires `["motions-03-textobjects"]`.
  - Verifies both walkthroughs (`exchange-and-surround` and `keymap-contexts`).
  - Verifies start and goal fixtures for all 3 exercises.
- `test_five_module_runnable_curriculum_admission_and_prerequisites` in `crates/zedkata-core/tests/lesson_loading.rs`:
  - Validates 5 runnable modules and 15 total drills across the curriculum DAG.
- Workspace test suite:
  - `cargo test`: 161 tests passing cleanly (0 failures, 0 errors).
  - Linter & formatting: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt -- --check` pass with 0 warnings.
