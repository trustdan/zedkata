# Keymap Contexts & Modal Binding Customization

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `operators-01-edit` (`[[walkthrough]] id = "keymap-contexts"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-24, REQ-25, REQ-29  

---

## 1. Why Keymap Customization is a Guided Walkthrough

Zed's editor communicates buffer modifications over language server protocol (`textDocument/didChange`) deltas. However, **Zed exposes no API or LSP notifications for configuration files (`keymap.json`), internal UI focus trees, cursor states, or physical keystrokes** (see `CLAUDE.md` and `docs/OPEN-QUESTIONS.md` OQ-04 / OQ-13).

Because keybinding evaluation and context dispatch are entirely internal to Zed's UI event loop:
- Modifying `keymap.json` produces no buffer deltas in exercise files.
- Attempting to score keymap edits as an automated drill would violate the LSP observation boundary.
- Per **REQ-25** and **REQ-29**, configuration teaching is classified as an **unscored guided walkthrough** with explicit, reproducible configuration outcomes and self-reported verification.

---

## 2. Zed's Keymap Context Architecture

### 2.1 Configuration File Locations
User keymaps are stored in JSON format at:
- **Windows:** `%APPDATA%\Zed\keymap.json` (or `~\AppData\Roaming\Zed\keymap.json`)
- **macOS / Linux:** `~/.config/zed/keymap.json`

Open this file directly in Zed using the Command Palette (<kbd>Ctrl-Shift-P</kbd> or <kbd>Cmd-Shift-P</kbd>):
`zed: open keymap file`

### 2.2 The Context Tree
Zed's UI elements form a nested context tree. For example, when focused in an editor buffer:
```text
Workspace os=windows keyboard_layout=...
  Pane
    Editor mode=full extension=rs vim_mode=normal
```
When focused in the project panel:
```text
Workspace os=windows
  Dock
    ProjectPanel not_editing
```

### 2.3 Context Rules and Predicates
1. **Attributes belong to specific tree nodes:**
   - `vim_mode` and `vim_operator` reside on the `Editor` node.
   - `Workspace && vim_mode == normal` **will never match** because `Workspace` does not possess the `vim_mode` property.
2. **Boolean Combinators:**
   - `&&` (Logical AND): `"Editor && vim_mode == normal"`
   - `||` (Logical OR): `"vim_mode == normal || vim_mode == visual"`
   - `!` (Logical NOT): `!menu` (ensures bindings don't fire when an autocompletion or pop-up menu is active)
   - `(...)` (Grouping): `"VimControl && (!menu || vim_mode == normal)"`
   - `>` (Ancestor Combinator): Matches if an ancestor in the tree matches the left-hand side and the current node matches the right-hand side, e.g. `"Workspace > Editor && vim_mode == normal"`.

### 2.4 Documented Modal Context Variables

| Context Expression | Target State | Notes |
|:---|:---|:---|
| `VimControl` | Normal, Visual, or Operator-pending modes | Shorthand alias for `vim_mode == normal \|\| vim_mode == visual \|\| vim_mode == operator`. |
| `vim_mode == normal` | Normal mode only | Primary scope for navigation, refactoring, and command chords. |
| `vim_mode == visual` | Visual (character/line/block) mode | Scope for selection modifiers and surround wrapping. |
| `vim_mode == insert` | Insert mode | Scope for chorded escapes (e.g. `j k`) or Alt-navigation. |
| `vim_mode == replace` | Replace mode | Single or multi-character replacement state. |
| `vim_mode == waiting` | Waiting for arbitrary character argument | Active immediately after pressing `f`, `F`, `t`, `T`, or `r`. |
| `vim_mode == operator` | Waiting for motion or text object | Active immediately after pressing an operator like `d`, `c`, `y`. |
| `vim_operator == <op>` | Specific pending operator | Identifies the active operator, e.g. `vim_operator == d`, `vim_operator == c`, `vim_operator == cs`. |

### 2.5 Precedence and Conflict Resolution
- **Depth Precedence:** Bindings matching deeper/lower nodes in the context tree take precedence over ancestors (`Editor` bindings override `Workspace` bindings).
- **Order Precedence:** If two bindings match at the same node depth, the one defined **later** in the configuration file takes precedence. User keymaps (`keymap.json`) load after default system keymaps.
- **Prefix Chords & Timeouts:** If keybinding $A$ is a prefix of multi-stroke keybinding $B$ (e.g. `"space"` vs `"space w"`), Zed waits up to 1000 ms (1 second) for subsequent strokes.

---

## 3. Concrete Configuration Outcomes & Reproducible Examples

Below are concrete, verified configurations you can apply in your `keymap.json`.

### Walkthrough Example 1: Normal-Mode Leader Save Chord
**Outcome:** Pressing <kbd>Space</kbd> followed by <kbd>w</kbd> in Normal mode saves the active buffer without triggering in Insert mode or when completion menus are open.

1. Open `keymap.json` via Command Palette: `zed: open keymap file`.
2. Add the following entry to the root JSON array:
   ```json
   {
     "context": "vim_mode == normal && !menu",
     "bindings": {
       "space w": "workspace::Save"
     }
   }
   ```
3. Save `keymap.json`.
4. **Verification:** Open any dirty buffer. In Normal mode, tap <kbd>Space</kbd> then <kbd>w</kbd>. The buffer save indicator disappears. In Insert mode, typing `space w` inserts literal text `" w"`.

---

### Walkthrough Example 2: Insert-Mode Fast Escape (`j k`)
**Outcome:** Typing `j` followed immediately by `k` in Insert mode exits to Normal mode without requiring reaching for <kbd>Esc</kbd>.

1. In `keymap.json`, add:
   ```json
   {
     "context": "vim_mode == insert",
     "bindings": {
       "j k": "vim::NormalBefore"
     }
   }
   ```
2. Save `keymap.json`.
3. **Verification:** Open an editor buffer. Press `i` to enter Insert mode. Type `j k`. Zed instantly switches back to Normal mode (block cursor restored, mode indicator shows normal). If you pause after typing `j`, the character `j` is inserted as normal once the chord timeout expires.

---

### Walkthrough Example 3: Subword Navigation across Modal States
**Outcome:** In Normal and Visual modes, `w` and `b` navigate by `camelCase` and `snake_case` subwords, but standard operator commands like `dw` continue operating on full words.

1. In `keymap.json`, add:
   ```json
   {
     "context": "VimControl && !menu && vim_mode != operator",
     "bindings": {
       "w": "vim::NextSubwordStart",
       "b": "vim::PreviousSubwordStart",
       "e": "vim::NextSubwordEnd",
       "g e": "vim::PreviousSubwordEnd"
     }
   }
   ```
2. Save `keymap.json`.
3. **Verification:** Position cursor on `parseHeaderPayload`. Press `w`. Cursor moves to `Header`, then `Payload`. In contrast, pressing `dw` deletes the entire identifier `parseHeaderPayload` because `vim_mode != operator` excludes operator-pending mode.

---

### Walkthrough Example 4: Scoping AnyQuotes to Delimiter Operators
**Outcome:** Bind `q` to `vim::AnyQuotes` strictly when an operator or surround change is pending (`vim_operator == a`, `i`, or `cs`), preserving `q` for macro recording in Normal mode.

1. In `keymap.json`, add:
   ```json
   {
     "context": "vim_operator == a || vim_operator == i || vim_operator == cs",
     "bindings": {
       "q": "vim::AnyQuotes"
     }
   }
   ```
2. Save `keymap.json`.
3. **Verification:**
   - In Normal mode, pressing `q` still initiates macro recording (`q{reg}`).
   - While inside `"hello world"`, typing `ciq` triggers `vim::AnyQuotes`, changing the content inside the quotes.

---

### Walkthrough Example 5: Inspecting Context with Key Context View
**Outcome:** Diagnose keybinding conflicts or check active context node attributes in real-time.

1. In Zed, open any code file and enter Normal mode.
2. Open the Command Palette (<kbd>Ctrl-Shift-P</kbd> or <kbd>Cmd-Shift-P</kbd>).
3. Run `dev: open key context view`.
4. Observe the live diagnostics pane:
   - Notice the hierarchy: `Workspace` -> `Pane` -> `Editor`.
   - Inspect the active attributes on `Editor`: `mode=full`, `vim_mode=normal`.
   - Switch to Insert mode (`i`): observe `vim_mode` changes dynamically from `normal` to `insert`.
   - Press an operator (`d`): observe `vim_mode=operator` and `vim_operator=d` appear during the pending state.

---

## 4. Troubleshooting Common Context Misconfigurations

| Symptom | Root Cause | Solution |
|:---|:---|:---|
| Binding never activates | Scoped to wrong node level (e.g. `"context": "Workspace && vim_mode == normal"`). | Change context to `"Editor && vim_mode == normal"` or `"Workspace > Editor && vim_mode == normal"`. |
| Normal-mode chord fires in autocomplete | Missing `!menu` guard. | Add `&& !menu` to context expression so Enter/Space chords do not conflict with completion popups. |
| Keypress experiences 1-second lag | Key is a prefix of another active chord binding. | Check `keymap.json` for multi-stroke shortcuts starting with that key. Remove unwanted prefixes or rebind. |
| Macro recording (`q`) broken | Bound `q` at `VimControl` or `Editor` level instead of `vim_operator`. | Scope custom text object bindings to `vim_operator == a \|\| vim_operator == i`. |
