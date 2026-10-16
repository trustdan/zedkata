# Multibuffer Project Search & Cross-File Editing

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `zed-03-multibuffer` (`[[walkthrough]] id = "multibuffer-walkthrough"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-25, REQ-29  
**Sample Fixture Workspace:** [`lessons/fixtures/walkthroughs/multibuffer/`](../../lessons/fixtures/walkthroughs/multibuffer/)  
**Prerequisites:** `["zed-01-lsp-actions"]`  

---

## 1. Why Multibuffers are Guided Walkthroughs

In ZedKata, drill exercises evaluate student technique through Language Server Protocol (`textDocument/didChange`) incremental notifications within a single `.kata` exercise buffer.

However, **multibuffers and cross-file editing cannot be represented as scored single-buffer drills** for four architectural reasons:

1. **Multibuffers are Aggregate GPUI Views Across Multiple Files:**
   - A Zed multibuffer is an interactive synthetic surface that dynamically stitches together live excerpts from across different files in a workspace.
   - It is not a single file buffer; it does not correspond to a single file URI on disk or a single `.kata` exercise file.
2. **Edits Mutate Disparate Project Files:**
   - When a user performs edits inside a multibuffer (e.g. updating an excerpt in `src/session.rs` and another in `src/handlers.rs`), the modifications emit `didChange` notifications for those respective source files.
   - The student drill observer (`zedkata-lsp`) attaches strictly to `.kata` buffers (configured for `language = "Zedkata"`). It cannot observe or score edits occurring across arbitrary `.rs` files or synthetic multibuffer excerpts.
3. **Absence of a Single-Buffer Deterministic Goal State:**
   - Standard drills evaluate a single start buffer reaching an exact target goal buffer within a strict Par edit budget.
   - Multibuffer operations modify multiple project files simultaneously or perform exploratory navigation. There is no single `.kata` start/goal pair that can capture simultaneous cross-file excerpts.
4. **Search and Excerpt UI States are Internal to Zed:**
   - Invoking project search (`pane::DeploySearch` / `g /`), toggling regex (`alt-r`) or case sensitivity (`alt-c`), jumping between excerpts, expanding context lines (`shift-enter`), and selecting multiple matches (`ctrl-d`) are internal GPUI state transitions.
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed exposes no external IPC or extension API to inspect panel focus, search input text, active excerpts, or cursor sets.

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*) and **REQ-29**, multibuffer workflows are formally classified as an **unscored guided walkthrough** with concrete reproducible workspace fixtures, verified step-by-step recipes, and self-reported verification criteria.

---

## 2. Zed's Multibuffer Architecture & Verified Keybindings

The **multibuffer** is Zed's signature architectural feature. Rather than forcing developers to open dozens of separate editor tabs to view search results, references, or related code snippets, Zed presents excerpts together in a single continuous, fully-editable canvas.

### 2.1 Multibuffer Anatomy
- **Excerpt Header:** A bar at the top of each excerpt displaying the file path relative to the workspace root (e.g., `src/session.rs`), along with its starting line number.
- **Context Lines:** Editable lines above and below each matched region (configurable via `excerpt_context_lines`, default `2`).
- **Divider:** Subtle horizontal rules separating excerpts from different files or non-contiguous sections of the same file.
- **Live Synchronization:** Edits made in an excerpt reflect immediately in the underlying file buffer.
- **Unified Save:** Pressing `ctrl-s` (or `:w` in Vim mode) inside a multibuffer saves all modified files represented in its excerpts at once.

### 2.2 Verified Multibuffer & Project Search Keybinding Table

Audited against Zed 1.19.2 (`assets/keymaps/vim.json`, `assets/keymaps/default.json`, and `https://zed.dev/docs`):

| Action / Intent | Vim Mode Shortcut | Default / Alternate Shortcut | Zed Action Identifier | Notes |
|:---|:---:|:---:|:---|:---|
| **Deploy Project Search** | `g /` | `ctrl-shift-f` / `cmd-shift-f` | `pane::DeploySearch` | Opens project search input at bottom or panel. |
| **Search in New Tab** | `<ctrl-enter>` (in search bar) | `cmd-enter` (macOS) | `project_search::SearchInNew` | Opens query results in a dedicated new multibuffer tab. |
| **Match Case Toggle** | `alt-c` | `alt-c` | `search::ToggleCaseSensitive` | Toggles case-sensitive searching. |
| **Whole Word Toggle** | `alt-w` | `alt-w` | `search::ToggleWholeWord` | Toggles whole-word boundary matching. |
| **Regex Mode Toggle** | `alt-r` | `alt-r` | `search::ToggleRegex` | Enables regular expression syntax. |
| **Toggle Replace Field** | `ctrl-alt-r` / `alt-enter` | `alt-cmd-f` / `ctrl-alt-f` | `search::ToggleReplace` | Expands replacement input field. |
| **Next Search Match** | `n` (in editor) / `enter` (in search) | `F3` / `enter` | `search::SelectNextMatch` | Jumps to next match in multibuffer. |
| **Previous Search Match** | `N` (in editor) / `shift-enter` (in search) | `shift-F3` | `search::SelectPrevMatch` | Jumps to previous match in multibuffer. |
| **Expand Excerpts Context** | `shift-enter` | `shift-enter` | `editor::ExpandExcerpts` | Expands context lines around active excerpt. |
| **Expand Excerpts Down** | — | — | `editor::ExpandExcerptsDown` | Expands context lines downward. |
| **Expand Excerpts Up** | — | — | `editor::ExpandExcerptsUp` | Expands context lines upward. |
| **Open Excerpt into Full File** | `alt-enter` | `alt-enter` | `editor::OpenExcerpts` | Opens the active excerpt's file in a full editor tab. |
| **Select Next Occurrence** | `ctrl-d` | `cmd-d` (macOS) | `editor::SelectNextMatch` | Adds a secondary cursor to the next matching symbol. |
| **Save All Multibuffer Files** | `ctrl-s` / `:w` | `cmd-s` / `ctrl-s` | `workspace::Save` | Persists edits to all modified files in the multibuffer. |
| **Find All References** | `g A` | `shift-F12` | `editor::FindAllReferences` | Generates a multibuffer of all symbol references across workspace. |

---

## 3. Concrete Step-by-Step Walkthrough Recipes

Open the sample workspace in Zed:
```bash
zed lessons/fixtures/walkthroughs/multibuffer
```

### Recipe 1: Project-Wide Search (`g /`) and Multibuffer Generation

**Objective:** Query all usages of `validate_token` across the sample project and view the matches in a unified multibuffer.

1. Ensure focus is in an editor pane in normal mode.
2. Press `g /` (or `ctrl-shift-f` / `cmd-shift-f`).
   - The Project Search bar appears.
3. Type `validate_token` and press `enter`.
4. Observe the newly deployed Multibuffer:
   - Notice excerpt sections for:
     - `src/session.rs`: method definition `pub fn validate_token(&self) -> Result<(), &'static str>`
     - `src/store.rs`: call `token.validate_token()?;`
     - `src/handlers.rs`: calls in `handle_protected_route` and `handle_logout`
     - `src/main.rs`: call `admin_token.validate_token()`
   - Notice the excerpt headers showing file paths and line offsets.
5. Press `escape` to move focus from the search input into the multibuffer in Vim normal mode.

---

### Recipe 2: Navigating Excerpts & Context Expansion (`shift-enter`)

**Objective:** Move across file boundaries within the multibuffer and reveal surrounding code context without leaving the unified view.

1. In normal mode inside the multibuffer, press `j` and `k` to move the cursor through the excerpts.
2. Press `n` to jump directly to the next search match. Zed smoothly scrolls down to the next excerpt, transitioning seamlessly from `src/session.rs` to `src/store.rs`.
3. Press `n` again to advance into `src/handlers.rs`.
4. Position your cursor on the `token.validate_token()?;` line in `src/handlers.rs`.
5. Press `shift-enter` (or run `editor: expand excerpts` from the command palette):
   - Notice how Zed expands the excerpt upwards and downwards, revealing more of the surrounding `handle_protected_route` function.
6. Press `shift-enter` again to expand further context lines until the complete function signature and return statement are visible.

---

### Recipe 3: Simultaneous Cross-File Editing (Direct Vim Edits Across Excerpts)

**Objective:** Refactor method logic across multiple files directly from the multibuffer without switching tabs.

1. Position the cursor in the first excerpt (`src/session.rs`) on the line:
   ```rust
   if !self.raw.starts_with("zk_") {
   ```
2. In normal mode, use standard Vim motion `f"` to jump to the quote, then `ci"` to change inside quotes:
   - Type `zed_` and press `escape`.
   - The check in `src/session.rs` is now updated to `starts_with("zed_")`.
3. Press `j` or `n` to move down into the `src/main.rs` excerpt:
   ```rust
   let admin_token = SessionToken::new("zk_admin_9999", 42);
   ```
4. Move cursor to `"zk_admin_9999"`, press `ci"`, type `"zed_admin_9999"`, and press `escape`.
5. Notice that both `src/session.rs` and `src/main.rs` now have unsaved changes, clearly reflected in their excerpt headers with a dirty indicator, all within the exact same view.

---

### Recipe 4: Multi-Cursor Batch Refactoring Across Excerpt Boundaries (`ctrl-d` + `c w`)

**Objective:** Perform simultaneous multi-cursor renaming across excerpt boundaries spanning different files.

1. In the multibuffer, place your cursor over the word `validate_token` in `src/store.rs`.
2. Press `ctrl-d` (macOS `cmd-d`):
   - Zed selects `validate_token` and places a primary cursor.
3. Press `ctrl-d` again:
   - Zed jumps into the next excerpt (`src/handlers.rs`) and selects the next occurrence of `validate_token`, adding a second active cursor.
4. Press `ctrl-d` once more:
   - A third cursor is added on the second occurrence in `src/handlers.rs`.
5. Press `c w` to enter insert mode across all three cursors at once:
   - Type `verify_token` and press `escape`.
6. Observe that all three occurrences across both files have been renamed simultaneously.

---

### Recipe 5: Opening Individual Excerpts into Full Tabs (`alt-enter`) and Saving All Files (`ctrl-s`)

**Objective:** Break out of an excerpt to view the entire file, and simultaneously persist all multibuffer modifications.

1. In the multibuffer, position your cursor anywhere within the `src/handlers.rs` excerpt.
2. Press `alt-enter` (`editor::OpenExcerpts`).
   - Zed immediately opens `src/handlers.rs` as a full, dedicated editor tab with the cursor positioned at the exact line of the excerpt.
3. Switch back to the multibuffer tab (`ctrl-w h` or click the tab).
4. In normal mode, type `:w` or press `ctrl-s` (macOS `cmd-s`).
   - Zed saves all dirty files (`src/session.rs`, `src/store.rs`, `src/handlers.rs`, and `src/main.rs`) in a single atomic action.
5. All dirty indicators across the excerpt headers clear immediately.

---

## 4. Self-Reported Verification Checklist

Because multibuffer interactions and cross-file modifications are unobservable by single-buffer LSP observers (REQ-25, REQ-29), students verify their mastery using this concrete self-report checklist:

- [ ] **Project Search Deployment:** Successfully triggered project search using `g /` and navigated the resulting multibuffer with standard Vim keys (`j`, `k`, `n`, `N`).
- [ ] **Context Expansion:** Expanded context lines around an excerpt using `shift-enter` (`editor::ExpandExcerpts`) to reveal surrounding code.
- [ ] **Cross-File Simultaneous Editing:** Modified code in two distinct file excerpts (`src/session.rs` and `src/main.rs`) without opening separate tabs.
- [ ] **Multi-Cursor across Excerpts:** Placed multiple cursors across file boundaries using `ctrl-d` and changed identifiers with `c w`.
- [ ] **Excerpt Breakout:** Jumped from an excerpt into a full editor tab using `alt-enter` (`editor::OpenExcerpts`).
- [ ] **Unified Multi-File Save:** Persisted all cross-file edits across the entire multibuffer at once using `ctrl-s` / `:w`.
