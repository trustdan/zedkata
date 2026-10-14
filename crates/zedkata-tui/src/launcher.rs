use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use thiserror::Error;
use zedkata_core::{catalog::Catalog, error::CoreError, session::SessionState};

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("Lesson '{0}' is locked (incomplete prerequisites: {1})")]
    LessonLocked(String, String),

    #[error("Lesson '{0}' not found in catalog")]
    LessonNotFound(String),

    #[error(
        "Exercise index {index} out of bounds for lesson '{lesson_id}' (total exercises: {total})"
    )]
    ExerciseIndexOutOfBounds {
        lesson_id: String,
        index: usize,
        total: usize,
    },

    #[error("Start fixture for exercise '{exercise_id}' in lesson '{lesson_id}' could not be resolved: {source}")]
    FixtureResolutionError {
        lesson_id: String,
        exercise_id: String,
        source: Box<CoreError>,
    },

    #[error("Start fixture does not exist at {0:?}")]
    FixtureNotFound(PathBuf),

    #[error("Failed to copy exercise fixture from {src:?} to {dest:?}: {source}")]
    CopyError {
        src: PathBuf,
        dest: PathBuf,
        source: std::io::Error,
    },

    #[error("Zed executable ('zed') was not found on PATH or via ZEDKATA_ZED_BIN")]
    ZedNotFound,

    #[error("Failed to execute Zed binary {bin:?}: {source}")]
    ZedExecutionError {
        bin: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to load or save session state at {path:?}: {source}")]
    SessionError {
        path: PathBuf,
        source: Box<CoreError>,
    },
}

#[derive(Debug, Clone)]
pub struct LaunchResult {
    pub lesson_id: String,
    pub exercise_id: String,
    pub buffer_path: PathBuf,
    pub target_arg: String,
    pub attempt_id: String,
}

/// Resolves the exercises directory where writable copies of exercise fixtures live.
///
/// Precedence:
/// 1. `ZEDKATA_EXERCISES_DIR` environment variable.
/// 2. `<session_path_parent>/exercises`.
pub fn resolve_exercises_dir(session_path: &Path) -> PathBuf {
    if let Ok(val) = std::env::var("ZEDKATA_EXERCISES_DIR") {
        if !val.trim().is_empty() {
            return PathBuf::from(val);
        }
    }
    let parent = session_path.parent().unwrap_or_else(|| Path::new("."));
    parent.join("exercises")
}

/// Formats a path cleanly for CLI argument passing, stripping any Windows extended UNC prefix
/// (`\\?\`) so that downstream right-to-left line:col parsing remains robust.
pub fn clean_path_for_cli(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s.to_string()
    }
}

/// Formats the launch target argument for Zed CLI.
///
/// Line and column are 1-based (verified in ZK-018.1 / OQ-10).
/// If cursor position `[line, column]` is provided, appends `:<line>:<col>`.
/// Quotes are handled automatically by `std::process::Command` when arguments contain spaces.
pub fn format_launch_target(buffer_path: &Path, cursor: Option<[u32; 2]>) -> String {
    let clean_path = clean_path_for_cli(buffer_path);
    if let Some([line, col]) = cursor {
        format!("{}:{}:{}", clean_path, line, col)
    } else {
        clean_path
    }
}

/// Discovers the Zed executable on the system.
///
/// Precedence:
/// 1. `ZEDKATA_ZED_BIN` or `ZED_BIN` environment variables.
/// 2. Search `PATH` directories for `zed` (and on Windows: `zed.exe`, `zed.cmd`, `zed.bat`).
/// 3. Standard platform install locations (e.g. `%LOCALAPPDATA%\Programs\Zed\bin\zed.exe`).
pub fn find_zed_binary() -> Result<PathBuf, LauncherError> {
    if let Ok(bin_override) = std::env::var("ZEDKATA_ZED_BIN").or_else(|_| std::env::var("ZED_BIN"))
    {
        let p = PathBuf::from(bin_override.trim());
        if !p.as_os_str().is_empty() && (p.is_file() || p.exists()) {
            return Ok(p);
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        #[cfg(target_os = "windows")]
        let separator = ';';
        #[cfg(not(target_os = "windows"))]
        let separator = ':';

        for dir in path_var.split(separator) {
            let dir_path = Path::new(dir);
            #[cfg(target_os = "windows")]
            {
                let candidates = ["zed.exe", "zed.cmd", "zed.bat", "zed"];
                for candidate in candidates {
                    let full = dir_path.join(candidate);
                    if full.is_file() {
                        return Ok(full);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let full = dir_path.join("zed");
                if full.is_file() {
                    return Ok(full);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let p1 = PathBuf::from(&local_app_data).join(r"Programs\Zed\bin\zed.exe");
            if p1.is_file() {
                return Ok(p1);
            }
            let p2 = PathBuf::from(&local_app_data).join(r"Programs\Zed\Zed.exe");
            if p2.is_file() {
                return Ok(p2);
            }
        }
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let p = PathBuf::from(&program_files).join(r"Zed\Zed.exe");
            if p.is_file() {
                return Ok(p);
            }
        }
    }

    Err(LauncherError::ZedNotFound)
}

/// Creates a writable exercise copy of the starting fixture in the exercises directory.
///
/// Returns the destination path on success. Does not modify session state.
pub fn prepare_exercise_buffer(
    catalog: &Catalog,
    lesson_id: &str,
    exercise_id: &str,
    exercises_dir: &Path,
) -> Result<PathBuf, LauncherError> {
    let start_fixture = catalog
        .resolve_fixture(lesson_id, exercise_id, true)
        .map_err(|source| LauncherError::FixtureResolutionError {
            lesson_id: lesson_id.to_string(),
            exercise_id: exercise_id.to_string(),
            source: Box::new(source),
        })?;

    if !start_fixture.is_file() {
        return Err(LauncherError::FixtureNotFound(start_fixture));
    }

    let target_dir = exercises_dir.join(lesson_id);
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        return Err(LauncherError::CopyError {
            src: start_fixture,
            dest: target_dir,
            source: e,
        });
    }

    let dest_path = target_dir.join(format!("{}.kata", exercise_id));

    // Copy start fixture to destination
    if let Err(e) = std::fs::copy(&start_fixture, &dest_path) {
        return Err(LauncherError::CopyError {
            src: start_fixture,
            dest: dest_path,
            source: e,
        });
    }

    // Ensure writable permissions on destination copy
    if let Ok(metadata) = std::fs::metadata(&dest_path) {
        let mut permissions = metadata.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        let _ = std::fs::set_permissions(&dest_path, permissions);
    }

    Ok(dest_path)
}

/// Spawns Zed CLI with the formatted launch target using separate arguments.
pub fn spawn_zed_process(zed_bin: &Path, target_arg: &str) -> Result<Child, LauncherError> {
    Command::new(zed_bin)
        .arg("-n")
        .arg(target_arg)
        .spawn()
        .map_err(|source| LauncherError::ZedExecutionError {
            bin: zed_bin.to_path_buf(),
            source,
        })
}

/// Full transactional exercise launch:
/// 1. Checks prerequisites (returns error if lesson is locked).
/// 2. Locates Zed executable (returns error if missing).
/// 3. Prepares writable exercise buffer (returns error if copy fails).
/// 4. Spawns Zed CLI process with separate arguments (`-n`, `target_arg`).
/// 5. ONLY upon successful CLI spawn, updates active attempt in `session.json`.
///
/// Guarantees: Missing CLI or copy failure does NOT leave a false active state.
pub fn launch_exercise(
    session_path: &Path,
    catalog: &Catalog,
    lesson_id: &str,
    exercise_index: usize,
    exercises_dir_override: Option<&Path>,
    zed_bin_override: Option<&Path>,
) -> Result<LaunchResult, LauncherError> {
    // 1. Validate lesson existence in catalog
    let entry = catalog
        .get(lesson_id)
        .ok_or_else(|| LauncherError::LessonNotFound(lesson_id.to_string()))?;

    // Load current session state to check prerequisites
    let mut session =
        SessionState::load_or_init(session_path).map_err(|source| LauncherError::SessionError {
            path: session_path.to_path_buf(),
            source: Box::new(source),
        })?;

    // Check prerequisites
    let mut missing_reqs = Vec::new();
    for req in &entry.lesson.requires {
        let passed = session
            .history
            .iter()
            .any(|h| h.lesson_id == *req && h.passed);
        if !passed {
            missing_reqs.push(req.clone());
        }
    }
    if !missing_reqs.is_empty() {
        return Err(LauncherError::LessonLocked(
            lesson_id.to_string(),
            missing_reqs.join(", "),
        ));
    }

    // 2. Validate exercise index
    let exercise = entry.lesson.exercises.get(exercise_index).ok_or_else(|| {
        LauncherError::ExerciseIndexOutOfBounds {
            lesson_id: lesson_id.to_string(),
            index: exercise_index,
            total: entry.lesson.exercises.len(),
        }
    })?;

    // 3. Locate Zed executable
    let zed_bin = match zed_bin_override {
        Some(bin) => bin.to_path_buf(),
        None => find_zed_binary()?,
    };

    // 4. Prepare writable exercise buffer copy
    let exercises_dir = match exercises_dir_override {
        Some(dir) => dir.to_path_buf(),
        None => resolve_exercises_dir(session_path),
    };

    let buffer_path = prepare_exercise_buffer(catalog, lesson_id, &exercise.id, &exercises_dir)?;

    // 5. Format launch target argument
    let target_arg = format_launch_target(&buffer_path, exercise.cursor);

    // 6. Spawn Zed CLI process
    let _child = spawn_zed_process(&zed_bin, &target_arg)?;

    // 7. Transactional update: Only update session state AFTER successful spawn
    let attempt_id = format!(
        "attempt_{}_{}",
        Utc::now().timestamp_millis(),
        std::process::id()
    );
    let clean_buffer_str = clean_path_for_cli(&buffer_path);

    session.switch_exercise(lesson_id.to_string(), exercise_index);
    session.start_attempt(attempt_id.clone(), Some(clean_buffer_str));
    session
        .save_to_file(session_path)
        .map_err(|source| LauncherError::SessionError {
            path: session_path.to_path_buf(),
            source: Box::new(source),
        })?;

    Ok(LaunchResult {
        lesson_id: lesson_id.to_string(),
        exercise_id: exercise.id.clone(),
        buffer_path,
        target_arg,
        attempt_id,
    })
}

/// Resets the active attempt for the current exercise:
/// 1. Verifies that an active exercise is selected in session state.
/// 2. Restores the writable exercise buffer on disk from the pristine start fixture.
/// 3. Issues a fresh attempt ID in session state with cleared edits and None verdict.
/// 4. Atomically persists the updated session state to disk.
pub fn reset_exercise_attempt(
    session_path: &Path,
    catalog: &Catalog,
    exercises_dir_override: Option<&Path>,
) -> Result<LaunchResult, LauncherError> {
    let mut session =
        SessionState::load_or_init(session_path).map_err(|source| LauncherError::SessionError {
            path: session_path.to_path_buf(),
            source: Box::new(source),
        })?;

    if session.current.lesson_id.is_empty() {
        return Err(LauncherError::LessonNotFound(
            "No active lesson currently selected in session".to_string(),
        ));
    }

    let lesson_id = session.current.lesson_id.clone();
    let exercise_index = session.current.exercise_index;

    let entry = catalog
        .get(&lesson_id)
        .ok_or_else(|| LauncherError::LessonNotFound(lesson_id.clone()))?;

    let exercise = entry.lesson.exercises.get(exercise_index).ok_or_else(|| {
        LauncherError::ExerciseIndexOutOfBounds {
            lesson_id: lesson_id.clone(),
            index: exercise_index,
            total: entry.lesson.exercises.len(),
        }
    })?;

    let exercises_dir = match exercises_dir_override {
        Some(d) => d.to_path_buf(),
        None => resolve_exercises_dir(session_path),
    };

    // Restore the pristine start fixture into the writable buffer file
    let buffer_path = prepare_exercise_buffer(catalog, &lesson_id, &exercise.id, &exercises_dir)?;
    let clean_buffer_str = clean_path_for_cli(&buffer_path);

    // Issue a fresh attempt ID and clear prior verdict and edit history
    let attempt_id = format!(
        "attempt_{}_{}",
        Utc::now().timestamp_millis(),
        std::process::id()
    );

    session.retry_attempt(attempt_id.clone());
    session.current.buffer_path = Some(clean_buffer_str);
    session.current.opened_at = Some(Utc::now());

    session
        .save_to_file(session_path)
        .map_err(|source| LauncherError::SessionError {
            path: session_path.to_path_buf(),
            source: Box::new(source),
        })?;

    let target_arg = format_launch_target(&buffer_path, exercise.cursor);

    Ok(LaunchResult {
        lesson_id,
        exercise_id: exercise.id.clone(),
        buffer_path,
        target_arg,
        attempt_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_launch_target_with_cursor() {
        let p = Path::new("C:/Users/Dan/test path/exercise.kata");
        let formatted = format_launch_target(p, Some([3, 12]));
        assert_eq!(formatted, "C:/Users/Dan/test path/exercise.kata:3:12");
    }

    #[test]
    fn test_format_launch_target_without_cursor() {
        let p = Path::new("C:/Users/Dan/test path/exercise.kata");
        let formatted = format_launch_target(p, None);
        assert_eq!(formatted, "C:/Users/Dan/test path/exercise.kata");
    }

    #[test]
    fn test_clean_path_for_cli_strips_extended_unc() {
        let p = Path::new(r"\\?\C:\Users\Dan\test.kata");
        let cleaned = clean_path_for_cli(p);
        assert_eq!(cleaned, r"C:\Users\Dan\test.kata");

        let p2 = Path::new(r"C:\Users\Dan\test.kata");
        let cleaned2 = clean_path_for_cli(p2);
        assert_eq!(cleaned2, r"C:\Users\Dan\test.kata");
    }
}
