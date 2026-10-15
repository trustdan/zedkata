use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::path::PathBuf;
use zedkata_core::session::SessionState;
use zedkata_tui::{
    app::{App, MIN_HEIGHT, MIN_WIDTH},
    event::handle_event,
    terminal::TerminalGuard,
    ui,
};

fn test_lessons_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_lessons = manifest_dir.join("..").join("..").join("lessons");
    if workspace_lessons.is_dir() {
        workspace_lessons
    } else {
        PathBuf::from("lessons")
    }
}

fn buffer_to_string(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            s.push_str(cell.symbol());
        }
        s.push('\n');
    }
    s
}

fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

#[test]
fn test_tui_render_catalog_and_task_panes() -> Result<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_tui_test_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&temp_dir)?;
    let session_file = temp_dir.join("session.json");
    let lessons_dir = test_lessons_dir();

    let backend = TestBackend::new(90, 30);
    let mut terminal = Terminal::new(backend)?;
    let app = App::load(session_file, lessons_dir);

    terminal.draw(|f| {
        ui::render(f, &app);
    })?;

    let rendered = buffer_to_string(&terminal);

    // Curriculum pane checks
    assert!(
        rendered.contains("Curriculum (7)"),
        "Should display lesson count in curriculum title"
    );
    assert!(
        rendered.contains("modal-00-baseline"),
        "Should list modal-00-baseline lesson"
    );
    assert!(
        rendered.contains("motions-01-word"),
        "Should list motions-01-word lesson"
    );
    assert!(
        rendered.contains("motions-02-find"),
        "Should list motions-02-find lesson"
    );
    assert!(
        rendered.contains("motions-03-textobjects"),
        "Should list motions-03-textobjects lesson"
    );
    assert!(
        rendered.contains("operators-01-edit"),
        "Should list operators-01-edit lesson"
    );
    assert!(
        rendered.contains("zed-01-lsp-actions"),
        "Should list zed-01-lsp-actions lesson"
    );
    assert!(
        rendered.contains("zed-02-project-panel"),
        "Should list zed-02-project-panel lesson"
    );
    assert!(rendered.contains("[L]"), "Should mark locked lessons");

    // Task pane checks for modal-00-baseline (initial selected lesson 0)
    assert!(
        rendered.contains("Make the 'timeout_ms' field public on line 2."),
        "Should display exercise 0 task description"
    );
    assert!(
        rendered.contains("Par: 1 edit(s) | Max: 2 edit(s)"),
        "Should display par and max budget"
    );
    assert!(
        rendered.contains("Forbid:"),
        "Should display forbid rule label"
    );
    assert!(
        rendered.contains("backspaces >= 3"),
        "Should display forbid rule value"
    );
    assert!(
        rendered.contains("Hint: [Hidden — press '?' to reveal]"),
        "Hint should initially be hidden"
    );

    // TUI rule check
    assert!(
        rendered.contains("all editing occurs in"),
        "Must clearly remind user that editing occurs in Zed"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_tui_hint_toggle() -> Result<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_tui_hint_test_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&temp_dir)?;
    let session_file = temp_dir.join("session.json");
    let lessons_dir = test_lessons_dir();

    let backend = TestBackend::new(90, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::load(session_file, lessons_dir);

    // Initially hidden
    assert!(!app.show_hint);
    terminal.draw(|f| ui::render(f, &app))?;
    let r1 = buffer_to_string(&terminal);
    assert!(r1.contains("[Hidden — press '?' to reveal]"));

    // Toggle on with '?'
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('?'), KeyModifiers::NONE),
    )?;
    assert!(app.show_hint);
    terminal.draw(|f| ui::render(f, &app))?;
    let r2 = buffer_to_string(&terminal);
    assert!(r2.contains("Enter insert mode before the cursor"));

    // Toggle off with '?'
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('?'), KeyModifiers::NONE),
    )?;
    assert!(!app.show_hint);
    terminal.draw(|f| ui::render(f, &app))?;
    let r3 = buffer_to_string(&terminal);
    assert!(r3.contains("[Hidden — press '?' to reveal]"));

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_tui_vim_navigation_and_locked_state() -> Result<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_tui_nav_test_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&temp_dir)?;
    let session_file = temp_dir.join("session.json");
    let lessons_dir = test_lessons_dir();

    let backend = TestBackend::new(90, 30);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::load(session_file.clone(), lessons_dir);

    assert_eq!(app.selected_lesson_idx, 0);
    assert_eq!(app.selected_exercise_idx, 0);

    // 'j' moves down to motions-01-word (idx 1)
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('j'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_lesson_idx, 1);

    // 'j' moves down to motions-02-find (idx 2)
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('j'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_lesson_idx, 2);

    // 'j' moves down to motions-03-textobjects (idx 3)
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('j'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_lesson_idx, 3);

    // motions-03-textobjects is locked because motions-01-word and motions-02-find are required
    terminal.draw(|f| ui::render(f, &app))?;
    let r_locked = buffer_to_string(&terminal);
    assert!(r_locked.contains("LOCKED LESSON: Text objects"));
    assert!(r_locked.contains("Required prerequisites incomplete:"));
    assert!(r_locked.contains("motions-01-word"));
    assert!(r_locked.contains("motions-02-find"));

    // Attempting to select locked lesson should be rejected
    handle_event(&mut app, make_key_event(KeyCode::Enter, KeyModifiers::NONE))?;
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .contains("Cannot select locked lesson"));

    // 'k' twice moves back up to motions-01-word (idx 1)
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('k'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_lesson_idx, 2);
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('k'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_lesson_idx, 1);

    // 'l' switches to next exercise (exercise 1: delete-to-word-end)
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('l'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_exercise_idx, 1);

    terminal.draw(|f| ui::render(f, &app))?;
    let r_ex1 = buffer_to_string(&terminal);
    assert!(r_ex1.contains("Delete the suffix 'Error' from the struct name"));

    // Press 'Enter' to activate exercise 1
    handle_event(&mut app, make_key_event(KeyCode::Enter, KeyModifiers::NONE))?;
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .contains("Activated exercise 'delete-to-word-end'"));

    // Verify session.json on disk reflects activated selection
    let disk_session = SessionState::load_from_file(&session_file)?;
    assert_eq!(disk_session.current.lesson_id, "motions-01-word");
    assert_eq!(disk_session.current.exercise_index, 1);

    // 'h' moves back to exercise 0
    handle_event(
        &mut app,
        make_key_event(KeyCode::Char('h'), KeyModifiers::NONE),
    )?;
    assert_eq!(app.selected_exercise_idx, 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_tui_render_small_terminal_warning() -> Result<()> {
    let backend = TestBackend::new(50, 10);
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(PathBuf::from("session.json"), test_lessons_dir());

    terminal.draw(|f| {
        ui::render(f, &app);
    })?;

    let rendered = buffer_to_string(&terminal);

    assert!(
        rendered.contains("TERMINAL WINDOW TOO SMALL"),
        "Rendered buffer must warn that terminal is too small"
    );
    assert!(
        rendered.contains("Current size:  50 cols x 10 rows"),
        "Rendered buffer should display current dimensions"
    );
    assert!(
        rendered.contains(&format!(
            "Required size: {} cols x {} rows",
            MIN_WIDTH, MIN_HEIGHT
        )),
        "Rendered buffer should display required minimum dimensions"
    );
    assert!(
        rendered.contains("Press 'q' or 'Esc' to exit"),
        "Small terminal warning must inform user how to quit"
    );

    Ok(())
}

#[test]
fn test_tui_render_exact_60x15_boundary() -> Result<()> {
    // 60 x 15 (exact minimum threshold) -> renders normal shell with all panes
    let backend_exact = TestBackend::new(60, 15);
    let mut terminal_exact = Terminal::new(backend_exact)?;
    let app = App::load(PathBuf::from("session.json"), test_lessons_dir());

    terminal_exact.draw(|f| ui::render(f, &app))?;
    let rendered = buffer_to_string(&terminal_exact);

    assert!(
        !rendered.contains("TERMINAL WINDOW TOO SMALL"),
        "Exact 60x15 should render full shell, not warning"
    );
    assert!(rendered.contains("Zedkata"));
    assert!(rendered.contains("Curriculum"));
    assert!(rendered.contains("Active Exercise Task"));
    assert!(rendered.contains("[j/k]"));

    Ok(())
}

#[test]
fn test_tui_empty_and_error_catalog() -> Result<()> {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    let app = App::load(
        PathBuf::from("session.json"),
        PathBuf::from("nonexistent_lessons_dir"),
    );

    terminal.draw(|f| ui::render(f, &app))?;
    let rendered = buffer_to_string(&terminal);

    assert!(
        rendered.contains("Catalog Error:"),
        "Should display catalog error when directory doesn't exist"
    );

    Ok(())
}

#[test]
fn test_terminal_guard_lifecycle() -> Result<()> {
    let mut guard = TerminalGuard::new();
    guard.restore()?;
    guard.restore()?;
    Ok(())
}
