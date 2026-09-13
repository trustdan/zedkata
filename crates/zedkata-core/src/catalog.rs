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
        list.sort_by(|a, b| a.lesson.id.cmp(&b.lesson.id));
        list
    }

    /// Return only runnable (non-draft) lessons.
    pub fn runnable_lessons(&self) -> Vec<&LessonEntry> {
        let mut list: Vec<&LessonEntry> =
            self.lessons.values().filter(|e| !e.lesson.draft).collect();
        list.sort_by(|a, b| a.lesson.id.cmp(&b.lesson.id));
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
        self.check_cycles(runnable)?;

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
}
