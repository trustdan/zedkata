use anyhow::{Context, Result};
use std::path::PathBuf;
use zedkata_core::{
    catalog::{Catalog, LessonEntry},
    lesson::{Exercise, Lesson},
    session::SessionState,
};

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

/// State for the interactive terminal application shell.
#[derive(Debug, Clone)]
pub struct App {
    pub should_quit: bool,
    pub session_path: PathBuf,
    pub lessons_dir: PathBuf,
    pub width: u16,
    pub height: u16,
    pub catalog: Option<Catalog>,
    pub catalog_error: Option<String>,
    pub session: SessionState,
    pub selected_lesson_idx: usize,
    pub selected_exercise_idx: usize,
    pub show_hint: bool,
    pub status_message: Option<String>,
}

impl App {
    /// Create a basic App struct (primarily used for unit testing).
    pub fn new(session_path: PathBuf, lessons_dir: PathBuf) -> Self {
        Self {
            should_quit: false,
            session_path,
            lessons_dir,
            width: 80,
            height: 24,
            catalog: None,
            catalog_error: None,
            session: SessionState::default(),
            selected_lesson_idx: 0,
            selected_exercise_idx: 0,
            show_hint: false,
            status_message: None,
        }
    }

    /// Load application state: reads session from disk and loads the curriculum catalog.
    pub fn load(session_path: PathBuf, lessons_dir: PathBuf) -> Self {
        let (catalog, catalog_error) = match Catalog::load_from_dir(&lessons_dir) {
            Ok(cat) => (Some(cat), None),
            Err(err) => (None, Some(err.to_string())),
        };

        let session = SessionState::load_or_init(&session_path).unwrap_or_default();

        let mut app = Self {
            should_quit: false,
            session_path,
            lessons_dir,
            width: 80,
            height: 24,
            catalog,
            catalog_error,
            session,
            selected_lesson_idx: 0,
            selected_exercise_idx: 0,
            show_hint: false,
            status_message: None,
        };

        // If session has an active lesson, select it in the catalog
        if !app.session.current.lesson_id.is_empty() {
            let lessons = app.lessons();
            if let Some(idx) = lessons
                .iter()
                .position(|l| l.lesson.id == app.session.current.lesson_id)
            {
                app.selected_lesson_idx = idx;
                app.selected_exercise_idx = app.session.current.exercise_index;
            }
        }

        app
    }

    /// List of all loaded lessons in deterministic order.
    pub fn lessons(&self) -> Vec<&LessonEntry> {
        self.catalog
            .as_ref()
            .map(|c| c.all_lessons())
            .unwrap_or_default()
    }

    /// The currently selected lesson entry, if any.
    pub fn current_lesson(&self) -> Option<&LessonEntry> {
        let list = self.lessons();
        list.get(self.selected_lesson_idx).copied()
    }

    /// The currently selected exercise within the selected lesson, if any.
    pub fn current_exercise(&self) -> Option<&Exercise> {
        let lesson_entry = self.current_lesson()?;
        lesson_entry
            .lesson
            .exercises
            .get(self.selected_exercise_idx)
    }

    /// Move to the next lesson in the curriculum list.
    pub fn next_lesson(&mut self) {
        let total = self.lessons().len();
        if total > 0 && self.selected_lesson_idx + 1 < total {
            self.selected_lesson_idx += 1;
            self.selected_exercise_idx = 0;
            self.show_hint = false;
        }
    }

    /// Move to the previous lesson in the curriculum list.
    pub fn prev_lesson(&mut self) {
        if self.selected_lesson_idx > 0 {
            self.selected_lesson_idx -= 1;
            self.selected_exercise_idx = 0;
            self.show_hint = false;
        }
    }

    /// Move to the next exercise within the current lesson.
    pub fn next_exercise(&mut self) {
        if let Some(entry) = self.current_lesson() {
            let total_ex = entry.lesson.exercises.len();
            if total_ex > 0 && self.selected_exercise_idx + 1 < total_ex {
                self.selected_exercise_idx += 1;
                self.show_hint = false;
            }
        }
    }

    /// Move to the previous exercise within the current lesson.
    pub fn prev_exercise(&mut self) {
        if self.selected_exercise_idx > 0 {
            self.selected_exercise_idx -= 1;
            self.show_hint = false;
        }
    }

    /// Toggle visibility of the current exercise hint.
    pub fn toggle_hint(&mut self) {
        self.show_hint = !self.show_hint;
    }

    /// Check if a lesson is locked due to incomplete prerequisites.
    pub fn is_lesson_locked(&self, lesson: &Lesson) -> (bool, Vec<String>) {
        let mut missing = Vec::new();
        for req in &lesson.requires {
            let passed = self
                .session
                .history
                .iter()
                .any(|h| h.lesson_id == *req && h.passed);
            if !passed {
                missing.push(req.clone());
            }
        }
        let is_locked = !missing.is_empty();
        (is_locked, missing)
    }

    /// Check if all exercises in a lesson have been passed in session history.
    pub fn is_lesson_completed(&self, lesson: &Lesson) -> bool {
        if lesson.exercises.is_empty() {
            return false;
        }
        lesson.exercises.iter().all(|ex| {
            self.session
                .history
                .iter()
                .any(|h| h.lesson_id == lesson.id && h.exercise_id == ex.id && h.passed)
        })
    }

    /// Activate the currently selected exercise and save state to session.json.
    pub fn activate_selected(&mut self) -> Result<()> {
        let lesson_data = if let Some(entry) = self.current_lesson() {
            let (is_locked, missing) = self.is_lesson_locked(&entry.lesson);
            let ex_id = self.current_exercise().map(|ex| ex.id.clone());
            Some((entry.lesson.id.clone(), is_locked, missing, ex_id))
        } else {
            None
        };

        if let Some((lesson_id, is_locked, missing, ex_id)) = lesson_data {
            if is_locked {
                self.status_message = Some(format!(
                    "Cannot select locked lesson '{}' (requires {})",
                    lesson_id,
                    missing.join(", ")
                ));
                return Ok(());
            }

            if let Some(ex_id) = ex_id {
                self.session
                    .switch_exercise(lesson_id.clone(), self.selected_exercise_idx);
                self.session
                    .save_to_file(&self.session_path)
                    .with_context(|| {
                        format!("Failed to save session to {:?}", self.session_path)
                    })?;

                self.status_message = Some(format!(
                    "Activated exercise '{}' in lesson '{}'",
                    ex_id, lesson_id
                ));
            }
        }
        Ok(())
    }

    /// Refresh session and catalog from disk.
    pub fn refresh_state(&mut self) -> Result<()> {
        if let Ok(cat) = Catalog::load_from_dir(&self.lessons_dir) {
            self.catalog = Some(cat);
            self.catalog_error = None;
        }
        if let Ok(s) = SessionState::load_from_file(&self.session_path) {
            self.session = s;
        }
        self.status_message = Some("Refreshed catalog and session state".to_string());
        Ok(())
    }

    /// Handles terminal window resizing.
    pub fn on_resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Returns true if current terminal dimensions are too small to render the shell cleanly.
    pub fn is_too_small(&self) -> bool {
        self.width < MIN_WIDTH || self.height < MIN_HEIGHT
    }

    /// Signals the main event loop to terminate.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initial_state_and_resizing() {
        let mut app = App::new(PathBuf::from("session.json"), PathBuf::from("lessons"));
        assert!(!app.should_quit);
        assert_eq!(app.width, 80);
        assert_eq!(app.height, 24);
        assert!(!app.is_too_small());

        app.on_resize(50, 20);
        assert!(app.is_too_small());

        app.on_resize(70, 10);
        assert!(app.is_too_small());

        app.on_resize(60, 15);
        assert!(!app.is_too_small());

        app.quit();
        assert!(app.should_quit);
    }
}
