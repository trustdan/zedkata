use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use zedkata_core::{
    catalog::Catalog,
    default_session_path,
    session::{SessionState, Verdict},
};
use zedkata_tui::terminal;

#[derive(Parser, Debug)]
#[command(
    name = "zedkata-tui",
    about = "Interactive terminal tutor and CLI harness for Zedkata vim drills",
    version
)]
struct Cli {
    /// Path to a specific session.json file (defaults to standard platform state path)
    #[arg(short, long)]
    session: Option<PathBuf>,

    /// Path to curriculum lessons directory (defaults to 'lessons')
    #[arg(long)]
    lessons_dir: Option<PathBuf>,

    /// Test flag: trigger error after terminal setup to verify cooked-mode restoration
    #[arg(long, hide = true)]
    test_error: bool,

    /// Test flag: trigger panic after terminal setup to verify panic-hook restoration
    #[arg(long, hide = true)]
    test_panic: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Select an active lesson and exercise in session state
    Select {
        /// Lesson ID (e.g. 'motions-01-word')
        lesson_id: String,

        /// 0-indexed exercise index within the lesson (defaults to 0)
        #[arg(default_value_t = 0)]
        exercise_index: usize,
    },

    /// Launch an exercise in Zed editor at the specified position
    Launch {
        /// Lesson ID (defaults to active lesson in session or first runnable lesson)
        lesson_id: Option<String>,

        /// 0-indexed exercise index within the lesson (defaults to 0)
        #[arg(default_value_t = 0)]
        exercise_index: usize,
    },

    /// Display current session and exercise status
    Status,

    /// Reset the active attempt for the current exercise
    Reset,

    /// Advance to the next exercise in the curriculum
    Next,
}

fn resolve_session_path(cli_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_path {
        Ok(path)
    } else {
        default_session_path().context("Could not resolve standard platform session.json path")
    }
}

fn resolve_lessons_dir(cli_path: Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_path {
        path
    } else if Path::new("lessons").is_dir() {
        PathBuf::from("lessons")
    } else if Path::new("../../lessons").is_dir() {
        PathBuf::from("../../lessons")
    } else {
        PathBuf::from("lessons")
    }
}

fn handle_select(
    session_path: &Path,
    lessons_dir: &Path,
    lesson_id: &str,
    exercise_index: usize,
) -> Result<()> {
    let mut session = SessionState::load_or_init(session_path)
        .with_context(|| format!("Failed to load session from {:?}", session_path))?;

    session.switch_exercise(lesson_id.to_string(), exercise_index);
    session
        .save_to_file(session_path)
        .with_context(|| format!("Failed to save session to {:?}", session_path))?;

    println!("Selected exercise:");
    println!("  Lesson:   {}", lesson_id);
    println!("  Index:    {}", exercise_index);
    println!("  Session:  {:?}", session_path);

    if let Ok(catalog) = Catalog::load_from_dir(lessons_dir) {
        if let Some(entry) = catalog.get(lesson_id) {
            if let Some(ex) = entry.lesson.exercises.get(exercise_index) {
                println!("  Task:     {}", ex.task);
                if let Ok(fixture_path) = catalog.resolve_fixture(lesson_id, &ex.id, true) {
                    println!("  Fixture:  {:?}", fixture_path);
                    println!();
                    println!("To practice in Zed, open the fixture file:");
                    println!("  zed {:?}", fixture_path);
                    return Ok(());
                }
            }
        }
    }

    println!();
    println!("Active exercise updated in session state.");
    Ok(())
}

fn handle_status(session_path: &Path) -> Result<()> {
    let session = SessionState::load_or_init(session_path)
        .with_context(|| format!("Failed to load session from {:?}", session_path))?;

    println!("Zedkata Session Status:");
    println!("  Session File:   {:?}", session_path);
    println!("  Schema Version: {}", session.version);
    println!("  Student Start:  {}", session.student.started_at);
    println!(
        "  Current Lesson: {}",
        if session.current.lesson_id.is_empty() {
            "(none)"
        } else {
            &session.current.lesson_id
        }
    );
    println!("  Exercise Index: {}", session.current.exercise_index);
    if let Some(ref bp) = session.current.buffer_path {
        println!("  Buffer Path:    {}", bp);
    }
    if let Some(ref opened) = session.current.opened_at {
        println!("  Opened At:      {}", opened);
    }

    if let Some(ref att) = session.attempt {
        println!();
        println!("Active Attempt:");
        println!("  Attempt ID:     {}", att.id);
        println!("  Edit Count:     {}", att.edit_count);
        match &att.verdict {
            Some(Verdict::Passed { edits, par, score }) => {
                println!(
                    "  Verdict:        PASSED (edits: {}, par: {:?}, score: {})",
                    edits, par, score
                );
            }
            Some(Verdict::Failed { reason }) => {
                println!("  Verdict:        FAILED ({})", reason);
            }
            Some(Verdict::InProgress) => {
                println!("  Verdict:        IN PROGRESS");
            }
            None => {
                println!("  Verdict:        (none)");
            }
        }
    } else {
        println!();
        println!("Active Attempt:   (none)");
    }

    println!();
    println!(
        "History Count:    {} completed exercise(s)",
        session.history.len()
    );
    for (i, h) in session.history.iter().enumerate() {
        println!(
            "  [{}] {}/{} - {} (score: {}, edits: {})",
            i + 1,
            h.lesson_id,
            h.exercise_id,
            if h.passed { "PASSED" } else { "FAILED" },
            h.score,
            h.actual_edits
        );
    }

    Ok(())
}

fn handle_reset(session_path: &Path, lessons_dir: &Path) -> Result<()> {
    if let Ok(catalog) = Catalog::load_from_dir(lessons_dir) {
        if let Ok(result) =
            zedkata_tui::launcher::reset_exercise_attempt(session_path, &catalog, None)
        {
            println!(
                "Reset active attempt for lesson '{}' (exercise '{}').",
                result.lesson_id, result.exercise_id
            );
            println!("  Buffer restored: {:?}", result.buffer_path);
            println!("  New Attempt ID:  {}", result.attempt_id);
            return Ok(());
        }
    }

    let mut session = SessionState::load_or_init(session_path)
        .with_context(|| format!("Failed to load session from {:?}", session_path))?;

    let now_id = format!("attempt_{}", chrono::Utc::now().timestamp_millis());
    session.retry_attempt(now_id.clone());
    session
        .save_to_file(session_path)
        .with_context(|| format!("Failed to save session to {:?}", session_path))?;

    println!(
        "Reset active attempt for lesson '{}' (index {}).",
        session.current.lesson_id, session.current.exercise_index
    );
    println!("New Attempt ID: {}", now_id);
    Ok(())
}

fn handle_next(session_path: &Path, lessons_dir: &Path) -> Result<()> {
    let mut app =
        zedkata_tui::app::App::load(session_path.to_path_buf(), lessons_dir.to_path_buf());
    if app.advance_and_activate()? {
        let lesson_id = app
            .current_lesson()
            .map(|l| l.lesson.id.clone())
            .unwrap_or_default();
        let ex_idx = app.selected_exercise_idx;
        let ex_id = app
            .current_exercise()
            .map(|ex| ex.id.clone())
            .unwrap_or_default();

        println!("Advanced to next exercise:");
        println!("  Lesson:   {}", lesson_id);
        println!("  Index:    {}", ex_idx);
        println!("  ID:       {}", ex_id);
        if let Some(ex) = app.current_exercise() {
            println!("  Task:     {}", ex.task);
        }
        if let Some(ref cat) = app.catalog {
            if let Ok(fixture_path) = cat.resolve_fixture(&lesson_id, &ex_id, true) {
                println!("  Fixture:  {:?}", fixture_path);
            }
        }
        println!();
        println!("Active exercise updated in session state.");
    } else if let Some(msg) = app.status_message {
        println!("{}", msg);
    }
    Ok(())
}

fn handle_launch(
    session_path: &Path,
    lessons_dir: &Path,
    lesson_id: Option<String>,
    exercise_index: usize,
) -> Result<()> {
    let catalog = Catalog::load_from_dir(lessons_dir)
        .with_context(|| format!("Failed to load catalog from {:?}", lessons_dir))?;

    let session = SessionState::load_or_init(session_path)
        .with_context(|| format!("Failed to load session from {:?}", session_path))?;

    let lid = match lesson_id {
        Some(id) => id,
        None if !session.current.lesson_id.is_empty() => session.current.lesson_id.clone(),
        None => {
            let runnable = catalog.runnable_lessons();
            if let Some(first) = runnable.first() {
                first.lesson.id.clone()
            } else {
                let all = catalog.all_lessons();
                let first = all.first().context("No lessons available in catalog")?;
                first.lesson.id.clone()
            }
        }
    };

    println!("Launching exercise in Zed:");
    println!("  Lesson:    {}", lid);
    println!("  Index:     {}", exercise_index);
    println!("  Session:   {:?}", session_path);

    let result = zedkata_tui::launcher::launch_exercise(
        session_path,
        &catalog,
        &lid,
        exercise_index,
        None,
        None,
    )?;

    println!("  Exercise:  {}", result.exercise_id);
    println!("  Buffer:    {:?}", result.buffer_path);
    println!("  Target:    {}", result.target_arg);
    println!("  Attempt:   {}", result.attempt_id);
    println!();
    println!("Exercise launched successfully in Zed.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let session_path = resolve_session_path(cli.session)?;
    let lessons_dir = resolve_lessons_dir(cli.lessons_dir);

    match cli.command {
        Some(Commands::Select {
            lesson_id,
            exercise_index,
        }) => {
            handle_select(&session_path, &lessons_dir, &lesson_id, exercise_index)?;
        }
        Some(Commands::Launch {
            lesson_id,
            exercise_index,
        }) => {
            handle_launch(&session_path, &lessons_dir, lesson_id, exercise_index)?;
        }
        Some(Commands::Status) => {
            handle_status(&session_path)?;
        }
        Some(Commands::Reset) => {
            handle_reset(&session_path, &lessons_dir)?;
        }
        Some(Commands::Next) => {
            handle_next(&session_path, &lessons_dir)?;
        }
        None => {
            terminal::run_tui_app(&session_path, &lessons_dir, cli.test_error, cli.test_panic)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_and_status_headless() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!(
            "zedkata_tui_test_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&temp_dir)?;
        let session_file = temp_dir.join("session.json");
        let lessons_dir = resolve_lessons_dir(None);

        handle_select(&session_file, &lessons_dir, "motions-01-word", 0)?;

        let session = SessionState::load_from_file(&session_file)?;
        assert_eq!(session.current.lesson_id, "motions-01-word");
        assert_eq!(session.current.exercise_index, 0);

        handle_reset(&session_file, &lessons_dir)?;
        let session_after_reset = SessionState::load_from_file(&session_file)?;
        assert!(session_after_reset.attempt.is_some());

        handle_next(&session_file, &lessons_dir)?;
        let session_after_next = SessionState::load_from_file(&session_file)?;
        assert_eq!(session_after_next.current.exercise_index, 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
