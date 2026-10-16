use serde::{Deserialize, Serialize};
use std::path::Path;
use zedkata_core::session::SessionState;

/// An unscored guided walkthrough record with verified steps and explicit self-report criteria.
///
/// Adheres strictly to REQ-25 and REQ-29: non-observable editor workflows and conceptual comparisons
/// are presented with verified steps and self-reported completion without entering drill scoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkthroughRecord {
    pub id: String,
    pub lesson_id: String,
    pub title: String,
    pub prompt: String,
    pub fixture: Option<String>,
    pub steps: Vec<String>,
    pub checklist: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WalkthroughError {
    #[error("Walkthrough '{0}' not found. Run 'zedkata-agent walkthrough list' to see available walkthroughs.")]
    NotFound(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl WalkthroughRecord {
    /// Returns all 10 curated curriculum walkthrough records across feature modules and conceptual comparisons.
    pub fn all() -> Vec<Self> {
        vec![
            // Feature Walkthroughs (REQ-25)
            Self {
                id: "lsp-actions-walkthrough".to_string(),
                lesson_id: "zed-01-lsp-actions".to_string(),
                title: "Normal-mode LSP intelligence in Zed".to_string(),
                prompt: "Demonstrate Zed's native normal-mode language server capabilities using `sample_service.rs`.".to_string(),
                fixture: Some("lessons/fixtures/walkthroughs/lsp/sample_service.rs".to_string()),
                steps: vec![
                    "Open the sample service in Zed: `zed lessons/fixtures/walkthroughs/lsp/sample_service.rs`.".to_string(),
                    "Multi-site rename: position cursor on `ProcessOrder` on line 12 and press `c d`. Type `HandleOrder` and press `<enter>`. Verify all call sites update synchronously across the buffer.".to_string(),
                    "Code action: navigate to the unused import on line 2, press `g .` to summon code actions, and select quick-fix with `<enter>`.".to_string(),
                    "Diagnostic navigation: press `g ]` to jump forward to the compiler warning and `g [` to return. Open the diagnostic list with `:clist`.".to_string(),
                    "Type inspection & definition: press `g h` on any function to view hover docs, and `g d` (or `<ctrl-w> g d` for split) to jump to its definition.".to_string(),
                ],
                checklist: vec![
                    "Multi-site rename executed with `c d` and committed synchronously across all references.".to_string(),
                    "Code action palette summoned with `g .` and quick-fix previewed/applied.".to_string(),
                    "Inline diagnostics traversed using `g ]` / `g [` and inspected with `:clist`.".to_string(),
                    "Hover documentation inspected with `g h` and definition jumped to with `g d`.".to_string(),
                ],
            },
            Self {
                id: "project-panel-walkthrough".to_string(),
                lesson_id: "zed-02-project-panel".to_string(),
                title: "Netrw-style project panel navigation and file management".to_string(),
                prompt: "Demonstrate Zed's native netrw-compatible project panel keyboard workflows using the sample project tree.".to_string(),
                fixture: Some("lessons/fixtures/walkthroughs/project_panel/".to_string()),
                steps: vec![
                    "Launch Zed on the sample project: `zed lessons/fixtures/walkthroughs/project_panel`.".to_string(),
                    "Toggle project panel focus: press `ctrl-shift-e` to enter the panel and `<escape>` to return to the editor.".to_string(),
                    "Directory traversal: navigate the tree using netrw vim keys `j` (down), `k` (up), `h` (collapse), `l` (expand), and `-` (parent folder).".to_string(),
                    "Opening into splits: press `v` on a file to open in a vertical split, `o` for a horizontal split, or `enter` for a tab.".to_string(),
                    "File management: press `%` to create a new file, `d` to create a directory, `shift-r` to rename, and `ctrl-z` to undo file operations.".to_string(),
                    "Git & diagnostic navigation: jump between modified Git files with `] c` / `[ c`, and between files with diagnostics using `] d` / `[ d`.".to_string(),
                ],
                checklist: vec![
                    "Project panel focused with `ctrl-shift-e` and returned with `<escape>`.".to_string(),
                    "Tree navigated using `j`, `k`, `h`, `l`, and `-` without mouse.".to_string(),
                    "Files opened into multi-pane splits using `v` and `o`.".to_string(),
                    "Created file with `%`, directory with `d`, and renamed with `shift-r`.".to_string(),
                    "Navigated modified Git items using `] c` and `[ c`.".to_string(),
                ],
            },
            Self {
                id: "multibuffer-walkthrough".to_string(),
                lesson_id: "zed-03-multibuffer".to_string(),
                title: "Multibuffer project search and cross-file editing".to_string(),
                prompt: "Demonstrate multibuffer project search, excerpt navigation, and simultaneous cross-file editing.".to_string(),
                fixture: Some("lessons/fixtures/walkthroughs/multibuffer/".to_string()),
                steps: vec![
                    "Open the sample project in Zed: `zed lessons/fixtures/walkthroughs/multibuffer`.".to_string(),
                    "Deploy project search: press `g /` (or `pane::DeploySearch` / `ctrl-shift-f`), type `BillingTier`, and press `<enter>` to populate the multibuffer.".to_string(),
                    "Excerpt navigation: press `n` to jump to the next matching excerpt and `N` for previous match.".to_string(),
                    "Expand context: press `shift-enter` on an excerpt boundary to expand surrounding lines, or `alt-enter` to open the file in a dedicated tab.".to_string(),
                    "Multi-cursor refactoring: use `ctrl-d` to select matching occurrences across different files, then type `c w SubscriptionTier <escape>`.".to_string(),
                    "Atomic save: press `ctrl-s` to save all modified excerpts and underlying files synchronously.".to_string(),
                ],
                checklist: vec![
                    "Multibuffer generated from project-wide search (`g /`).".to_string(),
                    "Search excerpts traversed using `n` and `N`.".to_string(),
                    "Context expanded using `shift-enter` and tab breakout tested with `alt-enter`.".to_string(),
                    "Simultaneous multi-cursor editing executed across file boundaries.".to_string(),
                    "Atomic multi-file save committed with `ctrl-s`.".to_string(),
                ],
            },
            Self {
                id: "tasks-and-debugger-walkthrough".to_string(),
                lesson_id: "zed-04-tasks-and-debugger".to_string(),
                title: "Workspace tasks and DAP debugging in Zed".to_string(),
                prompt: "Demonstrate workspace task automation, variable interpolation, and DAP debugging.".to_string(),
                fixture: Some("lessons/fixtures/walkthroughs/tasks_and_debugger/".to_string()),
                steps: vec![
                    "Open the fixture workspace in Zed: `zed lessons/fixtures/walkthroughs/tasks_and_debugger`.".to_string(),
                    "Spawn task: open command palette (`ctrl-shift-p`), trigger `task: spawn`, and choose `test: run-calculator-tests`.".to_string(),
                    "Task rerun: trigger `task: rerun` to repeat the last executed task in the terminal dock.".to_string(),
                    "Set breakpoint: open `src/main.rs`, navigate to line 18, and toggle breakpoint using `space shift-g b`.".to_string(),
                    "Start debugger: launch DAP debugging with `debugger: start` or `F4`.".to_string(),
                    "Execution stepping: step over with `F8`, step into with `F7`, step out with `F9`, and inspect locals in the debug dock.".to_string(),
                ],
                checklist: vec![
                    "Workspace task spawned via `task: spawn` and rerun with `task: rerun`.".to_string(),
                    "Variable interpolations (`$ZED_FILE`, `$ZED_WORKTREE_ROOT`) observed in `.zed/tasks.json`.".to_string(),
                    "Breakpoint set in source buffer using keybinding.".to_string(),
                    "DAP debugger started and stepping executed (`F8`, `F7`, `F9`).".to_string(),
                ],
            },
            Self {
                id: "agent-panel-walkthrough".to_string(),
                lesson_id: "zed-05-agent-panel".to_string(),
                title: "Zed Agent Panel and Agent Client Protocol (ACP)".to_string(),
                prompt: "Demonstrate the Zed Agent Panel and ACP conversational interactions.".to_string(),
                fixture: Some("lessons/fixtures/walkthroughs/agent_panel/".to_string()),
                steps: vec![
                    "Open the Agent Panel fixture workspace: `zed lessons/fixtures/walkthroughs/agent_panel`.".to_string(),
                    "Focus Agent Panel: press `ctrl-?` (or run `agent: focus` from the command palette).".to_string(),
                    "Inspect `.zed/settings.json`: verify `agent_servers.zedkata` configuration registering `zedkata-agent --acp`.".to_string(),
                    "Send conversational query: type `guide me through lsp-actions` or `explain my last attempt`.".to_string(),
                    "Observe response: verify the Zedkata agent responds with bounded, grounded context without hallucinating unobserved keystrokes.".to_string(),
                ],
                checklist: vec![
                    "Agent Panel focused using `ctrl-?` or command palette.".to_string(),
                    "`zedkata` custom ACP server registration verified in `settings.json`.".to_string(),
                    "Conversational prompt sent and stream response received.".to_string(),
                    "Bounded context contract and grounded feedback observed.".to_string(),
                ],
            },

            // Conceptual & Text Object Walkthroughs (REQ-23, REQ-24)
            Self {
                id: "mini-vs-any".to_string(),
                lesson_id: "motions-03-textobjects".to_string(),
                title: "When MiniBrackets disagrees with AnyBrackets".to_string(),
                prompt: "Demonstrate the distinction between delimiter-agnostic quotes/brackets (`AnyQuotes`/`AnyBrackets`) and tree-sitter context matching (`MiniQuotes`/`MiniBrackets`).".to_string(),
                fixture: Some("lessons/fixtures/motions-03/nested.start.kata".to_string()),
                steps: vec![
                    "Open the nested brackets fixture: `lessons/fixtures/motions-03/nested.start.kata` in Zed.".to_string(),
                    "Position cursor on line 11 inside the nested closure parameter: `filter_map(|(id, token)| ...)`.".to_string(),
                    "Test lexical delimiter selection (AnyBrackets): execute `ci)` or `ci(`. Notice that AnyBrackets selects strictly within the nearest lexical bracket pair.".to_string(),
                    "Undo changes with `u`, then test syntax-tree context matching (MiniBrackets / mini.ai): execute `ciq` or `cib`.".to_string(),
                    "Compare behavior: `MiniBrackets` resolves nested scopes based on grammar AST boundaries, while `AnyBrackets` targets the closest lexical delimiter characters regardless of syntax structure.".to_string(),
                ],
                checklist: vec![
                    "Observed lexical delimiter selection using `ci)` / `ci(`.".to_string(),
                    "Observed syntax-aware tree-sitter selection using `ciq` / `cib`.".to_string(),
                    "Understood when to reach for delimiter-agnostic pairs vs tree-sitter context.".to_string(),
                ],
            },
            Self {
                id: "keymap-contexts".to_string(),
                lesson_id: "operators-01-edit".to_string(),
                title: "Keymap contexts and modal binding customization".to_string(),
                prompt: "Configure and inspect custom keybindings in Zed using context expressions.".to_string(),
                fixture: None,
                steps: vec![
                    "Open Zed keymap configuration: press `ctrl-shift-p` and choose `zed: open keymap`.".to_string(),
                    "Add normal-mode leader shortcut: `{ \"context\": \"Editor && vim_mode == normal\", \"bindings\": { \"space w\": \"workspace::Save\" } }`.".to_string(),
                    "Add insert-mode fast escape shortcut: `{ \"context\": \"Editor && vim_mode == insert\", \"bindings\": { \"j k\": \"vim::NormalBefore\" } }`.".to_string(),
                    "Inspect active context trees: run `dev: open key context view` from the command palette.".to_string(),
                    "Verify bindings: open an editor buffer and test `<space> w` to save and `j k` to escape insert mode.".to_string(),
                ],
                checklist: vec![
                    "Scoped normal-mode binding configured with `Editor && vim_mode == normal`.".to_string(),
                    "Insert-mode fast escape chord configured with `Editor && vim_mode == insert`.".to_string(),
                    "Active context hierarchy inspected live using `dev: open key context view`.".to_string(),
                ],
            },
            Self {
                id: "exchange-and-surround".to_string(),
                lesson_id: "operators-01-edit".to_string(),
                title: "Text exchange vs surround operators".to_string(),
                prompt: "Demonstrate the difference between external plugin emulation (vim-exchange) and Zed's built-in native modal operators (vim-surround).".to_string(),
                fixture: Some("lessons/fixtures/operators-01/indent-line-block.start.kata".to_string()),
                steps: vec![
                    "Understand Zed's operator boundaries: `vim-exchange` ('cx') is not bundled in Zed.".to_string(),
                    "Practice native cut-and-paste text swapping: delete with `dd` or `dw`, move cursor, and put with `p` or `P`.".to_string(),
                    "Test native built-in surrounds: position cursor inside a quoted string and execute `cs\"'` to change delimiters to single quotes.".to_string(),
                    "Test surround deletion: execute `ds'` to strip delimiters.".to_string(),
                    "Test visual surround addition: select text in visual mode (`v`) and press `S)` to wrap in parentheses.".to_string(),
                ],
                checklist: vec![
                    "Practiced native modal cut-and-paste text swapping (`dd` / `dw` + `p`).".to_string(),
                    "Changed delimiter surroundings using `cs\"'`.".to_string(),
                    "Deleted delimiter surroundings using `ds'`.".to_string(),
                    "Wrapped visual selection with surround `S)`.".to_string(),
                ],
            },
            Self {
                id: "single-line-find-scope".to_string(),
                lesson_id: "motions-02-find".to_string(),
                title: "Single-line find scope and repeat motions in Zed".to_string(),
                prompt: "Demonstrate Zed's single-line character search confinement (`\"multiline\": false`) and repeat motions.".to_string(),
                fixture: Some("lessons/fixtures/motions-02/change-till-char.start.kata".to_string()),
                steps: vec![
                    "Open a buffer with multiple punctuation marks on a single line.".to_string(),
                    "Jump forward to a target character with `f<char>` (or `t<char>` to land before it).".to_string(),
                    "Repeat the search motion forward along the line using `;`.".to_string(),
                    "Reverse search direction backward along the line using `,`.".to_string(),
                    "Observe boundary confinement: notice that pressing `;` at the end of the line stops at the line edge without jumping to subsequent lines.".to_string(),
                ],
                checklist: vec![
                    "Jumped to character using `f` / `t`.".to_string(),
                    "Repeated forward find motion using `;`.".to_string(),
                    "Reversed find motion backward using `,`.".to_string(),
                    "Confirmed find motions never cross line boundaries unexpectedly.".to_string(),
                ],
            },
            Self {
                id: "diagnostic-navigation".to_string(),
                lesson_id: "modal-00-baseline".to_string(),
                title: "Inline diagnostics and error jumping in Zed".to_string(),
                prompt: "Demonstrate Zed's native inline diagnostic rendering and error traversal.".to_string(),
                fixture: Some("lessons/fixtures/modal-00/swap-words.start.kata".to_string()),
                steps: vec![
                    "Open any Rust buffer in Zed.".to_string(),
                    "Introduce a deliberate syntax error (e.g. remove a required semicolon or insert an invalid token).".to_string(),
                    "Observe Zed's inline diagnostic display appearing directly beneath the problematic line.".to_string(),
                    "Jump between diagnostics using `g ]` (next diagnostic) and `g [` (previous diagnostic).".to_string(),
                    "Undo the modification with `u` and verify the inline diagnostic disappears.".to_string(),
                ],
                checklist: vec![
                    "Observed inline compiler diagnostic rendering.".to_string(),
                    "Navigated forward to diagnostic with `g ]`.".to_string(),
                    "Navigated backward to diagnostic with `g [`.".to_string(),
                    "Reverted buffer with `u` and observed diagnostic clearance.".to_string(),
                ],
            },
        ]
    }

    /// Finds a walkthrough record by ID (case-insensitive).
    pub fn find(id: &str) -> Option<Self> {
        let needle = id.trim().to_lowercase();
        Self::all()
            .into_iter()
            .find(|w| w.id.to_lowercase() == needle || w.id.to_lowercase().contains(&needle))
    }

    /// Formats detailed walkthrough instructions and self-reported verification checklist adhering to REQ-29.
    pub fn format_detail(&self, is_completed: bool) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Guided Walkthrough: {}\n\n", self.title));
        out.push_str(&format!("- **Walkthrough ID**: `{}`\n", self.id));
        out.push_str(&format!("- **Curriculum Lesson**: `{}`\n", self.lesson_id));

        let status_badge = if is_completed {
            "[✓] COMPLETED (Self-Reported)"
        } else {
            "[ ] PENDING (Not Yet Completed)"
        };
        out.push_str(&format!("- **Completion Status**: {}\n", status_badge));
        out.push_str(
            "- **Scoring Classification**: Unscored guided walkthrough (REQ-25 / REQ-29)\n\n",
        );

        out.push_str("## Overview\n");
        out.push_str(&self.prompt);
        out.push_str("\n\n");

        if let Some(fixture) = &self.fixture {
            out.push_str("## Sample Fixture Workspace\n");
            out.push_str(&format!("`{}`\n\n", fixture));
        }

        out.push_str("## Verified Step-by-Step Instructions\n");
        for (i, step) in self.steps.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, step));
        }
        out.push('\n');

        out.push_str("## Explicit Self-Report Verification Checklist (REQ-29)\n");
        out.push_str("Confirm each item below before considering this walkthrough complete:\n");
        for item in &self.checklist {
            let mark = if is_completed { "x" } else { " " };
            out.push_str(&format!("- [{}] {}\n", mark, item));
        }
        out.push('\n');

        out.push_str("## Completion Actions\n");
        if is_completed {
            out.push_str(&format!(
                "- To reset completion status: `zedkata-agent walkthrough reset {}`\n",
                self.id
            ));
        } else {
            out.push_str(&format!(
                "- To confirm and record completion: `zedkata-agent walkthrough complete {}`\n",
                self.id
            ));
            out.push_str(
                "- Or confirm conversationally in the Zed Agent Panel by saying: `complete walkthrough ` followed by the ID.\n",
            );
        }

        out
    }

    /// Formats a single line entry for `zedkata-agent walkthrough list`.
    pub fn format_list_entry(&self, is_completed: bool) -> String {
        let badge = if is_completed { "[✓]" } else { "[ ]" };
        let fixture_str = self
            .fixture
            .as_deref()
            .map(|f| format!(" (fixture: {})", f))
            .unwrap_or_default();
        format!(
            "{} {:<30} [{}]{}",
            badge, self.id, self.lesson_id, fixture_str
        )
    }
}

/// Formats the complete walkthrough catalog listing.
pub fn format_walkthrough_list(records: &[WalkthroughRecord], session: &SessionState) -> String {
    let mut out = String::new();
    out.push_str("Zedkata Guided Walkthroughs (Unscored per REQ-25 / REQ-29):\n\n");

    let completed_count = records
        .iter()
        .filter(|w| session.is_walkthrough_completed(&w.id))
        .count();

    out.push_str(&format!(
        "Progress: {} / {} completed\n\n",
        completed_count,
        records.len()
    ));

    out.push_str("  Status ID                             Lesson\n");
    out.push_str("  ------ ------------------------------ -------------------------\n");

    for w in records {
        let completed = session.is_walkthrough_completed(&w.id);
        out.push_str(&format!("  {}\n", w.format_list_entry(completed)));
    }

    out.push_str("\nCommands:\n");
    out.push_str("  zedkata-agent walkthrough show <ID>     # Show verified steps & checklist\n");
    out.push_str("  zedkata-agent walkthrough complete <ID> # Mark self-reported completion\n");
    out.push_str("  zedkata-agent walkthrough reset <ID>    # Reset completion status\n");

    out
}

/// Marks a walkthrough as completed in session.json, saving atomically.
pub fn mark_completed_in_session(
    session_path: &Path,
    walkthrough_id: &str,
) -> Result<(WalkthroughRecord, bool), WalkthroughError> {
    let record = WalkthroughRecord::find(walkthrough_id)
        .ok_or_else(|| WalkthroughError::NotFound(walkthrough_id.to_string()))?;

    let mut state = if session_path.exists() {
        SessionState::load_from_file(session_path)
            .map_err(|e| WalkthroughError::Session(e.to_string()))?
    } else {
        SessionState::default()
    };

    let newly_completed = state.mark_walkthrough_completed(&record.id, &record.lesson_id);
    state
        .save_to_file(session_path)
        .map_err(|e| WalkthroughError::Session(e.to_string()))?;

    Ok((record, newly_completed))
}

/// Resets self-reported completion for a walkthrough in session.json.
pub fn reset_in_session(
    session_path: &Path,
    walkthrough_id: &str,
) -> Result<(WalkthroughRecord, bool), WalkthroughError> {
    let record = WalkthroughRecord::find(walkthrough_id)
        .ok_or_else(|| WalkthroughError::NotFound(walkthrough_id.to_string()))?;

    if !session_path.exists() {
        return Ok((record, false));
    }

    let mut state = SessionState::load_from_file(session_path)
        .map_err(|e| WalkthroughError::Session(e.to_string()))?;

    let removed = state.reset_walkthrough(&record.id);
    if removed {
        state
            .save_to_file(session_path)
            .map_err(|e| WalkthroughError::Session(e.to_string()))?;
    }

    Ok((record, removed))
}

/// Evaluates an ACP conversational prompt turn for guided walkthrough intents.
///
/// Returns `Some(response_markdown)` if the prompt requests walkthrough information,
/// guidance, completion, or listing; or `None` if the prompt should fall through to
/// exercise generation or drill attempt explanation.
pub fn handle_walkthrough_prompt(prompt_text: &str, session_path: Option<&Path>) -> Option<String> {
    let lower = prompt_text.to_lowercase();
    let is_walkthrough_intent = lower.contains("walkthrough")
        || lower.contains("guide me through")
        || lower.starts_with("guide ")
        || lower.starts_with("walk through");

    if !is_walkthrough_intent {
        return None;
    }

    let default_path = zedkata_core::session::default_session_path().ok();
    let resolved_session_path = session_path.or(default_path.as_deref());

    let session = resolved_session_path
        .and_then(|p| SessionState::load_from_file(p).ok())
        .unwrap_or_default();

    // 1. Observation and scoring query check (REQ-25 / REQ-29)
    if (lower.contains("observe")
        || lower.contains("scoring")
        || lower.contains("score")
        || lower.contains("keystroke")
        || lower.contains("did you see")
        || lower.contains("see my actions"))
        && !lower.contains("complete")
    {
        return Some(
            "### Walkthrough Observation & Scoring Policy (REQ-25 / REQ-29)\n\n\
            The Zedkata agent **does not claim or attempt to observe unseen editor actions** during guided walkthroughs.\n\n\
            - **Non-Observable Workflows**: Features such as LSP popups, the project panel, multibuffers, external tasks, DAP debugging, and agent panels execute in Zed's GPUI outside document buffer edit events (REQ-25).\n\
            - **Explicit Self-Reported Completion**: Walkthrough progress is verified strictly through your self-reported confirmation against the verified checklist (REQ-29).\n\
            - **Zero Drill Scoring**: Walkthroughs never enter drill scoring, generate attempts, or calculate par scores.\n\n\
            Commands:\n\
            - `guide me through <walkthrough-id>` — Show verified steps and self-report checklist\n\
            - `complete walkthrough <walkthrough-id>` — Record your self-reported completion\n\
            - `reset walkthrough <walkthrough-id>` — Restart walkthrough and clear completion status\n"
                .to_string(),
        );
    }

    // 2. Completion intent
    if lower.contains("complete") || lower.contains("finish") || lower.contains("done") {
        if let Some(record) = find_walkthrough_in_prompt(&lower) {
            if let Some(path) = resolved_session_path {
                match mark_completed_in_session(path, &record.id) {
                    Ok((rec, newly_completed)) => {
                        let status_note = if newly_completed {
                            "Recorded new self-reported completion."
                        } else {
                            "Refreshed existing completion record."
                        };
                        return Some(format!(
                            "### Walkthrough Completed (Self-Reported)\n\n\
                            **{}** (`{}`) marked as completed.\n\n\
                            - **Lesson**: `{}`\n\
                            - **Status**: [✓] COMPLETED\n\
                            - **Note**: {}\n\n\
                            *Guided walkthroughs use explicit self-reported verification without entering drill scoring (REQ-29).*",
                            rec.title, rec.id, rec.lesson_id, status_note
                        ));
                    }
                    Err(e) => {
                        return Some(format!(
                            "### Walkthrough Error\n\nFailed to record completion: {}\n",
                            e
                        ));
                    }
                }
            } else {
                return Some(format!(
                    "### Walkthrough Completed (Self-Reported)\n\n\
                    **{}** (`{}`) verified in current session.\n\n\
                    *(Session file not resolved on disk; state retained in memory.)*",
                    record.title, record.id
                ));
            }
        } else {
            return Some(
                "### Walkthrough Completion\n\n\
                Please specify which walkthrough you completed, for example:\n\
                - `complete walkthrough lsp-actions`\n\
                - `complete walkthrough mini-vs-any`\n\
                - `complete walkthrough project-panel`\n\n\
                Run `walkthrough list` to see all available walkthrough IDs.\n"
                    .to_string(),
            );
        }
    }

    // 3. Reset intent
    if lower.contains("reset") {
        if let Some(record) = find_walkthrough_in_prompt(&lower) {
            if let Some(path) = resolved_session_path {
                match reset_in_session(path, &record.id) {
                    Ok((rec, removed)) => {
                        let status_note = if removed {
                            "Completion mark cleared."
                        } else {
                            "Walkthrough was not previously marked complete."
                        };
                        return Some(format!(
                            "### Walkthrough Reset\n\n\
                            **{}** (`{}`) completion status reset.\n\n\
                            - **Status**: [ ] PENDING\n\
                            - **Note**: {}\n",
                            rec.title, rec.id, status_note
                        ));
                    }
                    Err(e) => {
                        return Some(format!(
                            "### Walkthrough Error\n\nFailed to reset walkthrough: {}\n",
                            e
                        ));
                    }
                }
            }
        } else {
            return Some(
                "### Walkthrough Reset\n\n\
                Please specify which walkthrough to reset, for example:\n\
                - `reset walkthrough lsp-actions`\n\
                - `reset walkthrough mini-vs-any`\n\n\
                Run `walkthrough list` to see all available walkthrough IDs.\n"
                    .to_string(),
            );
        }
    }

    // 3. Specific walkthrough detail / guidance
    if let Some(record) = find_walkthrough_in_prompt(&lower) {
        let is_completed = session.is_walkthrough_completed(&record.id);
        return Some(record.format_detail(is_completed));
    }

    // 4. Default: list all walkthroughs
    Some(format_walkthrough_list(&WalkthroughRecord::all(), &session))
}

fn find_walkthrough_in_prompt(lower: &str) -> Option<WalkthroughRecord> {
    for record in WalkthroughRecord::all() {
        if lower.contains(&record.id.to_lowercase()) {
            return Some(record);
        }
        let stripped_id = record.id.replace("-walkthrough", "").replace('-', " ");
        if lower.contains(&stripped_id) {
            return Some(record);
        }
    }

    // Match by keywords
    if lower.contains("mini") || lower.contains("anybracket") || lower.contains("minibracket") {
        return WalkthroughRecord::find("mini-vs-any");
    }
    if lower.contains("lsp") || lower.contains("symbol rename") || lower.contains("hover") {
        return WalkthroughRecord::find("lsp-actions-walkthrough");
    }
    if lower.contains("project panel") || lower.contains("netrw") {
        return WalkthroughRecord::find("project-panel-walkthrough");
    }
    if lower.contains("multibuffer") || lower.contains("cross-file") {
        return WalkthroughRecord::find("multibuffer-walkthrough");
    }
    if lower.contains("tasks") || lower.contains("debugger") || lower.contains("dap") {
        return WalkthroughRecord::find("tasks-and-debugger-walkthrough");
    }
    if lower.contains("agent panel") || lower.contains("acp") {
        return WalkthroughRecord::find("agent-panel-walkthrough");
    }
    if lower.contains("keymap") || lower.contains("binding") {
        return WalkthroughRecord::find("keymap-contexts");
    }
    if lower.contains("surround") || lower.contains("exchange") {
        return WalkthroughRecord::find("exchange-and-surround");
    }
    if lower.contains("single line") || lower.contains("find scope") {
        return WalkthroughRecord::find("single-line-find-scope");
    }
    if lower.contains("diagnostic") {
        return WalkthroughRecord::find("diagnostic-navigation");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn setup_temp_session(prefix: &str) -> (PathBuf, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "zedkata_{}_{}",
            prefix,
            uuid::Uuid::new_v4().simple()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let session_file = temp_dir.join("session.json");
        (temp_dir, session_file)
    }

    #[test]
    fn test_walkthrough_registry_contains_all_curriculum_walkthroughs() {
        let all = WalkthroughRecord::all();
        assert_eq!(all.len(), 10);

        let ids: Vec<_> = all.iter().map(|w| w.id.as_str()).collect();
        assert!(ids.contains(&"lsp-actions-walkthrough"));
        assert!(ids.contains(&"project-panel-walkthrough"));
        assert!(ids.contains(&"multibuffer-walkthrough"));
        assert!(ids.contains(&"tasks-and-debugger-walkthrough"));
        assert!(ids.contains(&"agent-panel-walkthrough"));
        assert!(ids.contains(&"mini-vs-any"));
        assert!(ids.contains(&"keymap-contexts"));
        assert!(ids.contains(&"exchange-and-surround"));
        assert!(ids.contains(&"single-line-find-scope"));
        assert!(ids.contains(&"diagnostic-navigation"));
    }

    #[test]
    fn test_walkthrough_detail_formatting_and_checklist() {
        let mini = WalkthroughRecord::find("mini-vs-any").expect("Must find mini-vs-any");
        assert_eq!(mini.lesson_id, "motions-03-textobjects");

        let pending_detail = mini.format_detail(false);
        assert!(pending_detail.contains("PENDING"));
        assert!(pending_detail.contains("MiniBrackets disagrees with AnyBrackets"));
        assert!(pending_detail.contains("nested.start.kata"));
        assert!(pending_detail.contains("1. Open the nested brackets fixture"));
        assert!(pending_detail.contains("[ ] Observed lexical delimiter selection"));
        assert!(pending_detail.contains("zedkata-agent walkthrough complete mini-vs-any"));

        let completed_detail = mini.format_detail(true);
        assert!(completed_detail.contains("COMPLETED (Self-Reported)"));
        assert!(completed_detail.contains("[x] Observed lexical delimiter selection"));
        assert!(completed_detail.contains("zedkata-agent walkthrough reset mini-vs-any"));
    }

    #[test]
    fn test_mark_and_reset_in_session_persists_without_drill_scoring() {
        let (temp_dir, session_file) = setup_temp_session("wt_mark_reset");

        // 1. Mark completed
        let (rec, newly_completed) =
            mark_completed_in_session(&session_file, "lsp-actions-walkthrough").unwrap();
        assert_eq!(rec.id, "lsp-actions-walkthrough");
        assert!(newly_completed);

        // Verify session state on disk
        let state = SessionState::load_from_file(&session_file).unwrap();
        assert!(state.is_walkthrough_completed("lsp-actions-walkthrough"));
        assert!(state.attempt.is_none());
        assert!(state.history.is_empty());
        assert!(state.weaknesses.is_empty());

        // 2. Reset completion
        let (_, removed) = reset_in_session(&session_file, "lsp-actions-walkthrough").unwrap();
        assert!(removed);

        let state_after_reset = SessionState::load_from_file(&session_file).unwrap();
        assert!(!state_after_reset.is_walkthrough_completed("lsp-actions-walkthrough"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_handle_walkthrough_prompt_routing() {
        let (temp_dir, session_file) = setup_temp_session("wt_prompt");

        // 1. List prompt
        let list_resp = handle_walkthrough_prompt("walkthrough list", Some(&session_file)).unwrap();
        assert!(list_resp.contains("Zedkata Guided Walkthroughs"));
        assert!(list_resp.contains("mini-vs-any"));
        assert!(list_resp.contains("lsp-actions-walkthrough"));

        // 2. Specific detail prompt
        let detail_resp =
            handle_walkthrough_prompt("guide me through mini-vs-any", Some(&session_file)).unwrap();
        assert!(detail_resp
            .contains("Guided Walkthrough: When MiniBrackets disagrees with AnyBrackets"));
        assert!(detail_resp.contains("PENDING"));

        // 3. Complete prompt
        let complete_resp =
            handle_walkthrough_prompt("complete walkthrough mini-vs-any", Some(&session_file))
                .unwrap();
        assert!(complete_resp.contains("Walkthrough Completed (Self-Reported)"));
        assert!(complete_resp.contains("mini-vs-any"));

        // 4. Detail prompt now shows completed
        let updated_detail =
            handle_walkthrough_prompt("show walkthrough mini-vs-any", Some(&session_file)).unwrap();
        assert!(updated_detail.contains("COMPLETED (Self-Reported)"));

        // 5. Unrelated prompt returns None
        assert!(handle_walkthrough_prompt("explain my attempt", Some(&session_file)).is_none());
        assert!(handle_walkthrough_prompt("generate a drill", Some(&session_file)).is_none());

        // 6. Observation and scoring policy queries (REQ-25 / REQ-29)
        let obs_resp = handle_walkthrough_prompt(
            "did you observe my walkthrough actions?",
            Some(&session_file),
        )
        .unwrap();
        assert!(obs_resp.contains("Walkthrough Observation & Scoring Policy"));
        assert!(obs_resp.contains("does not claim or attempt to observe unseen editor actions"));
        assert!(obs_resp.contains("Zero Drill Scoring"));

        let score_resp =
            handle_walkthrough_prompt("can you score my walkthrough?", Some(&session_file))
                .unwrap();
        assert!(score_resp.contains("does not claim or attempt to observe unseen editor actions"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
