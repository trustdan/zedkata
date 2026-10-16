use crate::error::CoreError;
use crate::lesson::Lesson;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Clone, Copy)]
enum VisitState {
    Visiting,
    Visited,
}

/// Entry representing a loaded lesson and its origin on disk.
#[derive(Debug, Clone, PartialEq)]
pub struct LessonEntry {
    pub lesson: Lesson,
    pub path: PathBuf,
}

/// Catalog holding all loaded curriculum lessons and managing dependency resolution.
#[derive(Debug, Clone)]
pub struct Catalog {
    base_dir: PathBuf,
    lessons: HashMap<String, LessonEntry>,
}

impl Catalog {
    /// Load a catalog of lessons from a directory containing `.toml` files.
    ///
    /// Resolves `dir` to an absolute path so that subsequent fixture resolution is
    /// completely independent of the process's working directory (`std::env::current_dir`).
    pub fn load_from_dir(dir: &Path) -> Result<Self, CoreError> {
        let abs_dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(CoreError::IoError)?
                .join(dir)
        };

        let mut catalog = Self {
            base_dir: abs_dir.clone(),
            lessons: HashMap::new(),
        };

        let read_dir = std::fs::read_dir(&abs_dir).map_err(|err| CoreError::LessonIoError {
            path: abs_dir.clone(),
            source: err,
        })?;

        for entry in read_dir {
            let entry = entry.map_err(CoreError::IoError)?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                let lesson = Lesson::from_file(&path)?;

                if let Some(existing) = catalog.lessons.get(&lesson.id) {
                    return Err(CoreError::DuplicateLessonId {
                        id: lesson.id,
                        first_path: existing.path.clone(),
                        second_path: path,
                    });
                }

                catalog
                    .lessons
                    .insert(lesson.id.clone(), LessonEntry { lesson, path });
            }
        }

        Ok(catalog)
    }

    /// Load from an explicit list of lessons (useful for in-memory testing).
    pub fn from_lessons(base_dir: PathBuf, entries: Vec<LessonEntry>) -> Result<Self, CoreError> {
        let mut catalog = Self {
            base_dir,
            lessons: HashMap::new(),
        };

        for entry in entries {
            if let Some(existing) = catalog.lessons.get(&entry.lesson.id) {
                return Err(CoreError::DuplicateLessonId {
                    id: entry.lesson.id,
                    first_path: existing.path.clone(),
                    second_path: entry.path,
                });
            }

            catalog.lessons.insert(entry.lesson.id.clone(), entry);
        }

        Ok(catalog)
    }

    /// Retrieve a lesson entry by ID.
    pub fn get(&self, id: &str) -> Option<&LessonEntry> {
        self.lessons.get(id)
    }

    /// Return all loaded lessons, including drafts.
    pub fn all_lessons(&self) -> Vec<&LessonEntry> {
        let mut list: Vec<&LessonEntry> = self.lessons.values().collect();
        list.sort_by(|a, b| {
            let a_gen = a.lesson.is_generated();
            let b_gen = b.lesson.is_generated();
            a_gen
                .cmp(&b_gen)
                .then_with(|| a.lesson.id.cmp(&b.lesson.id))
        });
        list
    }

    /// Return only runnable (non-draft) lessons.
    pub fn runnable_lessons(&self) -> Vec<&LessonEntry> {
        let mut list: Vec<&LessonEntry> =
            self.lessons.values().filter(|e| !e.lesson.draft).collect();
        list.sort_by(|a, b| {
            let a_gen = a.lesson.is_generated();
            let b_gen = b.lesson.is_generated();
            a_gen
                .cmp(&b_gen)
                .then_with(|| a.lesson.id.cmp(&b.lesson.id))
        });
        list
    }

    /// Return the base directory of the catalog.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Resolve the full path to an exercise fixture file.
    ///
    /// Resolution is relative to the parent directory of the lesson file containing the exercise,
    /// making it completely independent of the process's working directory.
    pub fn resolve_fixture(
        &self,
        lesson_id: &str,
        exercise_id: &str,
        is_start: bool,
    ) -> Result<PathBuf, CoreError> {
        let entry = self.lessons.get(lesson_id).ok_or_else(|| {
            CoreError::VerificationError(format!("Lesson '{}' not found in catalog", lesson_id))
        })?;

        let exercise = entry
            .lesson
            .exercises
            .iter()
            .find(|ex| ex.id == exercise_id)
            .ok_or_else(|| {
                CoreError::VerificationError(format!(
                    "Exercise '{}' not found in lesson '{}'",
                    exercise_id, lesson_id
                ))
            })?;

        let rel_fixture = if is_start {
            &exercise.start
        } else {
            &exercise.goal
        };

        let lesson_dir = entry.path.parent().unwrap_or(&self.base_dir);
        let resolved = lesson_dir.join(rel_fixture);
        Ok(resolved)
    }

    /// Validate all lessons for prerequisite existence and acyclicity.
    pub fn validate_catalog(&self) -> Result<(), CoreError> {
        self.check_prerequisites(self.lessons.values())?;
        self.check_cycles(self.lessons.values())?;
        Ok(())
    }

    /// Validate runnable lessons: ensures all prerequisites exist, there are no cycles,
    /// and all fixture files exist on disk.
    pub fn validate_runnable(&self) -> Result<(), CoreError> {
        let runnable = self.runnable_lessons();

        // Check prerequisites for runnable lessons
        for entry in &runnable {
            for req in &entry.lesson.requires {
                let req_entry =
                    self.lessons
                        .get(req)
                        .ok_or_else(|| CoreError::MissingPrerequisite {
                            lesson_id: entry.lesson.id.clone(),
                            prerequisite: req.clone(),
                        })?;

                if req_entry.lesson.draft {
                    return Err(CoreError::LessonValidationError {
                        path: Some(entry.path.clone()),
                        field: "lesson.requires".to_string(),
                        message: format!(
                            "Runnable lesson '{}' cannot depend on draft lesson '{}'",
                            entry.lesson.id, req
                        ),
                    });
                }
            }
        }

        // Check cycles among runnable lessons
        self.check_cycles(runnable.iter().copied())?;

        for entry in &runnable {
            if entry.lesson.exercises.is_empty() {
                return Err(CoreError::LessonValidationError {
                    path: Some(entry.path.clone()),
                    field: "lesson.exercise".to_string(),
                    message: format!(
                        "Runnable lesson '{}' must contain at least one drillable exercise",
                        entry.lesson.id
                    ),
                });
            }
        }

        // Check fixture existence on disk
        for entry in self.runnable_lessons() {
            for ex in &entry.lesson.exercises {
                let start_path = self.resolve_fixture(&entry.lesson.id, &ex.id, true)?;
                if !start_path.is_file() {
                    return Err(CoreError::MissingFixture {
                        lesson_id: entry.lesson.id.clone(),
                        exercise_id: ex.id.clone(),
                        path: start_path,
                    });
                }

                let goal_path = self.resolve_fixture(&entry.lesson.id, &ex.id, false)?;
                if !goal_path.is_file() {
                    return Err(CoreError::MissingFixture {
                        lesson_id: entry.lesson.id.clone(),
                        exercise_id: ex.id.clone(),
                        path: goal_path,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate the complete shipped catalog:
    /// 1. Validates all lessons (prerequisites exist, entire DAG is acyclic).
    /// 2. Validates that every lesson has at least one exercise or walkthrough.
    /// 3. Validates runnable lessons (draft exclusion: no runnable lesson depends on a draft).
    /// 4. Validates that all runnable exercises have at least one drillable exercise and fixtures exist on disk.
    pub fn validate_shipped_catalog(&self) -> Result<(), CoreError> {
        self.validate_catalog()?;
        for entry in self.lessons.values() {
            if entry.lesson.exercises.is_empty() && entry.lesson.walkthroughs.is_empty() {
                return Err(CoreError::LessonValidationError {
                    path: Some(entry.path.clone()),
                    field: "lesson".to_string(),
                    message: format!(
                        "Lesson '{}' must contain at least one exercise or walkthrough",
                        entry.lesson.id
                    ),
                });
            }
        }
        self.validate_runnable()?;
        Ok(())
    }

    /// Check if all exercises in a lesson have been passed in session history.
    pub fn is_lesson_completed(
        &self,
        lesson_id: &str,
        history: &[crate::session::HistoryEntry],
    ) -> bool {
        let Some(entry) = self.lessons.get(lesson_id) else {
            return false;
        };
        if entry.lesson.exercises.is_empty() {
            return false;
        }
        entry.lesson.exercises.iter().all(|ex| {
            history
                .iter()
                .any(|h| h.lesson_id == lesson_id && h.exercise_id == ex.id && h.passed)
        })
    }

    /// Check if a lesson's prerequisites are satisfied by session history.
    ///
    /// Returns `(is_unlocked, missing_prerequisites)`:
    /// - If a required lesson is in the catalog, all of its exercises must be completed (`is_lesson_completed`).
    /// - If a required lesson is not in the catalog (e.g. external lesson), at least one passed attempt
    ///   for that lesson ID must exist in `history`.
    /// - Failed attempts (`passed: false`) never satisfy a prerequisite.
    pub fn is_lesson_unlocked(
        &self,
        lesson_id: &str,
        history: &[crate::session::HistoryEntry],
    ) -> (bool, Vec<String>) {
        let Some(entry) = self.lessons.get(lesson_id) else {
            return (true, Vec::new());
        };

        let mut missing = Vec::new();
        for req in &entry.lesson.requires {
            let satisfied = if self.lessons.contains_key(req) {
                self.is_lesson_completed(req, history)
            } else {
                history.iter().any(|h| h.lesson_id == *req && h.passed)
            };

            if !satisfied {
                missing.push(req.clone());
            }
        }

        let is_unlocked = missing.is_empty();
        (is_unlocked, missing)
    }

    fn check_prerequisites<'a, I>(&self, entries: I) -> Result<(), CoreError>
    where
        I: IntoIterator<Item = &'a LessonEntry>,
    {
        for entry in entries {
            for req in &entry.lesson.requires {
                if !self.lessons.contains_key(req) {
                    return Err(CoreError::MissingPrerequisite {
                        lesson_id: entry.lesson.id.clone(),
                        prerequisite: req.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn check_cycles<'a, I>(&self, entries: I) -> Result<(), CoreError>
    where
        I: IntoIterator<Item = &'a LessonEntry>,
    {
        let mut states: HashMap<&str, VisitState> = HashMap::new();
        let target_entries: Vec<&LessonEntry> = entries.into_iter().collect();

        for entry in &target_entries {
            let id = entry.lesson.id.as_str();
            if !states.contains_key(id) {
                let mut path = Vec::new();
                self.dfs_cycle(id, &mut states, &mut path)?;
            }
        }

        Ok(())
    }

    fn dfs_cycle<'a>(
        &'a self,
        current: &'a str,
        states: &mut HashMap<&'a str, VisitState>,
        path: &mut Vec<&'a str>,
    ) -> Result<(), CoreError> {
        states.insert(current, VisitState::Visiting);
        path.push(current);

        if let Some(entry) = self.lessons.get(current) {
            for req in &entry.lesson.requires {
                let req_str = req.as_str();
                match states.get(req_str) {
                    Some(VisitState::Visiting) => {
                        // Cycle detected
                        let mut cycle = Vec::new();
                        let mut recording = false;
                        for node in path.iter() {
                            if *node == req_str {
                                recording = true;
                            }
                            if recording {
                                cycle.push(node.to_string());
                            }
                        }
                        cycle.push(req_str.to_string());

                        return Err(CoreError::PrerequisiteCycle { cycle });
                    }
                    Some(VisitState::Visited) => {}
                    None => {
                        self.dfs_cycle(req_str, states, path)?;
                    }
                }
            }
        }

        path.pop();
        states.insert(current, VisitState::Visited);
        Ok(())
    }

    /// Insert a lesson entry into the catalog, preserving any existing lesson with the same ID.
    ///
    /// Returns `true` if inserted, or `false` if skipped because an existing lesson was preserved.
    pub fn insert_preserving_existing(&mut self, entry: LessonEntry) -> bool {
        if self.lessons.contains_key(&entry.lesson.id) {
            false
        } else {
            self.lessons.insert(entry.lesson.id.clone(), entry);
            true
        }
    }

    /// Merges lessons from a directory (e.g. generated lessons directory) into this catalog.
    ///
    /// In accordance with REQ-28, ADR 0005, and ZK-032.2:
    /// - If `dir` does not exist on disk, returns `Ok(MergeSummary::default())`.
    /// - ID collisions preserve existing lessons: if a lesson ID already exists in the catalog,
    ///   the existing lesson remains intact and the duplicate is recorded in `skipped_collisions`.
    /// - Malformed or unparseable files in `dir` are recorded in `skipped_invalid` and do not fail the catalog.
    pub fn merge_from_dir(&mut self, dir: &Path) -> Result<MergeSummary, CoreError> {
        let abs_dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(CoreError::IoError)?
                .join(dir)
        };

        let mut summary = MergeSummary::default();

        if !abs_dir.exists() || !abs_dir.is_dir() {
            return Ok(summary);
        }

        let read_dir = match std::fs::read_dir(&abs_dir) {
            Ok(rd) => rd,
            Err(e) => {
                summary.skipped_invalid.push((abs_dir, e.to_string()));
                return Ok(summary);
            }
        };

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    summary
                        .skipped_invalid
                        .push((abs_dir.clone(), e.to_string()));
                    continue;
                }
            };
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                match Lesson::from_file(&path) {
                    Ok(lesson) => {
                        let id = lesson.id.clone();
                        let entry = LessonEntry { lesson, path };
                        if self.insert_preserving_existing(entry) {
                            summary.added.push(id);
                        } else {
                            summary.skipped_collisions.push(id);
                        }
                    }
                    Err(e) => {
                        summary.skipped_invalid.push((path, e.to_string()));
                    }
                }
            }
        }

        Ok(summary)
    }

    /// Loads the base catalog and merges generated lessons from `generated_dir` if present.
    ///
    /// If `generated_dir` is `Some(p)`, merges generated lessons from `p`.
    /// If `generated_dir` is `None`:
    /// 1. Attempts to load from `default_generated_dir()` if it exists.
    /// 2. Also checks `shipped_dir.join("generated")` if present and distinct from the above.
    pub fn load_with_generated(
        shipped_dir: &Path,
        generated_dir: Option<&Path>,
    ) -> Result<(Self, Option<MergeSummary>), CoreError> {
        let mut catalog = Self::load_from_dir(shipped_dir)?;

        let mut combined_summary = MergeSummary::default();
        let mut merged_any = false;

        if let Some(gp) = generated_dir {
            if gp.exists() && gp.is_dir() {
                let summary = catalog.merge_from_dir(gp)?;
                combined_summary.added.extend(summary.added);
                combined_summary
                    .skipped_collisions
                    .extend(summary.skipped_collisions);
                combined_summary
                    .skipped_invalid
                    .extend(summary.skipped_invalid);
                merged_any = true;
            }
        }

        let local_gen = shipped_dir.join("generated");
        if local_gen.exists() && local_gen.is_dir() {
            let already_checked = generated_dir
                .and_then(|g| g.canonicalize().ok())
                .and_then(|gc| local_gen.canonicalize().ok().map(|lc| lc == gc))
                .unwrap_or(false);

            if !already_checked {
                let summary = catalog.merge_from_dir(&local_gen)?;
                combined_summary.added.extend(summary.added);
                combined_summary
                    .skipped_collisions
                    .extend(summary.skipped_collisions);
                combined_summary
                    .skipped_invalid
                    .extend(summary.skipped_invalid);
                merged_any = true;
            }
        }

        if merged_any {
            Ok((catalog, Some(combined_summary)))
        } else {
            Ok((catalog, None))
        }
    }
}

/// Summary of merging additional lessons into an existing Catalog.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeSummary {
    /// Lesson IDs successfully added to the catalog.
    pub added: Vec<String>,
    /// Lesson IDs skipped because an existing lesson in the catalog was preserved.
    pub skipped_collisions: Vec<String>,
    /// Files that were skipped due to I/O or parsing errors.
    pub skipped_invalid: Vec<(PathBuf, String)>,
}

/// Resolves standard platform path for user generated lessons directory.
///
/// Precedence (REQ-12, REQ-28, REQ-31, ADR 0005):
/// 1. `ZEDKATA_GENERATED_DIR` environment variable if set and non-empty.
/// 2. `ZEDKATA_DATA_DIR` environment variable if set and non-empty: `<ZEDKATA_DATA_DIR>/lessons/generated`.
/// 3. Windows: `%LOCALAPPDATA%\zedkata\lessons\generated\` (fallback `%USERPROFILE%\AppData\Local\zedkata\lessons\generated\`).
/// 4. Linux: `$XDG_DATA_HOME/zedkata/lessons/generated/` (fallback `~/.local/share/zedkata/lessons/generated/`).
/// 5. macOS: `~/Library/Application Support/zedkata/lessons/generated/`.
pub fn default_generated_dir() -> Result<PathBuf, CoreError> {
    if let Ok(override_path) = std::env::var("ZEDKATA_GENERATED_DIR") {
        if !override_path.trim().is_empty() {
            return Ok(PathBuf::from(override_path));
        }
    }

    if let Ok(data_dir) = std::env::var("ZEDKATA_DATA_DIR") {
        if !data_dir.trim().is_empty() {
            return Ok(PathBuf::from(data_dir).join("lessons").join("generated"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data)
                .join("zedkata")
                .join("lessons")
                .join("generated"));
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("zedkata")
                .join("lessons")
                .join("generated"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("zedkata")
                .join("lessons")
                .join("generated"));
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data)
                .join("zedkata")
                .join("lessons")
                .join("generated"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("zedkata")
                .join("lessons")
                .join("generated"));
        }
    }

    Err(CoreError::DataPathError(
        "Could not resolve platform standard generated lessons directory for zedkata".to_string(),
    ))
}

/// Resolves standard platform path for user data directory root (`.../zedkata`).
pub fn default_data_dir() -> Result<PathBuf, CoreError> {
    if let Ok(override_path) = std::env::var("ZEDKATA_DATA_DIR") {
        if !override_path.trim().is_empty() {
            return Ok(PathBuf::from(override_path));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data).join("zedkata"));
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("zedkata"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("zedkata"));
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join("zedkata"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("zedkata"));
        }
    }

    Err(CoreError::DataPathError(
        "Could not resolve platform standard data directory for zedkata".to_string(),
    ))
}

/// Resolves the curriculum lessons directory based on:
/// 1. Explicit override path (if provided)
/// 2. `ZEDKATA_LESSONS_DIR` environment variable
/// 3. Executable-relative lookup (e.g. `<exe_dir>/../lessons`, `<exe_dir>/lessons`, `<exe_dir>/../share/zedkata/lessons`)
/// 4. Current working directory / workspace fallback (`lessons`, `../../lessons`)
/// 5. Platform standard data directory (`default_data_dir().join("lessons")`)
pub fn resolve_lessons_dir(cli_path: Option<&Path>) -> Result<PathBuf, CoreError> {
    if let Some(path) = cli_path {
        return Ok(path.to_path_buf());
    }

    if let Ok(env_val) = std::env::var("ZEDKATA_LESSONS_DIR") {
        let trimmed = env_val.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    // Check executable-relative paths if available
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Layout 1: <root>/bin/zedkata -> <root>/lessons
            let candidate1 = exe_dir.join("..").join("lessons");
            if candidate1.is_dir() {
                return Ok(candidate1);
            }
            // Layout 2: <root>/zedkata -> <root>/lessons
            let candidate2 = exe_dir.join("lessons");
            if candidate2.is_dir() {
                return Ok(candidate2);
            }
            // Layout 3: <prefix>/bin/zedkata -> <prefix>/share/zedkata/lessons (Unix FHS)
            let candidate3 = exe_dir
                .join("..")
                .join("share")
                .join("zedkata")
                .join("lessons");
            if candidate3.is_dir() {
                return Ok(candidate3);
            }
        }
    }

    // Check current working directory or relative workspace paths
    if Path::new("lessons").is_dir() {
        return Ok(PathBuf::from("lessons"));
    }
    if Path::new("../../lessons").is_dir() {
        return Ok(PathBuf::from("../../lessons"));
    }

    // Check platform standard data directory
    if let Ok(data_dir) = default_data_dir() {
        let platform_lessons = data_dir.join("lessons");
        if platform_lessons.is_dir() {
            return Ok(platform_lessons);
        }
    }

    // Default fallback to "lessons" relative to cwd
    Ok(PathBuf::from("lessons"))
}

/// Returns the default lessons directory by invoking `resolve_lessons_dir(None)`.
pub fn default_lessons_dir() -> Result<PathBuf, CoreError> {
    resolve_lessons_dir(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_lessons_dir_explicit_override() {
        let explicit = Path::new("/custom/lessons/path");
        let resolved = resolve_lessons_dir(Some(explicit)).unwrap();
        assert_eq!(resolved, PathBuf::from("/custom/lessons/path"));
    }

    #[test]
    fn test_resolve_lessons_dir_env_var() {
        std::env::set_var("ZEDKATA_LESSONS_DIR", "/custom/env/lessons");
        let resolved = resolve_lessons_dir(None).unwrap();
        assert_eq!(resolved, PathBuf::from("/custom/env/lessons"));
        std::env::remove_var("ZEDKATA_LESSONS_DIR");
    }

    #[test]
    fn test_default_lessons_dir_fallback() {
        std::env::remove_var("ZEDKATA_LESSONS_DIR");
        let dir = default_lessons_dir().unwrap();
        assert!(!dir.as_os_str().is_empty());
    }
}
