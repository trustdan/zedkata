use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
use zedkata_core::{
    catalog::{Catalog, LessonEntry},
    lesson::{Exercise, Lesson},
    session::SessionState,
};

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

/// Connection and synchronization state between the TUI, Zed editor, and LSP observer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// No drill is currently active in session state.
    Idle,
    /// A drill is selected in session, but Zed has not yet opened the buffer or LSP has not attached.
    WaitingForZed {
        lesson_id: String,
        exercise_index: usize,
        expected_buffer: Option<PathBuf>,
    },
    /// Active drill buffer is open in Zed, attempt is attached and observed by LSP.
    Connected {
        lesson_id: String,
        exercise_index: usize,
        buffer_path: PathBuf,
        attempt_id: String,
    },
    /// TUI curriculum selection differs from the currently active exercise in Zed session.
    MismatchedSession {
        selected_lesson: String,
        selected_exercise: usize,
        active_lesson: String,
        active_exercise: usize,
    },
    /// Buffer path in session does not exist on disk.
    StaleBuffer {
        buffer_path: PathBuf,
        reason: String,
    },
    /// Start or goal fixture is missing on disk.
    MissingFixture {
        fixture_path: PathBuf,
        is_start: bool,
    },
}

/// Status of start and goal fixtures on disk for an exercise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureStatus {
    Available {
        start_path: PathBuf,
        goal_path: PathBuf,
    },
    MissingStart(PathBuf),
    MissingGoal(PathBuf),
    ResolutionError(String),
}

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
    pub last_session_poll: Instant,
    pub last_session_mtime: Option<SystemTime>,
    pub session_warning: Option<String>,
    pub corrupt_backup_path: Option<PathBuf>,
    pub poll_interval: Duration,
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
            last_session_poll: Instant::now(),
            last_session_mtime: None,
            session_warning: None,
            corrupt_backup_path: None,
            poll_interval: Duration::from_millis(100),
        }
    }

    /// Load application state: reads session from disk and loads the curriculum catalog.
    ///
    /// If session.json exists but is corrupt or has an incompatible version, it is safely
    /// preserved to `session.json.corrupt.<nanos>.bak` to avoid destroying recoverable student data.
    pub fn load(session_path: PathBuf, lessons_dir: PathBuf) -> Self {
        let (catalog, catalog_error) = match Catalog::load_from_dir(&lessons_dir) {
            Ok(cat) => (Some(cat), None),
            Err(err) => (None, Some(err.to_string())),
        };

        let (session, session_warning, initial_mtime, corrupt_backup_path) =
            if session_path.exists() {
                let mtime = std::fs::metadata(&session_path)
                    .and_then(|m| m.modified())
                    .ok();
                match SessionState::load_from_file(&session_path) {
                    Ok(s) => (s, None, mtime, None),
                    Err(err) => {
                        let backup_res = SessionState::backup_corrupt_file(&session_path);
                        let backup_path = backup_res.ok();
                        let warning_msg = if let Some(ref bp) = backup_path {
                            format!(
                                "Damaged state preserved at {:?}. Clean session initialized.",
                                bp.file_name().unwrap_or_default()
                            )
                        } else {
                            format!("Session load error: {}", err)
                        };
                        (
                            SessionState::default(),
                            Some(warning_msg),
                            None,
                            backup_path,
                        )
                    }
                }
            } else {
                (SessionState::default(), None, None, None)
            };

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
            last_session_poll: Instant::now(),
            last_session_mtime: initial_mtime,
            session_warning,
            corrupt_backup_path,
            poll_interval: Duration::from_millis(100),
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

    /// Check whether the start and goal fixtures for the currently selected exercise exist on disk.
    pub fn current_exercise_fixture_status(&self) -> FixtureStatus {
        let Some(lesson_entry) = self.current_lesson() else {
            return FixtureStatus::ResolutionError("No lesson selected".to_string());
        };
        let Some(exercise) = self.current_exercise() else {
            return FixtureStatus::ResolutionError("No exercise selected".to_string());
        };
        let Some(catalog) = &self.catalog else {
            return FixtureStatus::ResolutionError("Catalog not loaded".to_string());
        };

        let start_path = match catalog.resolve_fixture(&lesson_entry.lesson.id, &exercise.id, true)
        {
            Ok(p) => p,
            Err(e) => return FixtureStatus::ResolutionError(e.to_string()),
        };
        if !start_path.is_file() {
            return FixtureStatus::MissingStart(start_path);
        }

        let goal_path = match catalog.resolve_fixture(&lesson_entry.lesson.id, &exercise.id, false)
        {
            Ok(p) => p,
            Err(e) => return FixtureStatus::ResolutionError(e.to_string()),
        };
        if !goal_path.is_file() {
            return FixtureStatus::MissingGoal(goal_path);
        }

        FixtureStatus::Available {
            start_path,
            goal_path,
        }
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

    /// Move to the next exercise within the current lesson, or to the first exercise
    /// of the next lesson if at the end of the current lesson (and the next lesson is unlocked).
    pub fn next_exercise(&mut self) {
        if let Some(entry) = self.current_lesson() {
            let total_ex = entry.lesson.exercises.len();
            if total_ex > 0 && self.selected_exercise_idx + 1 < total_ex {
                self.selected_exercise_idx += 1;
                self.show_hint = false;
                return;
            }
        }

        // At end of current lesson; try moving to next lesson
        let total_lessons = self.lessons().len();
        if self.selected_lesson_idx + 1 < total_lessons {
            let next_idx = self.selected_lesson_idx + 1;
            let lessons = self.lessons();
            let next_lesson = &lessons[next_idx].lesson;
            let (is_locked, missing) = self.is_lesson_locked(next_lesson);
            if is_locked {
                self.status_message = Some(format!(
                    "Cannot navigate to locked lesson '{}' (requires {})",
                    next_lesson.id,
                    missing.join(", ")
                ));
            } else {
                self.selected_lesson_idx = next_idx;
                self.selected_exercise_idx = 0;
                self.show_hint = false;
            }
        }
    }

    /// Move to the previous exercise within the current lesson, or to the last exercise
    /// of the previous lesson if at the start of the current lesson.
    pub fn prev_exercise(&mut self) {
        if self.selected_exercise_idx > 0 {
            self.selected_exercise_idx -= 1;
            self.show_hint = false;
        } else if self.selected_lesson_idx > 0 {
            self.selected_lesson_idx -= 1;
            let prev_total = self
                .current_lesson()
                .map(|l| l.lesson.exercises.len())
                .unwrap_or(1);
            self.selected_exercise_idx = prev_total.saturating_sub(1);
            self.show_hint = false;
        }
    }

    /// Advances selection to the next exercise:
    /// - If there are more exercises in the current lesson, advances within the lesson.
    /// - If at the end of the current lesson, attempts to advance to the first exercise of the next lesson.
    /// - If the next lesson is locked, halts and reports the missing prerequisites.
    /// - If at the end of the entire curriculum, reports course completion.
    ///
    /// Returns `true` if selection was successfully advanced.
    pub fn advance_exercise(&mut self) -> bool {
        let Some(entry) = self.current_lesson() else {
            return false;
        };
        let total_ex = entry.lesson.exercises.len();
        if total_ex > 0 && self.selected_exercise_idx + 1 < total_ex {
            self.selected_exercise_idx += 1;
            self.show_hint = false;
            return true;
        }

        let total_lessons = self.lessons().len();
        if self.selected_lesson_idx + 1 < total_lessons {
            let next_idx = self.selected_lesson_idx + 1;
            let lessons = self.lessons();
            let next_lesson = &lessons[next_idx].lesson;
            let (locked, missing) = self.is_lesson_locked(next_lesson);
            if locked {
                self.status_message = Some(format!(
                    "Cannot advance: next lesson '{}' is locked (requires {})",
                    next_lesson.id,
                    missing.join(", ")
                ));
                return false;
            }
            self.selected_lesson_idx = next_idx;
            self.selected_exercise_idx = 0;
            self.show_hint = false;
            return true;
        }

        self.status_message = Some("Course completed! All exercises reached.".to_string());
        false
    }

    /// Advances selection to the next exercise and immediately activates it in session state,
    /// persisting the update to `session.json`.
    pub fn advance_and_activate(&mut self) -> Result<bool> {
        if self.advance_exercise() {
            self.activate_selected()?;
            if let Some(ex) = self.current_exercise() {
                let lesson_id = self
                    .current_lesson()
                    .map(|l| l.lesson.id.clone())
                    .unwrap_or_default();
                self.status_message = Some(format!(
                    "Advanced to exercise '{}' in lesson '{}'. Press [o] to launch in Zed",
                    ex.id, lesson_id
                ));
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Toggle visibility of the current exercise hint.
    pub fn toggle_hint(&mut self) {
        self.show_hint = !self.show_hint;
    }

    /// Check if a lesson is locked due to incomplete prerequisites.
    pub fn is_lesson_locked(&self, lesson: &Lesson) -> (bool, Vec<String>) {
        if let Some(ref cat) = self.catalog {
            let (unlocked, missing) = cat.is_lesson_unlocked(&lesson.id, &self.session.history);
            (!unlocked, missing)
        } else {
            (false, Vec::new())
        }
    }

    /// Check if all exercises in a lesson have been passed in session history.
    pub fn is_lesson_completed(&self, lesson: &Lesson) -> bool {
        if let Some(ref cat) = self.catalog {
            cat.is_lesson_completed(&lesson.id, &self.session.history)
        } else {
            false
        }
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
                // If destination session.json on disk contains corrupt/unparseable data, preserve it first
                if self.session_path.exists()
                    && SessionState::load_from_file(&self.session_path).is_err()
                {
                    let _ = SessionState::backup_corrupt_file(&self.session_path);
                }

                self.session
                    .switch_exercise(lesson_id.clone(), self.selected_exercise_idx);
                self.session
                    .save_to_file(&self.session_path)
                    .with_context(|| {
                        format!("Failed to save session to {:?}", self.session_path)
                    })?;

                if let Ok(meta) = std::fs::metadata(&self.session_path) {
                    self.last_session_mtime = meta.modified().ok();
                }

                self.status_message = Some(format!(
                    "Activated exercise '{}' in lesson '{}'",
                    ex_id, lesson_id
                ));
            }
        }
        Ok(())
    }

    /// Launch the currently selected exercise in Zed and set active attempt.
    pub fn launch_selected(&mut self) -> Result<()> {
        let (lesson_id, ex_idx, _ex_id) = if let Some(entry) = self.current_lesson() {
            let (is_locked, missing) = self.is_lesson_locked(&entry.lesson);
            if is_locked {
                self.status_message = Some(format!(
                    "Cannot launch locked lesson '{}' (requires {})",
                    entry.lesson.id,
                    missing.join(", ")
                ));
                return Ok(());
            }
            let ex_id = self.current_exercise().map(|ex| ex.id.clone());
            (entry.lesson.id.clone(), self.selected_exercise_idx, ex_id)
        } else {
            return Ok(());
        };

        // Check fixture status before launching
        match self.current_exercise_fixture_status() {
            FixtureStatus::MissingStart(ref p) => {
                self.status_message =
                    Some(format!("Cannot launch: start fixture missing at {:?}", p));
                return Ok(());
            }
            FixtureStatus::MissingGoal(ref p) => {
                self.status_message =
                    Some(format!("Cannot launch: goal fixture missing at {:?}", p));
                return Ok(());
            }
            FixtureStatus::ResolutionError(ref err) => {
                self.status_message = Some(format!("Fixture error: {}", err));
                return Ok(());
            }
            FixtureStatus::Available { .. } => {}
        }

        let Some(catalog) = &self.catalog else {
            self.status_message = Some("Catalog not loaded".to_string());
            return Ok(());
        };

        match crate::launcher::launch_exercise(
            &self.session_path,
            catalog,
            &lesson_id,
            ex_idx,
            None,
            None,
        ) {
            Ok(result) => {
                self.force_poll_session();
                self.status_message = Some(format!(
                    "Launched '{}' in Zed ({})",
                    result.exercise_id,
                    result.buffer_path.display()
                ));
            }
            Err(err) => {
                self.status_message = Some(format!("Launch failed: {}", err));
            }
        }
        Ok(())
    }

    /// Resets the active attempt for the current exercise:
    /// Restores the baseline start fixture into the writable buffer, issues a fresh attempt ID,
    /// clears feedback, and forces a state refresh.
    pub fn reset_active(&mut self) -> Result<()> {
        let Some(catalog) = &self.catalog else {
            self.status_message = Some("Catalog not loaded".to_string());
            return Ok(());
        };

        if self.session.current.lesson_id.is_empty() {
            self.status_message = Some("No active exercise to reset".to_string());
            return Ok(());
        }

        if let FixtureStatus::MissingStart(ref p) = self.current_exercise_fixture_status() {
            self.status_message = Some(format!("Cannot reset: start fixture missing at {:?}", p));
            return Ok(());
        }

        match crate::launcher::reset_exercise_attempt(&self.session_path, catalog, None) {
            Ok(result) => {
                self.force_poll_session();
                self.status_message = Some(format!(
                    "Reset attempt for '{}'. Baseline buffer restored.",
                    result.exercise_id
                ));
            }
            Err(err) => {
                self.status_message = Some(format!("Reset failed: {}", err));
            }
        }

        Ok(())
    }

    /// Polls `session.json` on disk if `poll_interval` has elapsed since the last check.
    ///
    /// Returns `true` if session state was updated with new content.
    /// Does not panic or crash on missing, corrupt, or locked files.
    pub fn poll_session(&mut self) -> bool {
        self.poll_session_internal(false)
    }

    /// Unconditionally forces a reload of `session.json` on disk, bypassing poll interval
    /// and modification-time checks.
    pub fn force_poll_session(&mut self) -> bool {
        self.poll_session_internal(true)
    }

    fn poll_session_internal(&mut self, force: bool) -> bool {
        let now = Instant::now();
        if !force && now.duration_since(self.last_session_poll) < self.poll_interval {
            return false;
        }
        self.last_session_poll = now;

        if !self.session_path.exists() {
            return false;
        }

        let meta = match std::fs::metadata(&self.session_path) {
            Ok(m) => m,
            Err(err) => {
                // Transient I/O error or lock during atomic replace
                self.session_warning = Some(format!("Failed reading session metadata: {}", err));
                return false;
            }
        };

        let current_mtime = meta.modified().ok();
        if !force && current_mtime.is_some() && current_mtime == self.last_session_mtime {
            return false;
        }

        match SessionState::load_from_file(&self.session_path) {
            Ok(loaded_session) => {
                self.session = loaded_session;
                self.last_session_mtime = current_mtime;
                self.session_warning = None;
                true
            }
            Err(err) => {
                // Retain previous valid session state; surface non-fatal warning
                self.last_session_mtime = current_mtime;
                self.session_warning = Some(format!("Session read error: {}", err));
                false
            }
        }
    }

    /// Refresh session and catalog from disk.
    pub fn refresh_state(&mut self) -> Result<()> {
        if let Ok(cat) = Catalog::load_from_dir(&self.lessons_dir) {
            self.catalog = Some(cat);
            self.catalog_error = None;
        }
        self.force_poll_session();
        self.status_message = Some("Refreshed catalog and session state".to_string());
        Ok(())
    }

    /// Evaluates the current connection and synchronization status between TUI selection,
    /// session state, and Zed buffer/LSP attachment.
    pub fn connection_status(&self) -> ConnectionStatus {
        let active_lesson = &self.session.current.lesson_id;
        if active_lesson.is_empty() {
            return ConnectionStatus::Idle;
        }

        let active_ex_idx = self.session.current.exercise_index;
        let selected_lesson = self
            .current_lesson()
            .map(|l| l.lesson.id.as_str())
            .unwrap_or("");
        let selected_ex_idx = self.selected_exercise_idx;

        // If TUI selection is on a different exercise than active session:
        if selected_lesson != active_lesson || selected_ex_idx != active_ex_idx {
            return ConnectionStatus::MismatchedSession {
                selected_lesson: selected_lesson.to_string(),
                selected_exercise: selected_ex_idx,
                active_lesson: active_lesson.clone(),
                active_exercise: active_ex_idx,
            };
        }

        // Check if start fixture for active exercise is missing
        if let Some(ref cat) = self.catalog {
            if let Some(ex) = self.current_exercise() {
                if let Ok(start_path) = cat.resolve_fixture(active_lesson, &ex.id, true) {
                    if !start_path.is_file() {
                        return ConnectionStatus::MissingFixture {
                            fixture_path: start_path,
                            is_start: true,
                        };
                    }
                }
            }
        }

        // Selected matches active session. Check buffer path and attempt.
        if let Some(ref bp) = self.session.current.buffer_path {
            let buf_path = PathBuf::from(bp);
            if !buf_path.exists() {
                return ConnectionStatus::StaleBuffer {
                    buffer_path: buf_path,
                    reason: "Exercise buffer file does not exist on disk".to_string(),
                };
            }

            if let Some(ref att) = self.session.attempt {
                ConnectionStatus::Connected {
                    lesson_id: active_lesson.clone(),
                    exercise_index: active_ex_idx,
                    buffer_path: buf_path,
                    attempt_id: att.id.clone(),
                }
            } else {
                ConnectionStatus::WaitingForZed {
                    lesson_id: active_lesson.clone(),
                    exercise_index: active_ex_idx,
                    expected_buffer: Some(buf_path),
                }
            }
        } else {
            // Buffer path not yet recorded in session by launcher/LSP
            let expected = self.catalog.as_ref().and_then(|c| {
                self.current_exercise()
                    .and_then(|ex| c.resolve_fixture(active_lesson, &ex.id, true).ok())
            });

            ConnectionStatus::WaitingForZed {
                lesson_id: active_lesson.clone(),
                exercise_index: active_ex_idx,
                expected_buffer: expected,
            }
        }
    }

    /// Synchronizes the TUI selection to match the active exercise in session state.
    pub fn sync_to_active(&mut self) {
        if self.session.current.lesson_id.is_empty() {
            self.status_message = Some("No active exercise in session to sync with".to_string());
            return;
        }

        let lessons = self.lessons();
        if let Some(idx) = lessons
            .iter()
            .position(|l| l.lesson.id == self.session.current.lesson_id)
        {
            self.selected_lesson_idx = idx;
            self.selected_exercise_idx = self.session.current.exercise_index;
            self.show_hint = false;
            self.status_message = Some(format!(
                "Synchronized view to active drill: {}:{}",
                self.session.current.lesson_id,
                self.session.current.exercise_index + 1
            ));
        } else {
            self.status_message = Some(format!(
                "Active lesson '{}' not found in loaded catalog",
                self.session.current.lesson_id
            ));
        }
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
