# Normal-Mode LSP Actions & Code Intelligence

**Category:** Guided Walkthrough (Unscored)  
**Lesson:** `zed-01-lsp-actions` (`[[walkthrough]] id = "lsp-actions-walkthrough"`)  
**Target Editor:** Zed 1.19.2+ (Vim mode enabled)  
**Requirements:** REQ-25, REQ-29  
**Sample Fixture:** [`lessons/fixtures/walkthroughs/lsp/sample_service.rs`](../../lessons/fixtures/walkthroughs/lsp/sample_service.rs)  

---

## 1. Why LSP Actions are Guided Walkthroughs

In ZedKata, drill exercises are scored by inferring technique from Language Server Protocol (`textDocument/didChange`) incremental notifications in `.kata` buffers.

However, **normal-mode LSP actions cannot be represented as scored buffer drills** for two architectural reasons:

1. **Non-Mutating Inspection Actions Emit No Deltas:**
   - Jumping to definitions (`g d`), declarations (`g D`), type definitions (`g y`), implementations (`g I`), or references (`g A`) moves the cursor and alters UI focus, but produces **zero buffer modifications**.
   - Hover documentation (`g h`) opens an ephemeral popover.
   - Diagnostic navigation (`g ]` / `g [`, `] d` / `[ d`) jumps between compiler diagnostics.
   - Per `CLAUDE.md`, REQ-10, and OQ-04, Zed exposes no API to observe cursor position, active selections, or open popovers. These actions are fundamentally unobservable by `zedkata-lsp`.

2. **Mutating Refactorings Require Semantic Language Servers:**
   - Symbol rename (`c d`) and quick-fix code actions (`g .`) do modify buffer text.
   - However, they **strictly require an active semantic language server** (such as `rust-analyzer`, `pyright`, `gopls`, or `vtsls`) with project-wide symbol indexes, ASTs, and `renameProvider` / `codeActionProvider` implementations.
   - In student drill buffers (`*.kata`), the only attached language server is `zedkata-lsp`. `zedkata-lsp` deliberately acts solely as a drill observer and progress coordinator; it does not implement language-specific AST compilation or rename providers.
   - Invoking `c d` or `g .` in a `.kata` file fails with *"No rename provider available"* or produces no actions.
   - Running drills in real language files (`.rs`, `.py`) would bypass `zedkata-lsp` (which only binds to `language = "Zedkata"` per `extension.toml`), and would introduce hard dependencies on external toolchain installations on student machines.

Per **REQ-25** and **REQ-29**, LSP-backed actions in Zed are therefore classified as **unscored guided walkthroughs** with concrete reproducible examples, verified instructions, and self-reported completion semantics.

---

## 2. Zed's LSP Keybinding Architecture

Zed integrates language server intelligence directly into its Vim mode emulation. Rather than forcing users to reach for modifier keys, mouse clicks, or command palette strings, Zed maps semantic actions to ergonomic Normal-mode sequences.

### 2.1 Verified Normal-Mode LSP Shortcut Table

Audited against Zed 1.19.2 (`assets/keymaps/vim.json` and `https://zed.dev/docs/vim.md#language-server`):

| Action | Vim Mode Shortcut | Standard / Alternate Shortcut | Zed Action Identifier | Notes |
|:---|:---:|:---:|:---|:---|
| **Rename Symbol** | `c d` | `space r`, `F2` | `editor::Rename` | Mnemonic: *"change definition"*. Triggers inline rename prompt. |
| **Code Actions** | `g .` | `space .` | `editor::ToggleCodeActions` | Opens quick-fix suggestions for diagnostic or cursor. |
| **Go to Definition** | `g d` | `F12` | `editor::GoToDefinition` | Jumps to symbol declaration/definition. |
| **Definition in Split** | `<ctrl-w> g d` | — | `editor::GoToDefinitionSplit` | Opens definition in adjacent pane split. |
| **Go to Declaration** | `g D` | — | `editor::GoToDeclaration` | Jumps to explicit header/declaration. |
| **Go to Type Definition** | `g y` | — | `editor::GoToTypeDefinition` | Jumps to the struct/enum/type definition. |
| **Go to Implementation** | `g I` | — | `editor::GoToImplementation` | Jumps to trait `impl` block. |
| **Find All References** | `g A` | `shift-F12` | `editor::FindAllReferences` | Opens multibuffer listing all references. |
| **Next Diagnostic** | `g ]` or `] d` | `F8` | `editor::GoToDiagnostic` | Jumps to next warning/error. |
| **Previous Diagnostic** | `g [` or `[ d` | `shift-F8` | `editor::GoToPrevDiagnostic` | Jumps to previous warning/error. |
| **Hover Documentation** | `g h` | `space k`, `K` | `editor::ShowHover` | Opens Markdown signature & docstring popover. |
| **Symbol in File** | `g s` | `ctrl-t` | `outline::Toggle` | Fuzzy-searches outline symbols in current file. |
| **Symbol in Project** | `g S` | `ctrl-T` | `project_symbols::Toggle` | Fuzzy-searches symbols across whole workspace. |
| **Diagnostics Panel** | `:clist` | — | `diagnostics::Deploy` | Vim ex-command opening the diagnostics panel. |

---

## 3. Concrete Reproducible Walkthrough Scenarios

To practice these workflows, open the sample fixture in Zed:
```bash
zed lessons/fixtures/walkthroughs/lsp/sample_service.rs
```

*(Ensure the language server for Rust, such as `rust-analyzer`, is installed or enabled in Zed when working with `.rs` files).*

### Scenario 1: Multi-Site Symbol Rename (`c d`)

**Objective:** Safely rename a struct method and its invocation call sites across the buffer in a single atomic refactor.

1. In Normal mode, move the cursor onto the method definition `record_metric` on line 49:
   ```rust
   pub fn record_metric(&mut self, name: &str, value: f64, level: MetricLevel) {
   ```
2. Press `c d` (or `space r`).
3. **Observed Behavior:**
   - Zed opens an inline text input box highlighting `record_metric`.
   - All references to `record_metric` in the file (e.g. lines 73, 74, 75) are highlighted with simultaneous linked edit cursors.
4. Type `ingest_metric` and press <kbd>Enter</kbd>.
5. **Outcome:**
   - The method definition on line 49 and every call site on lines 73, 74, and 75 update simultaneously to `ingest_metric`.
   - No search-and-replace regex or multi-cursor gymnastics are required.
   - To cancel a rename prompt without applying changes, press <kbd>Esc</kbd>.

### Scenario 2: Quick-Fix Code Actions (`g .`)

**Objective:** Inspect and apply automated compiler quick-fixes using language server code actions.

1. Navigate to line 21:
   ```rust
   pub struct TelemetryMetric {
   ```
2. Place the cursor on `TelemetryMetric` or on a diagnostic warning (such as an unused import).
3. Press `g .` (or `space .`).
4. **Observed Behavior:**
   - A contextual popup menu appears anchored below the cursor, displaying available refactoring actions (e.g. *"Generate `new`"*, *"Derive Serialize/Deserialize"*, *"Qualify path"*).
5. Use `j` and `k` to navigate options, and press <kbd>Enter</kbd> to apply the selected action.
6. In Insert mode, you can also trigger this menu with `ctrl-x ctrl-l`.

### Scenario 3: Inline Diagnostic Navigation (`g ]` / `g [`) and Diagnostics List (`:clist`)

**Objective:** Navigate compiler errors without touching the mouse or guessing line numbers.

1. Introduce a deliberate type error on line 73 by passing a string instead of a float:
   ```rust
   client.record_metric("cpu_usage_pct", "forty-two", MetricLevel::Info);
   ```
2. Return to Normal mode (<kbd>Esc</kbd>).
3. **Observed Behavior:**
   - Zed displays an inline diagnostic block directly beneath line 73 with a red indicator and compiler explanation: `expected f64, found &str`.
4. Move your cursor to the top of the file (`gg`).
5. Press `g ]` (or `] d`).
   - The cursor jumps directly to the diagnostic error on line 73.
6. Press `g [` (or `[ d`).
   - The cursor jumps backward to any prior diagnostic warning.
7. Open the project-wide diagnostics view:
   - Type `:clist` and press <kbd>Enter</kbd>.
   - A panel opens listing all errors and warnings across the workspace. Press <kbd>Enter</kbd> on any entry to jump directly to it.
8. Press `u` in the editor to undo the deliberate error and restore a clean buffer.

### Scenario 4: Definition Jumping (`g d`, `<ctrl-w> g d`) and Type Definition (`g y`)

**Objective:** Traverse code hierarchy rapidly using semantic jumps.

1. Navigate to line 66 in `initialize_client`:
   ```rust
   let mut client = TelemetryClient::new();
   ```
2. Position the cursor on `TelemetryClient` and press `g y` (Go to Type Definition).
   - Cursor jumps to the struct definition `pub struct TelemetryClient` on line 28.
3. Position the cursor on `new` and press `<ctrl-w> g d` (Go to Definition in Split).
   - Zed automatically splits the editor pane and opens the constructor definition `pub fn new() -> Self` in the new split pane!
4. Close the split pane with `<ctrl-w> c` or `:q`.

### Scenario 5: Rich Hover Documentation (`g h`) and Find All References (`g A`)

**Objective:** Inspect type signatures and locate every call site without leaving Normal mode.

1. Navigate to line 59:
   ```rust
   pub fn compute_average(&self) -> f64 {
   ```
2. Position the cursor over `compute_average` and press `g h` (or `K` / `space k`).
3. **Observed Behavior:**
   - A markdown popover appears displaying the full signature `pub fn compute_average(&self) -> f64` and the docstring: *"Compute the arithmetic mean of all recorded metric values."*
   - Press <kbd>Esc</kbd> or move the cursor to dismiss the hover.
4. Position the cursor over `MetricLevel` on line 12 and press `g A`.
5. **Observed Behavior:**
   - Zed opens a Multibuffer pane listing every occurrence of `MetricLevel` across the entire project (definition, struct field type, function signature, and enum variant usages).

---

## 4. Self-Reported Completion Verification Checklist

Per **REQ-29**, guided walkthroughs are verified through self-reported confirmation rather than false automated drill scores. Confirm each of the following before considering this module complete:

- [ ] **Rename (`c d`):** Successfully renamed a symbol across multiple lines simultaneously without search/replace regex.
- [ ] **Code Action (`g .`):** Opened the code action menu and applied or reviewed a compiler quick-fix.
- [ ] **Diagnostic Jumping (`g ]` / `g [`):** Jumped forward and backward through diagnostics using single-motion Normal mode commands.
- [ ] **Diagnostic Panel (`:clist`):** Opened the workspace diagnostics panel via ex-command.
- [ ] **Definition & Split (`g d`, `<ctrl-w> g d`):** Jumped directly to symbol definitions in the current buffer and in an adjacent split pane.
- [ ] **Hover (`g h`):** Inspected signature and docstrings with the inline hover popover.
- [ ] **References (`g A`):** Inspected all project usages of a symbol in a reference pane.
