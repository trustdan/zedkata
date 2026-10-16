use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{stdout, Stdout},
    path::{Path, PathBuf},
    sync::Once,
    time::Duration,
};

use crate::{app::App, event, ui};

static PANIC_HOOK_ONCE: Once = Once::new();

/// Restores the terminal back to cooked mode, normal screen buffer, and visible cursor.
pub fn restore_terminal() -> Result<()> {
    let _ = disable_raw_mode();
    let mut out = stdout();
    let _ = execute!(out, LeaveAlternateScreen, Show);
    Ok(())
}

/// Installs a panic hook to guarantee terminal restoration before printing panic information.
pub fn install_panic_hook() {
    PANIC_HOOK_ONCE.call_once(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = restore_terminal();
            default_hook(info);
        }));
    });
}

/// RAII guard ensuring the terminal is restored upon drop (e.g. on error return or unwind).
pub struct TerminalGuard {
    restored: bool,
}

impl TerminalGuard {
    pub fn new() -> Self {
        Self { restored: false }
    }

    pub fn restore(&mut self) -> Result<()> {
        if !self.restored {
            self.restored = true;
            restore_terminal()?;
        }
        Ok(())
    }
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Initializes the terminal: enables raw mode, enters alternate screen, hides cursor, and clears.
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

/// Runs the interactive TUI application event loop with optional generated lessons directory.
pub fn run_tui_app_with_generated(
    session_path: &Path,
    lessons_dir: &Path,
    generated_dir: Option<&Path>,
    test_error: bool,
    test_panic: bool,
) -> Result<()> {
    install_panic_hook();

    let mut terminal = init_terminal()?;
    let mut guard = TerminalGuard::new();

    // Trigger test verification branches if requested
    if test_panic {
        panic!("Triggered test panic for terminal restoration verification");
    }
    if test_error {
        anyhow::bail!("Triggered test error for terminal restoration verification");
    }

    let mut app = App::load_with_generated(
        session_path.to_path_buf(),
        lessons_dir.to_path_buf(),
        generated_dir.map(PathBuf::from),
    );

    // Initialize with current terminal size
    let size = terminal.size()?;
    app.on_resize(size.width, size.height);

    while !app.should_quit {
        app.poll_session();

        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        event::poll_and_handle_events(&mut app, Duration::from_millis(50))?;
    }

    guard.restore()?;
    Ok(())
}

/// Runs the interactive TUI application event loop.
pub fn run_tui_app(
    session_path: &Path,
    lessons_dir: &Path,
    test_error: bool,
    test_panic: bool,
) -> Result<()> {
    run_tui_app_with_generated(session_path, lessons_dir, None, test_error, test_panic)
}
