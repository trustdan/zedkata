use crate::app::App;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

/// Handles a single crossterm input/terminal event.
pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
    match event {
        Event::Key(key) => {
            // Ctrl+C abort
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                app.quit();
                return Ok(());
            }

            // Standard navigation & control keys
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    app.quit();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    app.next_lesson();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.prev_lesson();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    app.next_exercise();
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    app.prev_exercise();
                }
                KeyCode::Enter => {
                    let _ = app.activate_selected();
                }
                KeyCode::Char('?') | KeyCode::Char('H') => {
                    app.toggle_hint();
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    let _ = app.refresh_state();
                }
                _ => {}
            }
        }
        Event::Resize(width, height) => {
            app.on_resize(width, height);
        }
        _ => {}
    }
    Ok(())
}

/// Polls for incoming events up to the specified timeout duration and processes any received event.
pub fn poll_and_handle_events(app: &mut App, timeout: Duration) -> Result<()> {
    if event::poll(timeout)? {
        let ev = event::read()?;
        handle_event(app, ev)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};
    use std::path::PathBuf;

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn test_handle_quit_keys() -> Result<()> {
        let mut app = App::new(PathBuf::from("session.json"), PathBuf::from("lessons"));

        // 'q' quits
        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('q'), KeyModifiers::NONE),
        )?;
        assert!(app.should_quit);

        app.should_quit = false;
        // 'Q' quits
        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('Q'), KeyModifiers::NONE),
        )?;
        assert!(app.should_quit);

        app.should_quit = false;
        // 'Esc' quits
        handle_event(&mut app, make_key_event(KeyCode::Esc, KeyModifiers::NONE))?;
        assert!(app.should_quit);

        app.should_quit = false;
        // Ctrl+C quits
        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
        )?;
        assert!(app.should_quit);

        app.should_quit = false;
        // Unrelated key does not quit
        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('z'), KeyModifiers::NONE),
        )?;
        assert!(!app.should_quit);

        Ok(())
    }

    #[test]
    fn test_handle_resize_event() -> Result<()> {
        let mut app = App::new(PathBuf::from("session.json"), PathBuf::from("lessons"));
        assert_eq!(app.width, 80);
        assert_eq!(app.height, 24);

        handle_event(&mut app, Event::Resize(120, 40))?;
        assert_eq!(app.width, 120);
        assert_eq!(app.height, 40);

        Ok(())
    }

    #[test]
    fn test_handle_hint_toggle_key() -> Result<()> {
        let mut app = App::new(PathBuf::from("session.json"), PathBuf::from("lessons"));
        assert!(!app.show_hint);

        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('?'), KeyModifiers::NONE),
        )?;
        assert!(app.show_hint);

        handle_event(
            &mut app,
            make_key_event(KeyCode::Char('?'), KeyModifiers::NONE),
        )?;
        assert!(!app.show_hint);

        Ok(())
    }
}
