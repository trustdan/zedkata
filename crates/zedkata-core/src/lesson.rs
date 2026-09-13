use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A curriculum lesson containing one or more exercises and optional walkthroughs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub module: Option<u32>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(default)]
    pub draft: bool,
    pub prose: String,
    #[serde(default, rename = "exercise")]
    pub exercises: Vec<Exercise>,
    #[serde(default, rename = "walkthrough")]
    pub walkthroughs: Vec<Walkthrough>,
}

/// A specific drill within a lesson.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub task: String,
    pub start: String,
    pub goal: String,
    pub par_edits: Option<u32>,
    pub max_edits: Option<u32>,
    pub hint: Option<String>,
    pub forbid: Option<ForbidRules>,
}

/// A guided walkthrough targeting conceptual or non-drillable topics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Walkthrough {
    pub id: String,
    pub title: String,
    pub prompt: String,
}

/// Verification rules that immediately fail an attempt if triggered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ForbidRules {
    pub backspace_run: Option<u32>,
    #[serde(default)]
    pub unsupported: Vec<String>,
}

#[derive(Deserialize)]
struct RawLessonFile {
    lesson: RawLessonHeader,
    #[serde(default, rename = "exercise")]
    exercises: Vec<RawExercise>,
    #[serde(default, rename = "walkthrough")]
    walkthroughs: Vec<RawWalkthrough>,
}

#[derive(Deserialize)]
struct RawLessonHeader {
    id: String,
    title: String,
    module: Option<u32>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    concepts: Vec<String>,
    #[serde(default)]
    draft: Option<bool>,
    prose: String,
}

#[derive(Deserialize)]
struct RawExercise {
    id: String,
    task: String,
    start: String,
    goal: String,
    par_edits: Option<u32>,
    max_edits: Option<u32>,
    hint: Option<String>,
    forbid: Option<toml::Table>,
}

#[derive(Deserialize)]
struct RawWalkthrough {
    id: String,
    title: String,
    prompt: String,
}

impl Lesson {
    /// Parse a lesson from a TOML string, validating schema and verification rules.
    pub fn from_toml(content: &str, path: Option<&Path>) -> Result<Self, CoreError> {
        let raw: RawLessonFile = toml::from_str(content).map_err(|err| {
            let (line, col) = err
                .span()
                .and_then(|span| calculate_line_col(content, span.start))
                .unzip();

            CoreError::LessonParseError {
                path: path.map(PathBuf::from),
                message: err.message().to_string(),
                line,
                col,
            }
        })?;

        let path_opt = path.map(PathBuf::from);

        // Validate lesson header
        validate_id(&raw.lesson.id, "lesson.id", path_opt.as_deref())?;
        if raw.lesson.title.trim().is_empty() {
            return Err(CoreError::LessonValidationError {
                path: path_opt.clone(),
                field: "lesson.title".to_string(),
                message: "Lesson title cannot be empty".to_string(),
            });
        }
        if raw.lesson.prose.trim().is_empty() {
            return Err(CoreError::LessonValidationError {
                path: path_opt.clone(),
                field: "lesson.prose".to_string(),
                message: "Lesson prose cannot be empty".to_string(),
            });
        }

        let is_draft = raw.lesson.draft.unwrap_or(false);

        // Validate exercises
        let mut exercises = Vec::with_capacity(raw.exercises.len());
        let mut seen_exercise_ids = HashSet::new();

        for raw_ex in raw.exercises {
            let field_prefix = format!("exercise.{}", raw_ex.id);
            validate_id(
                &raw_ex.id,
                &format!("{}.id", field_prefix),
                path_opt.as_deref(),
            )?;

            if !seen_exercise_ids.insert(raw_ex.id.clone()) {
                return Err(CoreError::DuplicateExerciseId {
                    lesson_id: raw.lesson.id.clone(),
                    exercise_id: raw_ex.id.clone(),
                });
            }

            if raw_ex.task.trim().is_empty() {
                return Err(CoreError::LessonValidationError {
                    path: path_opt.clone(),
                    field: format!("{}.task", field_prefix),
                    message: "Exercise task cannot be empty".to_string(),
                });
            }

            validate_fixture_path(
                &raw_ex.start,
                &format!("{}.start", field_prefix),
                path_opt.as_deref(),
            )?;
            validate_fixture_path(
                &raw_ex.goal,
                &format!("{}.goal", field_prefix),
                path_opt.as_deref(),
            )?;

            if let Some(par) = raw_ex.par_edits {
                if par == 0 {
                    return Err(CoreError::LessonValidationError {
                        path: path_opt.clone(),
                        field: format!("{}.par_edits", field_prefix),
                        message: "par_edits must be greater than 0".to_string(),
                    });
                }
            }

            if let Some(max) = raw_ex.max_edits {
                if max == 0 {
                    return Err(CoreError::LessonValidationError {
                        path: path_opt.clone(),
                        field: format!("{}.max_edits", field_prefix),
                        message: "max_edits must be greater than 0".to_string(),
                    });
                }
            }

            if let (Some(par), Some(max)) = (raw_ex.par_edits, raw_ex.max_edits) {
                if par > max {
                    return Err(CoreError::LessonValidationError {
                        path: path_opt.clone(),
                        field: format!("{}.par_edits", field_prefix),
                        message: format!("par_edits ({}) cannot exceed max_edits ({})", par, max),
                    });
                }
            }

            let forbid = if let Some(forbid_table) = raw_ex.forbid {
                let rules = parse_forbid_rules(
                    &forbid_table,
                    &field_prefix,
                    path_opt.as_deref(),
                    is_draft,
                )?;
                Some(rules)
            } else {
                None
            };

            exercises.push(Exercise {
                id: raw_ex.id,
                task: raw_ex.task,
                start: raw_ex.start,
                goal: raw_ex.goal,
                par_edits: raw_ex.par_edits,
                max_edits: raw_ex.max_edits,
                hint: raw_ex.hint,
                forbid,
            });
        }

        // Validate walkthroughs
        let mut walkthroughs = Vec::with_capacity(raw.walkthroughs.len());
        let mut seen_walkthrough_ids = HashSet::new();

        for raw_wt in raw.walkthroughs {
            let field_prefix = format!("walkthrough.{}", raw_wt.id);
            validate_id(
                &raw_wt.id,
                &format!("{}.id", field_prefix),
                path_opt.as_deref(),
            )?;

            if !seen_walkthrough_ids.insert(raw_wt.id.clone()) {
                return Err(CoreError::LessonValidationError {
                    path: path_opt.clone(),
                    field: format!("{}.id", field_prefix),
                    message: format!("Duplicate walkthrough ID '{}'", raw_wt.id),
                });
            }

            if raw_wt.title.trim().is_empty() {
                return Err(CoreError::LessonValidationError {
                    path: path_opt.clone(),
                    field: format!("{}.title", field_prefix),
                    message: "Walkthrough title cannot be empty".to_string(),
                });
            }

            if raw_wt.prompt.trim().is_empty() {
                return Err(CoreError::LessonValidationError {
                    path: path_opt.clone(),
                    field: format!("{}.prompt", field_prefix),
                    message: "Walkthrough prompt cannot be empty".to_string(),
                });
            }

            walkthroughs.push(Walkthrough {
                id: raw_wt.id,
                title: raw_wt.title,
                prompt: raw_wt.prompt,
            });
        }

        Ok(Lesson {
            id: raw.lesson.id,
            title: raw.lesson.title,
            module: raw.lesson.module,
            requires: raw.lesson.requires,
            concepts: raw.lesson.concepts,
            draft: is_draft,
            prose: raw.lesson.prose,
            exercises,
            walkthroughs,
        })
    }

    /// Read and parse a lesson from a filesystem path.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|err| CoreError::LessonIoError {
            path: path.to_path_buf(),
            source: err,
        })?;

        Self::from_toml(&content, Some(path))
    }
}

fn validate_id(id: &str, field: &str, path: Option<&Path>) -> Result<(), CoreError> {
    if id.is_empty() {
        return Err(CoreError::LessonValidationError {
            path: path.map(PathBuf::from),
            field: field.to_string(),
            message: "ID cannot be empty".to_string(),
        });
    }

    let is_valid = id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !is_valid {
        return Err(CoreError::LessonValidationError {
            path: path.map(PathBuf::from),
            field: field.to_string(),
            message: format!(
                "ID '{}' must contain only lowercase ASCII letters, digits, and hyphens",
                id
            ),
        });
    }

    Ok(())
}

fn validate_fixture_path(
    fixture_path: &str,
    field: &str,
    path: Option<&Path>,
) -> Result<(), CoreError> {
    if fixture_path.is_empty() {
        return Err(CoreError::LessonValidationError {
            path: path.map(PathBuf::from),
            field: field.to_string(),
            message: "Fixture path cannot be empty".to_string(),
        });
    }

    let p = Path::new(fixture_path);
    if p.is_absolute() {
        return Err(CoreError::LessonValidationError {
            path: path.map(PathBuf::from),
            field: field.to_string(),
            message: format!(
                "Fixture path '{}' must be relative, not absolute",
                fixture_path
            ),
        });
    }

    if !fixture_path.ends_with(".kata") {
        return Err(CoreError::LessonValidationError {
            path: path.map(PathBuf::from),
            field: field.to_string(),
            message: format!(
                "Fixture path '{}' must end with .kata extension",
                fixture_path
            ),
        });
    }

    Ok(())
}

fn parse_forbid_rules(
    table: &toml::Table,
    field_prefix: &str,
    path: Option<&Path>,
    is_draft: bool,
) -> Result<ForbidRules, CoreError> {
    let mut rules = ForbidRules::default();

    for (key, value) in table {
        match key.as_str() {
            "backspace_run" => {
                let count =
                    value.as_integer().and_then(
                        |v| {
                            if v > 0 {
                                u32::try_from(v).ok()
                            } else {
                                None
                            }
                        },
                    );

                match count {
                    Some(c) => rules.backspace_run = Some(c),
                    None => {
                        return Err(CoreError::LessonValidationError {
                            path: path.map(PathBuf::from),
                            field: format!("{}.forbid.backspace_run", field_prefix),
                            message: "backspace_run must be an integer greater than 0".to_string(),
                        });
                    }
                }
            }
            "visual_line_select" => {
                if is_draft {
                    rules.unsupported.push(key.clone());
                } else {
                    return Err(CoreError::LessonValidationError {
                        path: path.map(PathBuf::from),
                        field: format!("{}.forbid.visual_line_select", field_prefix),
                        message: "visual_line_select is unobservable: LSP cannot observe visual selection without cursor/keystroke APIs".to_string(),
                    });
                }
            }
            unknown => {
                if is_draft {
                    rules.unsupported.push(key.clone());
                } else {
                    return Err(CoreError::LessonValidationError {
                        path: path.map(PathBuf::from),
                        field: format!("{}.forbid.{}", field_prefix, unknown),
                        message: format!("unknown or unsupported forbid rule '{}'", unknown),
                    });
                }
            }
        }
    }

    Ok(rules)
}

fn calculate_line_col(content: &str, offset: usize) -> Option<(usize, usize)> {
    if offset > content.len() {
        return None;
    }

    let mut line = 1;
    let mut col = 1;
    for (i, c) in content.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Some((line, col))
}
