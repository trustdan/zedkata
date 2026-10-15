# Open questions

Every claim this project depends on, tagged. Nothing here is settled by memory or by an
AI's confident tone — only by current Zed docs, Zed source, or an observation reproduced
on this machine.

**Re-verify anything older than ~3 months.** Zed ships weekly.

Legend: **VERIFIED** (with date + source) · **ASSUMED** (load-bearing, unchecked) ·
**OPEN** (actively blocking)

---

## OQ-01 — `didChange` granularity · VERIFIED (2026-09-13)

Checked 2026-09-13 on Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`) on Windows.

Zed sends discrete, incremental `didChange` notifications with UTF-16 ranges without
coalescing or debouncing into full-document replacements under normal typing pace.

**Empirical evidence:**
- **Operator deletion (`dw`)**: Consistently emitted as a single notification deleting the
  exact word span `(1,17)–(1,23)` (Cases 01 & 02; sequence 32 and sequence 40).
- **Character deletion (Backspaces)**: Emitted as 5 separate notifications arriving
  ~180 ms apart, each deleting 1 character at decrementing column offsets (Case 03;
  sequences 52, 54, 56, 58, 60).
- Retained raw JSON-RPC captures: [`tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl),
  [`tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-02-dw-repeat.jsonl),
  and [`tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl`](../tests/fixtures/lsp/2026-09-13-matrix-03-backspace.jsonl).
- See detailed report in [matrix results](evidence/2026-09-13-matrix-results.md) and
  gate decision in [ADR 0003](adr/0003-phase-0-feasibility-gate.md).

Blocks: unblocked. Technique scoring, `par_edits`, and `forbid` rules are supported.

---

## OQ-02 — Registering a custom language server · VERIFIED (documented dev route)

Checked 2026-09-13 against current unversioned official docs. Local Zed is 1.19.2
(`df181c6f58d02677b385fa947d6bfde6d3530078`).

The [development guide](https://zed.dev/docs/extensions/developing-extensions)
documents installing a local dev extension without publishing it. The
[language guide](https://zed.dev/docs/extensions/languages#language-servers) documents
language metadata, manifest server registration, and a Rust `language_server_command`
implementation returning a command with arguments and environment.

The [settings guide](https://zed.dev/docs/configuring-languages) covers existing
language associations and server overrides; it does not establish a settings-only
route for a new language/server pair. Use the documented dev-extension route.

**Runtime checked 2026-09-13:** the dev extension loaded in Zed 1.19.2 and launched
the probe. Captured initialize/open/change/close and outgoing diagnostics; Dan
confirmed the visible diagnostic. See [connection evidence](evidence/2026-09-13-connection.md).

**Not established:** whether another settings-only route exists.

---

## OQ-03 — Diagnostic round-trip latency · VERIFIED (2026-09-13)

Checked 2026-09-13 on Zed 1.19.2.
- Instrumented server dispatch time after `didChange` receipt is 79 µs to 1.04 ms.
- Visible rendering of inline diagnostics confirmed on hover in Zed editor (ZK-003 connection check).
- Passes the <300 ms channel gate and <100 ms target. Inline diagnostics remain a viable primary feedback channel.

---

## OQ-04 — Extension API cannot observe buffer/cursor/selections · VERIFIED

Checked 2026-09-13. Zed's extension surface covers languages, grammars, language
servers, debug adapters, themes, icon themes, snippets, and MCP servers. There is an
open feature request (zed-industries/zed discussion #53131, Apr 2026) asking for an API
to observe buffer lifecycle events with cursor and selection awareness; it has one
comment and no commitment.

Zed's own extension blog post states there is no support for modifying the UI to create
new panels, making arbitrary HTTP requests, or arbitrary filesystem access.

**Treat as permanent.** All design flows from this.

---

## OQ-05 — Fake-LSP-as-observer is a workable seam · VERIFIED (by precedent)

Checked 2026-09-13. A Zed Discord-presence extension was built by shipping a language
server that reported which files the user had open. Confirms Zed will speak LSP to an
arbitrary binary and that the channel carries document lifecycle information.

Does **not** confirm granularity — that is OQ-01.

---

## OQ-06 — Extensions must be Rust → WASM · VERIFIED

Checked 2026-09-13, zed.dev extension pages. Functional parts are written in Rust,
compiled to WebAssembly, and the repo is linked as a submodule in
`zed-industries/extensions`.

---

## OQ-07 — Custom ACP agents are a supported install path · VERIFIED

Checked 2026-09-13. Zed supports External Agents over ACP, configured under
`agent_servers` in `settings.json` with `"type": "custom"` plus command, args, and env.
There is also an ACP Registry, opened with `zed: acp registry`.

---

## OQ-08 — Terminal hosting routes · VERIFIED

Checked 2026-09-13 and finalized under ZK-023.2 against Zed 1.19.2 on Windows 11.

Evaluated four terminal hosting routes across fullscreen/alt-screen buffer takeover,
dynamic resize with minimum 60x15 boundary handling, and clean restoration on exit/panic:

1. **Standard Zed Terminal Pane (`workspace: new terminal`, `Ctrl-~`):** **VERIFIED & RECOMMENDED.**
   Alternate screen buffer engages cleanly. ConPTY delivers `Event::Resize` signals smoothly.
   Exiting with `q`, `Esc`, `Ctrl-C`, or via error/panic restores cooked mode and original cursor
   cleanly without leaving escape artifacts in Zed's terminal buffer.
2. **Zed Center Terminal Tab (`workspace: new center terminal`):** **VERIFIED & RECOMMENDED.**
   Runs as a full workspace tab alongside editor buffers. Ideal for side-by-side split screen
   (exercise buffer on left, drill conductor on right).
3. **External Terminal Emulators (Windows Terminal, Alacritty, etc.):** **VERIFIED & RECOMMENDED.**
   Native execution with complete ConPTY support, ideal for multi-monitor setups.
4. **Assumed Terminal Thread Route (`agent: new thread` -> Terminal):** **NOT RECOMMENDED.**
   Terminal Threads live in Zed's narrow Agent Panel dock (`agent.dock`), defaulting to ~35-45
   columns which immediately triggers the `< 60x15` minimum dimension warning overlay. They
   cannot be zoomed as editor tabs, and their thread container remains retained in Zed's
   agent thread store after exit.

**Advertised supported routes:** Standard Zed terminal pane, center terminal tab, and external
terminals. See full report: [Supported terminal hosting routes evidence](evidence/terminal-hosting-routes.md).

---

## OQ-09 — ACP Rust SDK maturity · VERIFIED

Checked 2026-09-13. `agent-client-protocol` reached 1.0.0 in June 2026 alongside the
TypeScript SDK. Official SDKs exist for Rust, TypeScript, Python, Kotlin, and Java.
No Go SDK.

---

## OQ-10 — Can we place the student's cursor? · VERIFIED (2026-09-14)

Checked 2026-09-14 against the installed CLI on Zed 1.19.2
(`df181c6f58d02677b385fa947d6bfde6d3530078`), Windows 11.

`zed.exe --help` documents the syntax in its own positional-argument help — "Use
`path:line:column` syntax to open a file at the given line and column." This is the
installed binary's text, so it describes this exact version.

**Empirical evidence:**
- A path with spaces in both the directory and the file name, with a `:3:12` suffix
  appended, is accepted as a single argument — exit 0, no parse error and no
  truncated-path failure. The CLI splits the position suffix from the right.
- Dan confirmed the caret landed on line 3 at the `T` of `TARGET`, the exact
  character at column 12 of `LINE THREE TARGET COLUMN HERE`.
- **Both line and column are 1-based**, established by that character-exact landing.

Full report: [CLI cursor placement](evidence/2026-09-14-cli-cursor-placement.md).

Blocks: unblocked. ZK-018.2 may depend on CLI cursor placement.

Note this places the cursor at *launch* only. It does not let us read the cursor
later — that remains permanently unavailable per OQ-04.

---

## OQ-11 — Undo semantics on the wire · ASSUMED

Assumed that an undo produces a `didChange` that reverses the prior delta, and can
therefore be detected and excluded from the edit count. If undo is invisible or appears
as a full-document sync, `par_edits` is trivially gameable.

**Not closed by Phase 0.** The matrix run captured no deliberate undo: the restoration
at sequence 34 of Case 01 is
[explicitly recorded](evidence/2026-09-13-matrix-results.md) as consistent with the
discard flow and *not* evidence that `u` was pressed.

**What exists today.** `count_edit_units` cancels inverse edit pairs, covered by
`test_count_edit_units_undo_inversion` in `crates/zedkata-core/tests/scoring.rs`. That
test is **hand-constructed**, not replayed from a capture. It shows the counter handles
undo-shaped input; it does not show Zed emits that shape.

**To close:** capture `dw` then `u` on a real buffer, and confirm the second
notification is an insertion inverting the first rather than a full-document sync.
Until then, treat undo exclusion in scoring as resting on an unverified assumption.

**How to capture (added 2026-09-14).** `zedkata-lsp` writes raw traffic to a `.jsonl`
when `ZEDKATA_CAPTURE` is set before Zed launches — see
[capturing raw LSP traffic](DEVELOPMENT.md#capturing-raw-lsp-traffic). The Phase 0 probe
that produced the existing fixtures was retired in ZK-005.2, so this replaced it.

Three outcomes are possible, and they are not equally good:
- **Inverting insertion** at the deleted range — the assumed shape. `count_edit_units`
  already cancels it, and OQ-11 becomes VERIFIED.
- **Full-document sync** — undo is indistinguishable from retyping. `par_edits` is
  gameable by delete-undo-retry and ZK-011's undo handling needs an ADR before ZK-022
  calibrates budgets against it.
- **No notification at all** — undo is invisible to us; buffer state and our
  reconstruction diverge, which is worse than either of the above.

## OQ-12 — Zed vim mode capabilities and curriculum audit · VERIFIED

Audited 2026-10-15 against Zed 1.19.2, `assets/keymaps/vim.json`, and `crates/vim/src/object.rs`
under [ZK-024.1](BACKLOG.md#task-zk-024-1).

**Audit Findings:**
1. **Core Motions & Operators:** Full native support for word motions (`w`, `e`, `b`, `ge`,
   `W`, `E`, `B`), in-line find (`f`, `F`, `t`, `T`, `;`, `,`), and standard operators (`d`,
   `c`, `y`, `p`, `P`, `r`, `s`, `S`, `x`, `~`).
2. **Text Objects (`crates/vim/src/object.rs`):**
   - Quotes (`"`, `'`, `` ` ``) and brackets (`(`, `[`, `{`, `<`, `tag`) are natively supported.
   - Zed binds `"q"` to `vim::MiniQuotes` by default in `assets/keymaps/vim.json`.
   - `AnyQuotes`, `AnyBrackets`, and `MiniBrackets` are implemented in `crates/vim/src/object.rs`,
     with `AnyQuotes` and `AnyBrackets` available as alternative unbound actions.
   - Traditional delimiter text objects (`i"`, `a"`, `i(`, `a(`, `i{`, `a{`) work out-of-the-box.
3. **Surrounds & Extra Features:** Zed natively supports surround commands (`cs`, `ds`, `ys`),
   subword motions, and dot repeat (`.`).
4. **Registers & Macros:** Named registers (`"a` to `"z`), unnamed, clipboard, black hole,
   and macros (`q`, `@`, `@@`) are fully implemented and verified.
5. **Observation Boundaries:** Visual mode (`v`, `V`, `ctrl-v`) is fully supported in the
   editor, but LSP `didChange` notifications cannot observe visual selection state without
   cursor/keystroke APIs (OQ-04). Therefore, unobservable forbid rules (e.g. `visual_line_select`)
   are excluded from curriculum forbid tables per REQ-10 and enforced via Par edit budgets.
6. **Curriculum Classification:** Single-buffer text transformations are designed as scored
   katas; features without deterministic pass/fail buffer states (multibuffers, read-only LSP
   hops, project panel, agent panel) are classified as guided walkthroughs.

See full specification: [Curriculum Module Map](curriculum-module-map.md).

---

## OQ-13 — Zed keymap contexts and Vim mode scoping · VERIFIED (2026-10-15)

Audited 2026-10-15 against official Zed documentation (`https://zed.dev/docs/key-bindings.md`, `https://zed.dev/docs/vim.md`) under [ZK-026.2](BACKLOG.md#task-zk-026-2).

**Key Findings:**
1. **Context Hierarchy:** Zed organizes active UI contexts in a hierarchy tree (e.g., `Workspace` -> `Pane` -> `Editor`). Context expressions evaluate predicates at specific nodes:
   - Properties like `vim_mode` and `vim_operator` reside on the `Editor` node, not on `Workspace`.
   - Direct combinations like `"Editor && vim_mode == normal"` match the editor node.
   - Cross-level matching requires the ancestor operator (`>`), e.g. `"Workspace > Editor && vim_mode == normal"`.
   - Combining `"Workspace && vim_mode == normal"` directly will never match because the properties reside on different nodes.
2. **Modal Contexts & Expressions:**
   - `VimControl`: Convenience shorthand alias for `vim_mode == normal || vim_mode == visual || vim_mode == operator`.
   - `vim_mode == normal`: Normal mode only.
   - `vim_mode == visual`: Visual mode only.
   - `vim_mode == insert`: Insert mode only.
   - `vim_mode == replace`: Replace mode only.
   - `vim_mode == waiting`: Waiting for an arbitrary character argument (e.g. after `f`, `t`, `r`).
   - `vim_mode == operator`: Waiting for a motion or text-object target (e.g. after `d`, `c`, `y`).
   - `vim_operator`: Current pending operator symbol (e.g. `vim_operator == d`, `vim_operator == a || vim_operator == i`).
   - Standard boolean operators (`&&`, `||`, `!`) and grouping parentheses `(...)` are supported.
3. **Precedence Rules:**
   - Lower nodes in the tree take precedence over higher nodes (`Editor` beats `Workspace`).
   - At the same node level, bindings defined later take precedence (user `keymap.json` loads after built-in defaults).
   - Prefix conflicts (chords) wait on a 1-second timeout by default.
4. **Key Context Inspection:**
   - `dev: open key context view` in the command palette opens real-time inspection of active context tree nodes and attributes.
5. **Observability Boundary & Curriculum Classification:**
   - `keymap.json` edits and key context activations take place entirely in Zed's GPUI / client process and emit no LSP notifications (`didChange`, etc.).
   - Consequently, per REQ-24, REQ-25, and REQ-29, keymap context configuration cannot be an automatically verified buffer drill. It is strictly classified as a guided walkthrough with explicit, self-reported configuration outcomes.

---

## OQ-14 — Zed normal-mode LSP actions and observation boundaries · VERIFIED (2026-10-15)

Audited 2026-10-15 against Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`), official Zed documentation (`https://zed.dev/docs/vim.md#language-server`), and `crates/zedkata-lsp` under [ZK-027.1](BACKLOG.md#task-zk-027-1).

**Audit Findings & Keybindings:**
1. **Verified Normal-Mode LSP Shortcuts:**
   - **Symbol Rename:** `c d` (mnemonic: *"change definition"*; also `space r`, `F2`). Triggers inline symbol rename input box.
   - **Code Actions:** `g .` (also `space .`, `ctrl-x ctrl-l` in Insert mode). Opens context quick-fix popup menu.
   - **Go to Definition:** `g d` (also `<ctrl-w> g d` for definition in adjacent split pane).
   - **Go to Declaration:** `g D`.
   - **Go to Type Definition:** `g y`.
   - **Go to Implementation:** `g I`.
   - **Find All References:** `g A`. Opens multibuffer referencing all occurrences.
   - **Diagnostics Navigation:** `g ]` or `] d` (next diagnostic); `g [` or `[ d` (previous diagnostic).
   - **Diagnostics Panel:** `:clist` (ex-command opening the workspace diagnostics panel).
   - **Hover Documentation:** `g h` (also `K`, `space k`). Opens inline markdown signature/doc popover.
   - **Outline / Symbols:** `g s` (file outline); `g S` (workspace symbol search).
2. **Outcome Classification & Observability Boundaries (REQ-25, REQ-29):**
   - **Non-mutating navigation actions** (`g d`, `g D`, `g y`, `g I`, `g A`, `g ]`, `g [`, `g h`, `g s`, `g S`, `:clist`) change cursor position or open UI popovers, but produce zero buffer deltas. They emit no LSP `textDocument/didChange` notifications and are completely unobservable by `zedkata-lsp` (REQ-10, OQ-04).
   - **Mutating actions** (`c d` symbol rename, `g .` code actions) alter buffer text, but strictly require an underlying semantic language server (`rust-analyzer`, `pyright`, `gopls`, etc.) providing `renameProvider` or `codeActionProvider`.
   - In student exercise buffers (`*.kata`), the only attached language server is `zedkata-lsp`. `zedkata-lsp` provides document sync observation only and does not implement semantic language providers; invoking `c d` or `g .` in `.kata` files yields "No rename provider available" or no actions.
   - Real language files (`.rs`, `.py`) run against external toolchains that vary by student environment and are unobserved by `zedkata-lsp` (which binds exclusively to `Zedkata` files per `extension.toml`).
3. **Curriculum Classification:**
   - Per REQ-25 and REQ-29, all normal-mode LSP actions are formally classified as **Guided Walkthroughs** with complete reproducible fixtures (`lessons/fixtures/walkthroughs/lsp/sample_service.rs`), verified instructions (`docs/walkthroughs/lsp-actions.md`), and self-reported verification criteria, rather than false automated drill scores.

---

## OQ-15 — Zed Project Panel netrw bindings and observation boundaries · VERIFIED (2026-10-15)

Audited 2026-10-15 against Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`), official Zed documentation (`https://zed.dev/docs/project-panel`), and native keymap `assets/keymaps/vim.json` under [ZK-027.2](BACKLOG.md#task-zk-027-2).

**Audit Findings & Keybindings:**
1. **Native Netrw Compatibility (`ProjectPanel && not_editing`):**
   - **Focus Handoff:** `ctrl-shift-e` (macOS `cmd-shift-e`) toggles panel focus (`project_panel::ToggleFocus`). In the panel, `<escape>` returns focus instantly to the active editor pane (`vim::ToggleProjectPanelFocus`).
   - **Tree Traversal:** `j` / `down` and `k` / `up` for vertical motion; `h` collapses folder or ascends to parent; `l` expands folder; `-` ascends directly to parent directory (`project_panel::SelectParent`); `g g` and `G` jump to tree extremities; `}` and `{` jump between folders.
   - **Multi-Pane Splits:** `v` opens file in vertical split (`project_panel::OpenSplitVertical`); `o` opens file in horizontal split (`project_panel::OpenSplitHorizontal`); `enter` or `t` opens permanent tab; `p` or `<space>` opens ephemeral preview tab.
   - **Netrw File Manipulation:** `%` creates new file (`project_panel::NewFile`); `d` creates new directory (`project_panel::NewDirectory`); `R` (`shift-r`) renames with stem pre-selected (`project_panel::Rename`); `D` (`shift-d`) deletes entry (`project_panel::Delete`); `<delete>` moves to system trash; `ctrl-z` undoes panel file creation, rename, and trash operations.
   - **Diagnostics & Git:** `] d` / `[ d` navigates files with diagnostics; `] c` / `[ c` navigates files with git modifications; `z d` opens visual diff when 2 files are marked.
2. **Outcome Classification & Observability Boundaries (REQ-25, REQ-29):**
   - **Panel navigation and focus transitions** (`ctrl-shift-e`, `escape`, `j`, `k`, `h`, `l`, `-`, `g g`, `G`) are purely GPUI view states and emit **zero buffer modifications** and zero LSP `textDocument/didChange` deltas.
   - **File operations** (`%`, `d`, `R`, `D`) mutate disk files directly rather than emitting incremental editing deltas within an active `.kata` buffer.
   - `zedkata-lsp` has no access to tree cursor coordinates or filesystem manipulation hooks (OQ-04, REQ-10).
3. **Curriculum Classification:**
   - Per REQ-25 and REQ-29, project panel navigation and netrw workflows are formally classified as **Guided Walkthroughs** (`[[walkthrough]] id = "project-panel-walkthrough"`) with complete reproducible multi-file fixtures (`lessons/fixtures/walkthroughs/project_panel/`), verified recipes (`docs/walkthroughs/project-panel.md`), and self-reported verification criteria.


