use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Failed to read lesson file '{path}': {source}")]
    LessonIoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse lesson TOML{}: {message}", format_location(.path, *.line, *.col))]
    LessonParseError {
        path: Option<PathBuf>,
        message: String,
        line: Option<usize>,
        col: Option<usize>,
    },

    #[error("Validation error in field '{field}'{}: {message}", format_path(.path))]
    LessonValidationError {
        path: Option<PathBuf>,
        field: String,
        message: String,
    },

    #[error("Duplicate lesson ID '{id}' found in '{first_path}' and '{second_path}'")]
    DuplicateLessonId {
        id: String,
        first_path: PathBuf,
        second_path: PathBuf,
    },

    #[error("Duplicate exercise ID '{exercise_id}' within lesson '{lesson_id}'")]
    DuplicateExerciseId {
        lesson_id: String,
        exercise_id: String,
    },

    #[error("Fixture file for lesson '{lesson_id}', exercise '{exercise_id}' not found: {path}")]
    MissingFixture {
        lesson_id: String,
        exercise_id: String,
        path: PathBuf,
    },

    #[error("Lesson '{lesson_id}' requires prerequisite '{prerequisite}', which does not exist in the catalog")]
    MissingPrerequisite {
        lesson_id: String,
        prerequisite: String,
    },

    #[error("Circular dependency detected in lesson prerequisites: {}", cycle.join(" -> "))]
    PrerequisiteCycle { cycle: Vec<String> },

    #[error("Failed to serialize/deserialize session state: {0}")]
    SessionJsonError(#[from] serde_json::Error),

    #[error(
        "Incompatible session schema version: found {found}, maximum supported is {supported}"
    )]
    IncompatibleSessionVersion { found: u32, supported: u32 },

    #[error("Session state file corrupt or invalid at '{path}': {message}")]
    SessionCorruptError { path: PathBuf, message: String },

    #[error("Failed to resolve platform session path: {0}")]
    SessionPathError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Verification error: {0}")]
    VerificationError(String),
}

fn format_path(path: &Option<PathBuf>) -> String {
    match path {
        Some(p) => format!(" in '{}'", p.display()),
        None => String::new(),
    }
}

fn format_location(path: &Option<PathBuf>, line: Option<usize>, col: Option<usize>) -> String {
    let mut parts = Vec::new();
    if let Some(p) = path {
        parts.push(format!("'{}'", p.display()));
    }
    if let (Some(l), Some(c)) = (line, col) {
        parts.push(format!("line {}, col {}", l, c));
    } else if let Some(l) = line {
        parts.push(format!("line {}", l));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(" at "))
    }
}
