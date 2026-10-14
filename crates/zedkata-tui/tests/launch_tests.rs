use std::path::{Path, PathBuf};
use zedkata_core::{
    catalog::Catalog,
    session::{SessionState, Verdict},
};
use zedkata_tui::launcher::{
    clean_path_for_cli, format_launch_target, launch_exercise, prepare_exercise_buffer,
    resolve_exercises_dir, LauncherError,
};

fn setup_test_env(test_name: &str) -> (PathBuf, PathBuf, Catalog) {
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_launch_test_{}_{}_{}",
        test_name,
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lessons_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lessons");

    let catalog = Catalog::load_from_dir(&lessons_dir).expect("Failed to load catalog");
    let session_file = temp_dir.join("session.json");

    (temp_dir, session_file, catalog)
}

#[test]
fn test_resolve_exercises_dir_default_and_override() {
    let session_path = Path::new("C:/test/dir/session.json");
    let resolved = resolve_exercises_dir(session_path);
    assert_eq!(resolved, Path::new("C:/test/dir/exercises"));

    std::env::set_var("ZEDKATA_EXERCISES_DIR", "C:/custom/exercises");
    let overridden = resolve_exercises_dir(session_path);
    assert_eq!(overridden, Path::new("C:/custom/exercises"));
    std::env::remove_var("ZEDKATA_EXERCISES_DIR");
}

#[test]
fn test_prepare_exercise_buffer_in_path_with_spaces() {
    let (temp_dir, _session_file, catalog) = setup_test_env("spaces");
    let exercises_dir = temp_dir.join("student exercises with spaces");

    let buffer_path = prepare_exercise_buffer(
        &catalog,
        "motions-01-word",
        "delete-word-arg",
        &exercises_dir,
    )
    .expect("Failed to prepare exercise buffer");

    assert!(buffer_path.is_file());
    assert_eq!(
        buffer_path.file_name().unwrap().to_str().unwrap(),
        "delete-word-arg.kata"
    );

    // Verify content matches the start fixture
    let start_fixture = catalog
        .resolve_fixture("motions-01-word", "delete-word-arg", true)
        .unwrap();
    let expected = std::fs::read_to_string(&start_fixture).unwrap();
    let actual = std::fs::read_to_string(&buffer_path).unwrap();
    assert_eq!(actual, expected);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_format_launch_target_coordinates() {
    let path = Path::new(r"C:\Users\Dan\exercises\delete-word-arg.kata");
    let formatted_with_cursor = format_launch_target(path, Some([2, 5]));
    assert_eq!(
        formatted_with_cursor,
        r"C:\Users\Dan\exercises\delete-word-arg.kata:2:5"
    );

    let formatted_without_cursor = format_launch_target(path, None);
    assert_eq!(
        formatted_without_cursor,
        r"C:\Users\Dan\exercises\delete-word-arg.kata"
    );
}

#[test]
fn test_missing_cli_failure_does_not_leave_false_active_state() {
    let (temp_dir, session_file, catalog) = setup_test_env("missing_cli");

    // Initialize clean session on disk
    let initial_session = SessionState::default();
    initial_session.save_to_file(&session_file).unwrap();

    let nonexistent_zed = temp_dir.join("nonexistent_zed_binary_12345.exe");

    let res = launch_exercise(
        &session_file,
        &catalog,
        "motions-01-word",
        0,
        Some(&temp_dir.join("exercises")),
        Some(&nonexistent_zed),
    );

    // Launch must fail because Zed was not found / cannot be spawned
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(
        err,
        LauncherError::ZedExecutionError { .. } | LauncherError::ZedNotFound
    ));

    // Verify session.json on disk was NOT modified with a false active attempt
    let session_after = SessionState::load_from_file(&session_file).unwrap();
    assert!(
        session_after.attempt.is_none(),
        "Attempt must remain None on launch failure"
    );
    assert!(
        session_after.current.buffer_path.is_none(),
        "Buffer path must remain None on launch failure"
    );
    assert_eq!(session_after.current.lesson_id, "");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_locked_lesson_cannot_be_launched_and_leaves_no_active_state() {
    let (temp_dir, session_file, catalog) = setup_test_env("locked");

    let initial_session = SessionState::default();
    initial_session.save_to_file(&session_file).unwrap();

    // motions-03-textobjects requires motions-01-word and motions-02-find
    let res = launch_exercise(
        &session_file,
        &catalog,
        "motions-03-textobjects",
        0,
        Some(&temp_dir.join("exercises")),
        None,
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    match err {
        LauncherError::LessonLocked(lid, reqs) => {
            assert_eq!(lid, "motions-03-textobjects");
            assert!(reqs.contains("motions-01-word"));
        }
        other => panic!("Expected LessonLocked, got {:?}", other),
    }

    // Verify session state was not modified
    let session_after = SessionState::load_from_file(&session_file).unwrap();
    assert!(session_after.attempt.is_none());
    assert_eq!(session_after.current.lesson_id, "");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_successful_launch_with_mock_runner() {
    let (temp_dir, session_file, catalog) = setup_test_env("mock_launch");

    // Create a mock script/executable that records its arguments to a file
    let args_log = temp_dir.join("mock_args.txt");

    #[cfg(target_os = "windows")]
    let mock_bin = {
        let script = temp_dir.join("mock_zed.bat");
        let content = format!(
            "@echo off\r\necho %* > \"{}\"\r\nexit /b 0\r\n",
            args_log.display()
        );
        std::fs::write(&script, content).unwrap();
        script
    };

    #[cfg(not(target_os = "windows"))]
    let mock_bin = {
        use std::os::unix::fs::PermissionsExt;
        let script = temp_dir.join("mock_zed.sh");
        let content = format!(
            "#!/bin/sh\necho \"$@\" > \"{}\"\nexit 0\n",
            args_log.display()
        );
        std::fs::write(&script, content).unwrap();
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
        script
    };

    let exercises_dir = temp_dir.join("exercises");

    let result = launch_exercise(
        &session_file,
        &catalog,
        "motions-01-word",
        0,
        Some(&exercises_dir),
        Some(&mock_bin),
    )
    .expect("Launch failed with mock Zed binary");

    assert_eq!(result.lesson_id, "motions-01-word");
    assert_eq!(result.exercise_id, "delete-word-arg");
    assert!(result.target_arg.ends_with(":2:5"));

    // Verify session state on disk
    let session = SessionState::load_from_file(&session_file).unwrap();
    assert_eq!(session.current.lesson_id, "motions-01-word");
    assert_eq!(session.current.exercise_index, 0);
    assert_eq!(
        session.current.buffer_path,
        Some(clean_path_for_cli(&result.buffer_path))
    );

    let att = session.attempt.expect("Active attempt must be set");
    assert_eq!(att.id, result.attempt_id);
    assert_eq!(att.edit_count, 0);
    assert_eq!(att.verdict, Some(Verdict::InProgress));

    // Wait a brief moment for mock script to write log
    std::thread::sleep(std::time::Duration::from_millis(200));

    if args_log.is_file() {
        let logged = std::fs::read_to_string(&args_log).unwrap();
        assert!(
            logged.contains("-n"),
            "Expected mock runner to receive -n flag: {}",
            logged
        );
        assert!(
            logged.contains(":2:5"),
            "Expected mock runner to receive :2:5 position: {}",
            logged
        );
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}
