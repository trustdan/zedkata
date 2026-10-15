# Project Panel & Netrw Navigation

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `zed-02-project-panel` (`[[walkthrough]] id = "project-panel-walkthrough"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-25, REQ-29  
**Sample Workspace Fixture:** [`lessons/fixtures/walkthroughs/project_panel/`](../../lessons/fixtures/walkthroughs/project_panel/)  

---

## 1. Why Project Panel Navigation is a Guided Walkthrough

In ZedKata, drill exercises are scored by inferring modal editing technique from Language Server Protocol (`textDocument/didChange`) incremental notifications in `.kata` buffers.

However, **project panel navigation and file management cannot be represented as scored buffer drills** for two architectural reasons:

1. **Panel Navigation Emits Zero Buffer Deltas:**
   - Toggling the project panel (`ctrl-shift-e`), moving through directories (`j`, `k`, `h`, `l`, `-`), previewing files (`p`), and returning to the editor (`escape`) are purely graphical UI and focus state transitions.
   - They alter GPUI view focus and directory tree state, but emit **zero `textDocument/didChange` deltas** to the Language Server Protocol.
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed exposes no external API or extension hook to observe panel focus, tree selection index, or UI interactions.

2. **File Operations Act on the Filesystem, Not Active Katas:**
   - Creating files (`%`), creating directories (`d`), renaming entries (`shift-r`), or deleting entries (`shift-d`) modify filesystem state on disk.
   - These actions do not produce observable editing deltas within an active `.kata` drill document.
   - Scoring filesystem manipulations would require external directory watchers and filesystem mutation sandboxing rather than modal editor technique evaluation.

Per **REQ-25** (*"LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough"*) and **REQ-29**, project panel workflows are classified as an **unscored guided walkthrough** with concrete reproducible workspace fixtures, verified instructions, and self-reported completion semantics.

---

## 2. Zed's Native Netrw-Style Keybinding Architecture

Zed provides native support for Vim's classic `netrw` file explorer bindings out of the box when Vim mode is enabled. These bindings reside under the context:
```json
"context": "ProjectPanel && not_editing"
```
When this context is active (the project panel is focused and you are not typing a filename into an inline rename/new file input), standard Vim motions and netrw shortcuts are directly available.

### 2.1 Audited Project Panel Keybinding Table

Audited against Zed 1.19.2 (`assets/keymaps/vim.json` and `https://zed.dev/docs/project-panel`):

| Action | Vim Mode Shortcut | Standard / Global Shortcut | Zed Command Identifier | Description / Behavior |
|:---|:---:|:---:|:---|:---|
| **Toggle Panel Focus** | `ctrl-shift-e` | `ctrl-shift-e` / `cmd-shift-e` | `project_panel::ToggleFocus` | Toggles focus between the active editor and the project panel. |
| **Return to Editor** | `escape` | — | `vim::ToggleProjectPanelFocus` | In panel: immediately returns focus back to the active editor pane. |
| **Next Entry** | `j` / `down` | `down` | `vim::MenuSelectNext` | Move tree selection cursor down one row. |
| **Previous Entry** | `k` / `up` | `up` | `vim::MenuSelectPrevious` | Move tree selection cursor up one row. |
| **Collapse Directory** | `h` | `left` | `project_panel::CollapseSelectedEntry` | Collapses selected folder, or moves to its parent folder if already collapsed. |
| **Expand Directory** | `l` | `right` | `project_panel::ExpandSelectedEntry` | Expands selected directory or focuses its first child. |
| **Select Parent** | `-` | — | `project_panel::SelectParent` | Classic netrw: jumps selection cursor directly to parent directory. |
| **Jump to Top** | `g g` | `home` | `menu::SelectFirst` | Jumps to the first entry at the top of the project tree. |
| **Jump to Bottom** | `G` (`shift-g`) | `end` | `menu::SelectLast` | Jumps to the last entry in the expanded project tree. |
| **Next / Prev Folder** | `}` / `{` | — | `project_panel::SelectNextDirectory` / `SelectPrevDirectory` | Skips intermediate files to jump between directory entries. |
| **Open in Permanent Tab** | `enter` / `t` | `enter` (or double-click) | `project_panel::OpenPermanent` | Opens selected file in a permanent editor tab and focuses editor. |
| **Preview File** | `p` / `space` | `space` (or single-click) | `project_panel::Open` | Opens selected file as a temporary preview tab (italicized title). |
| **Open in Vertical Split** | `v` | — | `project_panel::OpenSplitVertical` | Opens file into a new vertical split pane (side-by-side). |
| **Open in Horizontal Split** | `o` | — | `project_panel::OpenSplitHorizontal` | Opens file into a new horizontal split pane (stacked). |
| **New File** | `%` | `ctrl-n` / `cmd-n` | `project_panel::NewFile` | Classic netrw: prompts for filename inside selected directory. |
| **New Directory** | `d` | `alt-ctrl-n` / `alt-cmd-n` | `project_panel::NewDirectory` | Classic netrw: prompts for folder name inside selected directory. |
| **Rename Entry** | `R` (`shift-r`) | `F2` | `project_panel::Rename` | Opens inline rename box with filename stem pre-selected. |
| **Delete Entry** | `D` (`shift-d`) | `ctrl-delete` | `project_panel::Delete` | Prompts confirmation to permanently delete selected item. |
| **Move to Trash** | `delete` | `cmd-backspace` | `project_panel::Trash` | Moves item to system trash (recoverable). |
| **Undo File Operation** | `ctrl-z` | `cmd-z` / `ctrl-z` | `project_panel::Undo` | Undoes file/folder creation, rename, or trash from the panel. |
| **Compare Marked Files** | `z d` | `alt-d` | `project_panel::CompareMarkedFiles` | When exactly 2 files are selected, opens a visual side-by-side diff. |
| **Search in Directory** | `/` | `ctrl-alt-shift-f` | `project_panel::NewSearchInDirectory` | Opens project search scoped specifically to the selected folder. |
| **Next / Prev Git Change** | `] c` / `[ c` | — | `project_panel::SelectNextGitEntry` / `SelectPrevGitEntry` | Jumps between modified/added files in the git tree. |
| **Next / Prev Diagnostic** | `] d` / `[ d` | — | `project_panel::SelectNextDiagnostic` / `SelectPrevDiagnostic` | Jumps to files with errors or warnings in the project tree. |
| **Reveal in OS Explorer** | `x` | `alt-ctrl-r` | `project_panel::RevealInFileManager` | Opens Windows File Explorer / macOS Finder at selected item. |
| **Open with System App** | `s` | — | `workspace::OpenWithSystem` | Opens selected file in OS default external application. |

---

## 3. Concrete Reproducible Walkthrough Scenarios

To practice these workflows, open the sample fixture project folder in Zed:
```bash
zed lessons/fixtures/walkthroughs/project_panel
```

Once opened, the project panel will display the root directory `project_panel` containing `Cargo.toml`, `README.md`, `src/`, and `tests/`.

---

### Scenario 1: Focus Toggling & Seamless Editor Handoff

**Objective:** Move between code editing and project navigation without touching the mouse.

1. Ensure focus is currently inside the editor with `src/main.rs` open.
2. Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> (macOS: <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd>).
   - **Observation:** Focus shifts instantly to the project panel on the left dock; the active selection highlight appears on the tree.
3. Press <kbd>Escape</kbd>.
   - **Observation:** Focus returns immediately back into the active code editor with your cursor exactly where you left it.
4. Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> again to return to the project panel.

---

### Scenario 2: Netrw Tree Navigation & Directory Folding

**Objective:** Traverse deep directory hierarchies using pure Vim keys.

1. With the project panel focused, press `g g` to jump to the top item.
2. Press `j` and `k` to move up and down through the list of entries:
   - Notice how `j` advances down to `src/` and `tests/`.
3. Stop on `src/`. If it is collapsed, press `l` to expand it.
   - Notice child entries: `main.rs`, `lib.rs`, and folder `utils/`.
4. Press `j` until cursor is on `utils/`. Press `l` to expand it.
   - Child entries `mod.rs` and `format.rs` appear.
5. Press `h`:
   - Because `utils/` is expanded, pressing `h` collapses it back into a single row.
6. Press `h` again:
   - Because `utils/` is now collapsed, pressing `h` jumps the cursor up to the parent directory (`src/`).
7. Press `-` (minus):
   - Jumps directly up to the workspace root directory (`project_panel`).
8. Press `G` (`shift-g`):
   - Selection jumps directly to the last entry in the tree (`tests/integration_test.rs`).

---

### Scenario 3: Multi-Pane Layouts via Splits (`v`, `o`, `enter`, `p`)

**Objective:** Open files side-by-side or stacked without manual pane dragging.

1. Navigate in the project panel to `src/lib.rs`.
2. Press `v`:
   - **Observation:** Zed opens `src/lib.rs` in a **new vertical split pane** to the right of your existing editor. Focus moves to the new pane.
3. Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> to refocus the project panel.
4. Navigate to `src/utils/format.rs`.
5. Press `o`:
   - **Observation:** Zed opens `format.rs` in a **new horizontal split pane** below the active pane.
6. Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd> to refocus the panel and navigate to `README.md`.
7. Press `p` (or <kbd>Space</kbd>):
   - **Observation:** `README.md` opens in a **preview tab** (italicized tab title). If you navigate to another file and press `p`, the preview tab is reused rather than cluttering your tab bar.
8. Press <kbd>Enter</kbd> on `README.md`:
   - Promotes the preview to a permanent tab.

---

### Scenario 4: Keyboard-Driven File Management & Undo

**Objective:** Create, rename, and organize project files directly from the panel.

1. In the project panel, navigate to `src/utils/`.
2. Press `%` (netrw new file shortcut):
   - **Observation:** An inline text box opens inside `src/utils/` awaiting the new file name.
3. Type `cache.rs` and press <kbd>Enter</kbd>:
   - The file `src/utils/cache.rs` is created on disk and immediately opened in the editor.
4. Refocus the panel (<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>E</kbd>) and select `src/`.
5. Press `d` (netrw new directory shortcut):
   - An inline text box opens. Type `models` and press <kbd>Enter</kbd>.
   - Directory `src/models/` is created on disk.
6. Navigate to `src/utils/cache.rs` and press `R` (`shift-r`):
   - Inline rename box opens with `cache` highlighted (stem pre-selected). Type `store.rs` and press <kbd>Enter</kbd>.
   - The file is renamed to `store.rs`.
7. With `store.rs` selected, press `ctrl-z` (or `cmd-z`):
   - **Observation:** Zed's project panel undo history reverses the rename, returning the file to `cache.rs`!
8. Select `cache.rs` and press `D` (`shift-d`):
   - Confirmation dialog appears asking to delete `cache.rs`. Confirm deletion.

---

### Scenario 5: Scoped Search, Git Navigation & File Comparison

**Objective:** Inspect modified files, errors, and diffs using panel chords.

1. **Scoped Search (`/`):**
   - Select `src/utils/` and press `/`.
   - Zed opens the project search bar with the search path filter automatically scoped to `src/utils/`.
2. **File Diff Comparison (`z d`):**
   - In the project panel, navigate to `src/lib.rs`.
   - Hold <kbd>Shift</kbd> and press `j` to select both `src/lib.rs` and `src/main.rs`.
   - Press `z d` (or <kbd>Alt</kbd>+<kbd>D</kbd>).
   - **Observation:** Zed opens a visual side-by-side diff comparing the two marked files.
3. **Diagnostic Navigation (`] d` / `[ d`):**
   - If files in the workspace contain compile warnings or errors, pressing `] d` in the panel jumps directly to the next file with diagnostics, while `[ d` jumps to the previous.
4. **Git Change Navigation (`] c` / `[ c`):**
   - Pressing `] c` jumps directly to the next modified file with uncommitted git changes.

---

## 4. Optional Ergonomic Enhancements (`keymap.json`)

While `ctrl-shift-e` and `<escape>` are the built-in defaults, many modal developers prefer a spacebar leader key (`space e`) to toggle the project panel.

To enable this, open your keymap file (<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> -> `zed: open keymap file`) and add:

```json
[
  {
    "context": "Editor && VimControl && !VimWaiting && !menu",
    "bindings": {
      "space e": "project_panel::ToggleFocus"
    }
  },
  {
    "context": "ProjectPanel && not_editing",
    "bindings": {
      "space e": "vim::ToggleProjectPanelFocus"
    }
  }
]
```

This creates a symmetrical `space e` toggle: pressing `space e` in any normal-mode editor pane focuses the project panel, and pressing `space e` (or `<escape>`) in the project panel returns instantly to the editor.

---

## 5. Self-Reported Verification Checklist

To confirm mastery of Zed's project panel netrw workflows:

- [ ] Toggled focus into the project panel using `ctrl-shift-e` and returned to editor with `<escape>`.
- [ ] Navigated entries smoothly with `j` and `k`, expanded folders with `l`, and collapsed folders with `h`.
- [ ] Jumped to parent folders using `-` and tree boundaries using `g g` and `G`.
- [ ] Opened files in vertical splits (`v`) and horizontal splits (`o`).
- [ ] Previewed a file with `p` without creating a permanent tab.
- [ ] Created a new file with `%` and a new folder with `d`.
- [ ] Renamed a file with `shift-r` and tested panel undo with `ctrl-z`.
- [ ] Compared two marked files using `z d`.
