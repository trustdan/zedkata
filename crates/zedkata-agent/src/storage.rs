use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use zedkata_core::{catalog::Catalog, error::CoreError, lesson::Lesson};

use crate::generator::{CandidateExercise, CandidateStatus};

/// Errors encountered when validating or storing generated lesson assets under REQ-28.
#[derive(Debug, Error)]
pub enum GeneratedAssetError {
    /// TOML syntax or structural deserialization error.
    #[error("Malformed lesson TOML: {0}")]
    MalformedToml(String),

    /// Validation rule failure (e.g. invalid edit budget, empty title, unobservable rule).
    #[error("Invalid exercise rules: {0}")]
    InvalidRules(String),

    /// Missing, empty, or degenerate fixture buffer.
    #[error("Missing or invalid fixture: {0}")]
    MissingFixture(String),

    /// Path traversal attempt escaping the generated content root.
    #[error("Path traversal rejected for '{path}': {reason}")]
    PathTraversal { path: String, reason: String },

    /// Lesson ID collision with an existing lesson (preserves existing lesson).
    #[error(
        "Lesson ID collision for '{lesson_id}': existing lesson preserved at '{existing_path}'"
    )]
    IdCollision {
        lesson_id: String,
        existing_path: PathBuf,
    },

    /// Filesystem I/O error during asset operations.
    #[error("Storage I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Paths of stored assets on disk in the configured user data directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAssetPaths {
    /// Full path to the written `<lesson_id>.toml` file.
    pub toml_path: PathBuf,
    /// Full path to the written starting buffer fixture (`.kata`).
    pub start_fixture_path: PathBuf,
    /// Full path to the written goal buffer fixture (`.kata`).
    pub goal_fixture_path: PathBuf,
}

/// Verifies that `rel_path` is strictly relative and does not escape `root_dir`.
///
/// Rejects:
/// - Absolute paths (leading `/`, `\`, or Windows drive letters)
/// - Path components containing `..` (`Component::ParentDir`)
/// - Prefix or root directory components
/// - Empty paths
pub fn verify_path_within_root(
    root: &Path,
    rel_path: &str,
) -> Result<PathBuf, GeneratedAssetError> {
    if rel_path.trim().is_empty() {
        return Err(GeneratedAssetError::PathTraversal {
            path: rel_path.to_string(),
            reason: "Path cannot be empty".to_string(),
        });
    }

    let p = Path::new(rel_path);

    if p.is_absolute() {
        return Err(GeneratedAssetError::PathTraversal {
            path: rel_path.to_string(),
            reason: "Path must be relative, not absolute".to_string(),
        });
    }

    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                return Err(GeneratedAssetError::PathTraversal {
                    path: rel_path.to_string(),
                    reason: "Path contains parent directory traversal ('..')".to_string(),
                });
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(GeneratedAssetError::PathTraversal {
                    path: rel_path.to_string(),
                    reason: "Path contains root or drive prefix component".to_string(),
                });
            }
            _ => {}
        }
    }

    // Double check with root
    let joined = root.join(p);
    Ok(joined)
}

/// Validates all candidate exercise assets:
/// 1. Verifies lesson and exercise IDs contain valid characters and no directory traversal.
/// 2. Verifies fixture paths are strictly relative, end with `.kata`, and do not escape `root_dir`.
/// 3. Rejects empty, missing, or identical start and goal fixture buffers.
/// 4. Validates TOML formatting and exercise rules using `Lesson::from_toml`.
/// 5. Ensures consistency between Candidate metadata and generated TOML fields.
pub fn validate_candidate(
    candidate: &CandidateExercise,
    root_dir: &Path,
) -> Result<(), GeneratedAssetError> {
    // 1. Validate lesson ID format and traversal safety
    if candidate.lesson_id.is_empty() {
        return Err(GeneratedAssetError::PathTraversal {
            path: candidate.lesson_id.clone(),
            reason: "Lesson ID cannot be empty".to_string(),
        });
    }

    let valid_id_chars = candidate
        .lesson_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid_id_chars
        || candidate.lesson_id.contains('/')
        || candidate.lesson_id.contains('\\')
        || candidate.lesson_id == "."
        || candidate.lesson_id == ".."
    {
        return Err(GeneratedAssetError::PathTraversal {
            path: candidate.lesson_id.clone(),
            reason: "Lesson ID must contain only lowercase ASCII alphanumeric chars and hyphens"
                .to_string(),
        });
    }

    // Verify TOML filename path within root
    let toml_rel = format!("{}.toml", candidate.lesson_id);
    let _ = verify_path_within_root(root_dir, &toml_rel)?;

    // 2. Validate fixture paths
    let _ = verify_path_within_root(root_dir, &candidate.start_fixture_relative_path)?;
    let _ = verify_path_within_root(root_dir, &candidate.goal_fixture_relative_path)?;

    if !candidate.start_fixture_relative_path.ends_with(".kata") {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Start fixture path '{}' must end with .kata extension",
            candidate.start_fixture_relative_path
        )));
    }

    if !candidate.goal_fixture_relative_path.ends_with(".kata") {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Goal fixture path '{}' must end with .kata extension",
            candidate.goal_fixture_relative_path
        )));
    }

    // 3. Validate fixture content (Missing fixtures)
    if candidate.start_fixture_content.trim().is_empty() {
        return Err(GeneratedAssetError::MissingFixture(
            "Start fixture buffer content cannot be empty".to_string(),
        ));
    }

    if candidate.goal_fixture_content.trim().is_empty() {
        return Err(GeneratedAssetError::MissingFixture(
            "Goal fixture buffer content cannot be empty".to_string(),
        ));
    }

    if candidate.start_fixture_content == candidate.goal_fixture_content {
        return Err(GeneratedAssetError::MissingFixture(
            "Start and goal fixture buffers are identical; no motion change to practice"
                .to_string(),
        ));
    }

    // 4. Validate TOML schema and rules
    let parsed_lesson = Lesson::from_toml(&candidate.raw_toml, None).map_err(|err| match err {
        CoreError::LessonParseError { message, .. } => GeneratedAssetError::MalformedToml(message),
        CoreError::LessonValidationError { message, .. } => {
            if message.contains("parent directory traversal")
                || message.contains("must be relative")
            {
                GeneratedAssetError::PathTraversal {
                    path: String::new(),
                    reason: message,
                }
            } else {
                GeneratedAssetError::InvalidRules(message)
            }
        }
        other => GeneratedAssetError::InvalidRules(other.to_string()),
    })?;

    if parsed_lesson.exercises.is_empty() {
        return Err(GeneratedAssetError::InvalidRules(
            "Generated lesson must contain at least one drillable exercise".to_string(),
        ));
    }

    if parsed_lesson.id != candidate.lesson_id {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Lesson ID in TOML ('{}') does not match candidate lesson ID ('{}')",
            parsed_lesson.id, candidate.lesson_id
        )));
    }

    // Verify first exercise matches candidate
    let ex = &parsed_lesson.exercises[0];
    if ex.id != candidate.exercise_id {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Exercise ID in TOML ('{}') does not match candidate exercise ID ('{}')",
            ex.id, candidate.exercise_id
        )));
    }

    if ex.start != candidate.start_fixture_relative_path {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Start fixture path in TOML ('{}') does not match candidate path ('{}')",
            ex.start, candidate.start_fixture_relative_path
        )));
    }

    if ex.goal != candidate.goal_fixture_relative_path {
        return Err(GeneratedAssetError::InvalidRules(format!(
            "Goal fixture path in TOML ('{}') does not match candidate path ('{}')",
            ex.goal, candidate.goal_fixture_relative_path
        )));
    }

    let _ = verify_path_within_root(root_dir, &ex.start)?;
    let _ = verify_path_within_root(root_dir, &ex.goal)?;

    if ex.par_edits.is_none() || ex.par_edits == Some(0) {
        return Err(GeneratedAssetError::InvalidRules(
            "par_edits must be specified and greater than 0".to_string(),
        ));
    }

    if let (Some(par), Some(max)) = (ex.par_edits, ex.max_edits) {
        if par > max {
            return Err(GeneratedAssetError::InvalidRules(format!(
                "par_edits ({}) cannot exceed max_edits ({})",
                par, max
            )));
        }
    }

    Ok(())
}

/// Validates and stores generated assets together in `root_dir`.
///
/// In accordance with REQ-28, ADR 0005, and ZK-032.2:
/// - Rejects malformed TOML, invalid rules, missing fixtures, and path traversal.
/// - ID collisions preserve existing lessons: if a lesson with `candidate.lesson_id`
///   already exists on disk or in the optional catalog, it is preserved and the write is rejected.
/// - Valid assets write together (atomic multi-file write) to the configured user-data directory.
/// - On success, updates candidate status to `CandidateStatus::Persisted`.
pub fn store_candidate_assets(
    candidate: &mut CandidateExercise,
    root_dir: &Path,
    catalog: Option<&Catalog>,
) -> Result<StoredAssetPaths, GeneratedAssetError> {
    // 1. Perform complete validation
    validate_candidate(candidate, root_dir)?;
    candidate.status = CandidateStatus::Validated;

    // 2. Check for ID collisions to preserve existing lessons
    let toml_rel = format!("{}.toml", candidate.lesson_id);
    let toml_path = root_dir.join(&toml_rel);

    if toml_path.exists() {
        return Err(GeneratedAssetError::IdCollision {
            lesson_id: candidate.lesson_id.clone(),
            existing_path: toml_path,
        });
    }

    if let Some(cat) = catalog {
        if let Some(existing) = cat.get(&candidate.lesson_id) {
            return Err(GeneratedAssetError::IdCollision {
                lesson_id: candidate.lesson_id.clone(),
                existing_path: existing.path.clone(),
            });
        }
    }

    let start_path = root_dir.join(&candidate.start_fixture_relative_path);
    let goal_path = root_dir.join(&candidate.goal_fixture_relative_path);

    // 3. Prepare parent directories
    if let Some(parent) = toml_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = start_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = goal_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 4. Atomic write sequence: write to .tmp then rename
    let start_tmp = start_path.with_extension("kata.tmp");
    let goal_tmp = goal_path.with_extension("kata.tmp");
    let toml_tmp = toml_path.with_extension("toml.tmp");

    let write_result: Result<(), std::io::Error> = (|| {
        std::fs::write(&start_tmp, &candidate.start_fixture_content)?;
        std::fs::write(&goal_tmp, &candidate.goal_fixture_content)?;
        std::fs::write(&toml_tmp, &candidate.raw_toml)?;

        // Rename tmp to target paths
        std::fs::rename(&start_tmp, &start_path)?;
        std::fs::rename(&goal_tmp, &goal_path)?;
        std::fs::rename(&toml_tmp, &toml_path)?;
        Ok(())
    })();

    if let Err(e) = write_result {
        // Cleanup any orphaned temp or written files
        let _ = std::fs::remove_file(&start_tmp);
        let _ = std::fs::remove_file(&goal_tmp);
        let _ = std::fs::remove_file(&toml_tmp);
        let _ = std::fs::remove_file(&start_path);
        let _ = std::fs::remove_file(&goal_path);
        let _ = std::fs::remove_file(&toml_path);
        return Err(GeneratedAssetError::IoError(e));
    }

    // 5. Update lifecycle status
    candidate.status = CandidateStatus::Persisted;

    Ok(StoredAssetPaths {
        toml_path,
        start_fixture_path: start_path,
        goal_fixture_path: goal_path,
    })
}
