use crate::app::{App, ConnectionStatus, MIN_HEIGHT, MIN_WIDTH};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Renders the entire terminal UI shell frame with curriculum, task, hint, and progress panes.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.size();

    // Check if terminal is too small for standard layout
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_small_terminal_warning(frame, area);
        return;
    }

    // Standard shell layout: Header (3), Main Workspace (Min 9), Footer (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(9),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_footer(frame, chunks[2], area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title_line = Line::from(vec![
        Span::styled(
            " Zedkata ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("— Vim Drills for Zed"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title_line);

    let status_line = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                msg.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else if let Some(ref warn) = app.session_warning {
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                "State Warning",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(warn.clone(), Style::default().fg(Color::Yellow)),
        ])
    } else {
        match app.connection_status() {
            ConnectionStatus::Connected {
                ref lesson_id,
                exercise_index,
                ..
            } => Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "Connected to Zed",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | Active: "),
                Span::styled(
                    format!("{}:{}", lesson_id, exercise_index + 1),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            ConnectionStatus::WaitingForZed {
                ref lesson_id,
                exercise_index,
                ..
            } => Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "Waiting for Zed",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | Drill: "),
                Span::styled(
                    format!("{}:{}", lesson_id, exercise_index + 1),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    " (Press [o] to open in Zed)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            ConnectionStatus::MismatchedSession {
                ref selected_lesson,
                selected_exercise,
                ref active_lesson,
                active_exercise,
            } => Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "Viewing Inactive Drill",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" (Viewing: "),
                Span::styled(
                    format!("{}:{}", selected_lesson, selected_exercise + 1),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" | Active: "),
                Span::styled(
                    format!("{}:{}", active_lesson, active_exercise + 1),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    " | Press [s] to sync)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            ConnectionStatus::StaleBuffer { ref reason, .. } => Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "Stale Buffer",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled(reason.clone(), Style::default().fg(Color::LightRed)),
                Span::styled(
                    " (Press [o] to relaunch)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            ConnectionStatus::MissingFixture {
                ref fixture_path,
                is_start,
            } => Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "Missing Fixture",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::styled(
                    format!(
                        "{} fixture missing at {:?}",
                        if is_start { "Start" } else { "Goal" },
                        fixture_path.file_name().unwrap_or_default()
                    ),
                    Style::default().fg(Color::LightRed),
                ),
            ]),
            ConnectionStatus::Idle => Line::from(vec![
                Span::raw("Status: "),
                Span::styled("Idle", Style::default().fg(Color::DarkGray)),
                Span::raw(" | Select a drill and press [o] to launch in Zed"),
            ]),
        }
    };

    let header_paragraph = Paragraph::new(status_line)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(header_paragraph, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(area);

    render_curriculum_pane(frame, main_chunks[0], app);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_chunks[1]);

    render_task_pane(frame, right_chunks[0], app);
    render_feedback_pane(frame, right_chunks[1], app);
}

fn render_curriculum_pane(frame: &mut Frame, area: Rect, app: &App) {
    let lessons = app.lessons();
    let title = format!(" Curriculum ({}) ", lessons.len());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .title(title);

    if let Some(ref err) = app.catalog_error {
        let error_lines = vec![
            Line::from(Span::styled(
                "Catalog Error:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                err.clone(),
                Style::default().fg(Color::LightRed),
            )),
        ];
        let p = Paragraph::new(error_lines)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    }

    if lessons.is_empty() {
        let empty_lines = vec![
            Line::from(Span::styled(
                "No lessons found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Dir: {}", app.lessons_dir.display()),
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let p = Paragraph::new(empty_lines)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    }

    let mut lines = Vec::new();
    for (i, entry) in lessons.iter().enumerate() {
        let is_selected = i == app.selected_lesson_idx;
        let (is_locked, _) = app.is_lesson_locked(&entry.lesson);
        let is_completed = app.is_lesson_completed(&entry.lesson);
        let is_session_active = app.session.current.lesson_id == entry.lesson.id;

        let badge = if is_completed {
            Span::styled("[✓]", Style::default().fg(Color::Green))
        } else if is_locked {
            Span::styled("[L]", Style::default().fg(Color::Red))
        } else if entry.lesson.draft {
            Span::styled("[D]", Style::default().fg(Color::Magenta))
        } else {
            Span::styled("[ ]", Style::default().fg(Color::Cyan))
        };

        let pointer = if is_selected { "> " } else { "  " };

        let title_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_locked {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let mut spans = vec![
            Span::styled(
                pointer,
                if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                },
            ),
            badge,
            Span::raw(" "),
            Span::styled(entry.lesson.id.clone(), title_style),
        ];

        if is_session_active {
            spans.push(Span::styled(" *", Style::default().fg(Color::Yellow)));
        }

        lines.push(Line::from(spans));
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_task_pane(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .title(" Active Exercise Task ");

    let current_lesson = match app.current_lesson() {
        Some(l) => l,
        None => {
            let p = Paragraph::new(vec![Line::from(Span::styled(
                "No lesson selected.",
                Style::default().fg(Color::Yellow),
            ))])
            .block(block);
            frame.render_widget(p, area);
            return;
        }
    };

    let (is_locked, missing) = app.is_lesson_locked(&current_lesson.lesson);
    if is_locked {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "LOCKED LESSON: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    current_lesson.lesson.title.clone(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Required prerequisites incomplete:",
                Style::default().fg(Color::Yellow),
            )),
        ];

        for req in missing {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(
                    req,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Complete prerequisite exercises in Zed to unlock.",
            Style::default().fg(Color::DarkGray),
        )));

        let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    }

    let exercises = &current_lesson.lesson.exercises;
    if exercises.is_empty() {
        let lines = vec![
            Line::from(Span::styled(
                format!(
                    "{}: {}",
                    current_lesson.lesson.id, current_lesson.lesson.title
                ),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "No exercises in this lesson.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    }

    let current_ex = match app.current_exercise() {
        Some(ex) => ex,
        None => &exercises[0],
    };

    // Build exercise navigation pill bar
    let mut ex_pills = vec![Span::raw("Exercises: ")];
    for (idx, ex) in exercises.iter().enumerate() {
        let is_active_ex = idx == app.selected_exercise_idx;
        let is_session_current = app.session.current.lesson_id == current_lesson.lesson.id
            && app.session.current.exercise_index == idx;

        let style = if is_active_ex {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let marker = if is_session_current { "*" } else { "" };
        let tag = format!("[{}. {}{}] ", idx + 1, ex.id, marker);
        ex_pills.push(Span::styled(tag, style));
    }

    let mut lines = vec![
        Line::from(ex_pills),
        Line::from(vec![
            Span::styled(
                "Task: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(current_ex.task.clone(), Style::default().fg(Color::White)),
        ]),
    ];

    // Concepts
    if !current_lesson.lesson.concepts.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("Concepts: "),
            Span::styled(
                current_lesson.lesson.concepts.join(", "),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }

    // Budget & Rules
    let budget_str = if let Some(par) = current_ex.par_edits {
        let max_str = current_ex
            .max_edits
            .map(|m| format!(" | Max: {} edit(s)", m))
            .unwrap_or_default();
        format!("Par: {} edit(s){}", par, max_str)
    } else {
        let max_str = current_ex
            .max_edits
            .map(|m| format!(" | Max: {} edit(s)", m))
            .unwrap_or_default();
        format!("Goal-only (unscored){}", max_str)
    };
    let forbid_text = if let Some(ref forbid) = current_ex.forbid {
        if let Some(br) = forbid.backspace_run {
            format!(" | Forbid: backspaces >= {}", br)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    lines.push(Line::from(vec![
        Span::raw("Budget: "),
        Span::styled(
            format!("{}{}", budget_str, forbid_text),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    // Hint toggle
    let hint_line = if app.show_hint {
        Line::from(vec![
            Span::styled(
                "Hint: ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                current_ex
                    .hint
                    .clone()
                    .unwrap_or_else(|| "No hint provided.".to_string()),
                Style::default().fg(Color::LightMagenta),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Hint: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "[Hidden — press '?' to reveal]",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };
    lines.push(hint_line);

    // Practice instruction or fixture warning
    match app.current_exercise_fixture_status() {
        crate::app::FixtureStatus::Available { .. } => {
            lines.push(Line::from(vec![
                Span::raw("Practice in Zed: "),
                Span::styled(
                    "Press [o] to open in Zed editor",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        crate::app::FixtureStatus::MissingStart(ref p) => {
            lines.push(Line::from(vec![
                Span::styled(
                    "Fixture Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "Start fixture missing at {:?}",
                        p.file_name().unwrap_or_default()
                    ),
                    Style::default().fg(Color::LightRed),
                ),
            ]));
        }
        crate::app::FixtureStatus::MissingGoal(ref p) => {
            lines.push(Line::from(vec![
                Span::styled(
                    "Fixture Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "Goal fixture missing at {:?}",
                        p.file_name().unwrap_or_default()
                    ),
                    Style::default().fg(Color::LightRed),
                ),
            ]));
        }
        crate::app::FixtureStatus::ResolutionError(ref err) => {
            lines.push(Line::from(vec![
                Span::styled(
                    "Fixture Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(err.clone(), Style::default().fg(Color::LightRed)),
            ]));
        }
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_feedback_pane(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .title(" Live Attempt Feedback & Progress ");

    let conn_status = app.connection_status();
    let mut lines = Vec::new();

    // State Warning if any
    if let Some(ref warn) = app.session_warning {
        lines.push(Line::from(vec![
            Span::styled(
                "Sync Warning: ",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(warn.clone(), Style::default().fg(Color::Yellow)),
        ]));
    }

    // Corrupt Backup Recovery Notice
    if let Some(ref backup) = app.corrupt_backup_path {
        lines.push(Line::from(vec![
            Span::styled(
                "Recovery:     ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "Preserved corrupt session at {:?}",
                    backup.file_name().unwrap_or_default()
                ),
                Style::default().fg(Color::LightGreen),
            ),
        ]));
    }

    // Connection health and actionable diagnostic
    match &conn_status {
        ConnectionStatus::Connected { buffer_path, .. } => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    "CONNECTED TO ZED",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ("),
                Span::styled(
                    buffer_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("buffer"),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(")"),
            ]));
        }
        ConnectionStatus::WaitingForZed {
            ref expected_buffer,
            ..
        } => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    "WAITING FOR ZED",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " — Buffer not yet open in editor",
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            let action_hint = if let Some(ref p) = expected_buffer {
                format!("Press [o] to open in Zed, or run: zed {}", p.display())
            } else {
                "Press [o] to open in Zed editor".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    "Action:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action_hint, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Recovery:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "If Zed is open, run Command Palette -> 'workspace: restart language server'",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
        ConnectionStatus::MismatchedSession {
            ref selected_lesson,
            selected_exercise,
            ref active_lesson,
            active_exercise,
        } => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    "MISMATCHED VIEW",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — Viewing "),
                Span::styled(
                    format!("{}:{}", selected_lesson, selected_exercise + 1),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" while Zed has "),
                Span::styled(
                    format!("{}:{}", active_lesson, active_exercise + 1),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" active"),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Action:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Press [Enter] to switch drill to viewed, or [s] to sync view with Zed",
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        ConnectionStatus::StaleBuffer {
            ref buffer_path,
            ref reason,
        } => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    "STALE BUFFER",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — "),
                Span::styled(reason.clone(), Style::default().fg(Color::LightRed)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Action:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "Missing {:?}. Press [o] to relaunch fresh buffer in Zed",
                        buffer_path.file_name().unwrap_or_default()
                    ),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        ConnectionStatus::MissingFixture {
            ref fixture_path,
            is_start,
        } => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled(
                    "MISSING FIXTURE",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — "),
                Span::styled(
                    format!(
                        "{} fixture file does not exist at {:?}",
                        if *is_start { "Start" } else { "Goal" },
                        fixture_path
                    ),
                    Style::default().fg(Color::LightRed),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Action:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Restore fixture file to lesson directory before starting drill",
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        ConnectionStatus::Idle => {
            lines.push(Line::from(vec![
                Span::raw("Connection: "),
                Span::styled("IDLE", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " — No active exercise attached",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Action:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Select a drill and press [o] to launch in Zed",
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    // Active Attempt Verdict
    if let Some(ref att) = app.session.attempt {
        let verdict_span = match &att.verdict {
            Some(zedkata_core::session::Verdict::Passed { edits, par, score }) => {
                if let Some(par_val) = par {
                    Span::styled(
                        format!(
                            "PASSED! {} edit(s), par {}, score: {}",
                            edits, par_val, score
                        ),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!("PASSED! {} edit(s) (Goal-only / Unscored)", edits),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                }
            }
            Some(zedkata_core::session::Verdict::Failed { reason }) => Span::styled(
                format!("FAILED: {} (score: 0)", reason),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Some(zedkata_core::session::Verdict::InProgress) => Span::styled(
                format!("IN PROGRESS (Edits recorded: {})", att.edit_count),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            None => Span::styled(
                format!("Ready for drill (Edits: {})", att.edit_count),
                Style::default().fg(Color::Cyan),
            ),
        };

        lines.push(Line::from(vec![
            Span::raw("Attempt:    "),
            Span::styled(&att.id, Style::default().fg(Color::DarkGray)),
            Span::raw(" | "),
            verdict_span,
        ]));

        if matches!(
            att.verdict,
            Some(zedkata_core::session::Verdict::Passed { .. })
        ) {
            lines.push(Line::from(vec![
                Span::styled(
                    "Next Step:  ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "PASSED! Press [n] for next exercise, or [Enter] to activate",
                    Style::default().fg(Color::Green),
                ),
            ]));
        } else if matches!(
            att.verdict,
            Some(zedkata_core::session::Verdict::Failed { .. })
        ) {
            lines.push(Line::from(vec![
                Span::styled(
                    "Next Step:  ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "FAILED. Press [r] to retry active drill, or [o] to relaunch buffer in Zed",
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::raw("Attempt:    "),
            Span::styled(
                "Idle (Press [Enter] to activate drill)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    // Overall Progress & History: count unique passed exercises so repeated passes don't inflate progress
    let mut unique_passed = std::collections::HashSet::new();
    for h in &app.session.history {
        if h.passed {
            unique_passed.insert((&h.lesson_id, &h.exercise_id));
        }
    }
    let passed_count = unique_passed.len();
    let total_exercises: usize = app.lessons().iter().map(|l| l.lesson.exercises.len()).sum();

    let pct = (passed_count * 100)
        .checked_div(total_exercises)
        .unwrap_or(0);

    lines.push(Line::from(vec![
        Span::raw("Progress: "),
        Span::styled(
            format!(
                "{} of {} exercises completed ({}%)",
                passed_count, total_exercises, pct
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    if !app.session.history.is_empty() {
        let last_attempts: Vec<String> = app
            .session
            .history
            .iter()
            .rev()
            .take(3)
            .map(|h| {
                if !h.passed {
                    format!("{}/{} (failed)", h.lesson_id, h.exercise_id)
                } else if h.par_edits.is_some() {
                    format!("{}/{} (score {})", h.lesson_id, h.exercise_id, h.score)
                } else {
                    format!("{}/{} (unscored)", h.lesson_id, h.exercise_id)
                }
            })
            .collect();
        lines.push(Line::from(vec![
            Span::raw("Recent:   "),
            Span::styled(last_attempts.join(", "), Style::default().fg(Color::Gray)),
        ]));
    }

    // Concept Weakness Tracking
    if app.session.weaknesses.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("Weakness: "),
            Span::styled(
                "None recorded (clean)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        let mut sorted_weaknesses: Vec<(&String, &f64)> = app.session.weaknesses.iter().collect();
        sorted_weaknesses.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        let count = sorted_weaknesses.len();
        let items: Vec<Span> = sorted_weaknesses
            .into_iter()
            .take(4)
            .enumerate()
            .flat_map(|(idx, (concept, val))| {
                let pct = (*val * 100.0).round() as u32;
                let color = if *val >= 0.7 {
                    Color::LightRed
                } else if *val >= 0.3 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                let mut spans = vec![
                    Span::styled(concept.clone(), Style::default().fg(Color::White)),
                    Span::raw(": "),
                    Span::styled(
                        format!("{}%", pct),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                ];
                if idx + 1 < count.min(4) {
                    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
                }
                spans
            })
            .collect();

        let mut row = vec![Span::raw("Weakness: ")];
        row.extend(items);
        lines.push(Line::from(row));
    }

    lines.push(Line::from(vec![
        Span::styled("TUI Rule: ", Style::default().fg(Color::Green)),
        Span::styled(
            "Drill conductor only — all editing occurs in Zed",
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame, area: Rect, _full_area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let footer_line = Line::from(vec![
        Span::styled(
            " [j/k] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Lesson  "),
        Span::styled(
            " [h/l] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Ex  "),
        Span::styled(
            " [n] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Next  "),
        Span::styled(
            " [Enter] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Select  "),
        Span::styled(
            " [o] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Zed  "),
        Span::styled(
            " [?] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Hint  "),
        Span::styled(
            " [r] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Retry  "),
        Span::styled(
            " [s] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Sync  "),
        Span::styled(
            " [q] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit"),
    ]);

    let footer_p = Paragraph::new(footer_line).block(block);
    frame.render_widget(footer_p, area);
}

fn render_small_terminal_warning(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .title(" Terminal Too Small ");

    let warning_lines = vec![
        Line::from(Span::styled(
            "TERMINAL WINDOW TOO SMALL",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Current size:  "),
            Span::styled(
                format!("{} cols x {} rows", area.width, area.height),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("Required size: "),
            Span::styled(
                format!("{} cols x {} rows", MIN_WIDTH, MIN_HEIGHT),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from("Please enlarge your terminal window."),
        Line::from(Span::styled(
            "Press 'q' or 'Esc' to exit.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let p = Paragraph::new(warning_lines)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(p, area);
}
