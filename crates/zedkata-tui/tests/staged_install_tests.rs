use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use zedkata_core::{catalog::Catalog, session::SessionState};
use zedkata_lsp::feedback::FeedbackEngine;
use zedkata_tui::app::App;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn staged_dist_dir() -> PathBuf {
    repo_root().join("dist").join("zedkata-v0.1.0")
}

fn exe_suffix() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn setup_temp_workspace(prefix: &str) -> (PathBuf, PathBuf) {
    let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "zedkata_{}_{}_{}_{}",
        prefix,
        std::process::id(),
        nonce,
        count
    ));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_file = temp_dir.join("session.json");
    (temp_dir, session_file)
}

fn cleanup_temp_workspace(temp_dir: &Path) {
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_staged_install_tree_and_manifest_structure() -> Result<()> {
    let dist = staged_dist_dir();
    assert!(
        dist.is_dir(),
        "Staged install directory {:?} must exist (run scripts/package.ps1 or package.sh)",
        dist
    );

    // 1. Verify directory layout
    let bin_dir = dist.join("bin");
    let lessons_dir = dist.join("lessons");
    let extension_dir = dist.join("extension");
    let manifest_path = dist.join("MANIFEST.json");
    let readme_path = dist.join("README.md");

    assert!(bin_dir.is_dir(), "Staged bin directory must exist");
    assert!(lessons_dir.is_dir(), "Staged lessons directory must exist");
    assert!(
        extension_dir.is_dir(),
        "Staged extension directory must exist"
    );
    assert!(manifest_path.is_file(), "Staged MANIFEST.json must exist");
    assert!(readme_path.is_file(), "Staged README.md must exist");

    // 2. Verify all native binaries exist in staged bin/
    let ext = exe_suffix();
    let expected_bins = [
        format!("zedkata-tui{}", ext),
        format!("zedkata-lsp{}", ext),
        format!("zedkata-agent{}", ext),
    ];
    for b in &expected_bins {
        let p = bin_dir.join(b);
        assert!(p.is_file(), "Missing expected staged binary: {:?}", p);
    }

    // 3. Verify extension metadata
    assert!(
        extension_dir.join("extension.toml").is_file(),
        "extension.toml must exist in staged extension dir"
    );
    assert!(
        extension_dir.join("README.md").is_file(),
        "README.md must exist in staged extension dir"
    );

    // 4. Verify MANIFEST.json schema and contents
    let manifest_raw = fs::read_to_string(&manifest_path)?;
    let manifest_clean = manifest_raw.trim_start_matches('\u{feff}');
    let v: serde_json::Value = serde_json::from_str(manifest_clean)?;

    assert_eq!(v["name"], "zedkata");
    assert_eq!(v["version"], "0.1.0");
    assert!(v["packaged_at"].is_string());

    let binaries = v["binaries"]
        .as_array()
        .expect("binaries array in manifest");
    assert_eq!(binaries.len(), 3);
    for bin_obj in binaries {
        assert!(bin_obj["name"].is_string());
        assert!(bin_obj["path"].is_string());
        assert!(bin_obj["sha256"].is_string());
        let sha = bin_obj["sha256"].as_str().unwrap();
        assert_eq!(sha.len(), 64, "SHA-256 hash must be 64 hex characters");
    }

    assert_eq!(v["curriculum"]["lesson_count"], 10);
    assert_eq!(v["curriculum"]["fixtures_present"], true);

    Ok(())
}

#[test]
fn test_staged_native_binaries_version_and_help() -> Result<()> {
    let dist = staged_dist_dir();
    let bin_dir = dist.join("bin");
    let ext = exe_suffix();

    let binaries = [
        format!("zedkata-tui{}", ext),
        format!("zedkata-lsp{}", ext),
        format!("zedkata-agent{}", ext),
    ];

    for bin_name in &binaries {
        let bin_path = bin_dir.join(bin_name);

        // Test --version
        let version_output = Command::new(&bin_path).arg("--version").output()?;
        assert!(
            version_output.status.success(),
            "Binary {:?} --version failed with exit status {:?}",
            bin_path,
            version_output.status
        );
        let stdout = String::from_utf8_lossy(&version_output.stdout);
        assert!(
            stdout.contains("0.1.0"),
            "Binary {:?} --version output '{}' did not contain version 0.1.0",
            bin_name,
            stdout
        );

        // Test --help
        let help_output = Command::new(&bin_path).arg("--help").output()?;
        assert!(
            help_output.status.success(),
            "Binary {:?} --help failed with exit status {:?}",
            bin_path,
            help_output.status
        );
    }

    Ok(())
}

#[test]
fn test_staged_external_lessons_catalog_validation() -> Result<()> {
    let dist = staged_dist_dir();
    let lessons_dir = dist.join("lessons");

    // Load catalog directly from the staged external lessons directory
    let catalog = Catalog::load_from_dir(&lessons_dir)
        .expect("Failed to load catalog from staged lessons directory");

    // REQ-08: Catalog must strictly validate without drafts
    catalog
        .validate_shipped_catalog()
        .expect("Staged shipped catalog must pass all validation checks");

    // Verify 10 loaded lessons
    let all = catalog.all_lessons();
    assert_eq!(all.len(), 10, "Staged catalog must contain 10 lessons");

    // Verify 5 runnable drill lessons
    let runnable = catalog.runnable_lessons();
    assert_eq!(
        runnable.len(),
        5,
        "Staged catalog must have 5 runnable drill lessons"
    );

    // Verify zero draft lessons in runnable drill curriculum
    for entry in &runnable {
        assert!(
            !entry.lesson.draft,
            "Shipped runnable drill lesson '{}' must not be marked as draft",
            entry.lesson.id
        );
    }

    let runnable_ids: Vec<&str> = runnable.iter().map(|e| e.lesson.id.as_str()).collect();
    assert!(runnable_ids.contains(&"modal-00-baseline"));
    assert!(runnable_ids.contains(&"motions-01-word"));
    assert!(runnable_ids.contains(&"motions-02-find"));
    assert!(runnable_ids.contains(&"motions-03-textobjects"));
    assert!(runnable_ids.contains(&"operators-01-edit"));

    // Verify all fixtures exist on disk in the staged directory
    for entry in &runnable {
        for ex in &entry.lesson.exercises {
            let start = catalog.resolve_fixture(&entry.lesson.id, &ex.id, true)?;
            assert!(
                start.is_file(),
                "Start fixture for {}/{} does not exist at {:?}",
                entry.lesson.id,
                ex.id,
                start
            );
            let goal = catalog.resolve_fixture(&entry.lesson.id, &ex.id, false)?;
            assert!(
                goal.is_file(),
                "Goal fixture for {}/{} does not exist at {:?}",
                entry.lesson.id,
                ex.id,
                goal
            );
        }
    }

    Ok(())
}

#[test]
fn test_changing_external_lesson_files_needs_no_recompile() -> Result<()> {
    let (temp_dir, _) = setup_temp_workspace("hot_reload_test");
    let staged_lessons = staged_dist_dir().join("lessons");
    let test_lessons = temp_dir.join("lessons");

    // Copy staged lessons to temp directory
    copy_dir_recursive(&staged_lessons, &test_lessons)?;

    // 1. Initial catalog load
    let initial_catalog = Catalog::load_from_dir(&test_lessons)?;
    let entry = initial_catalog
        .get("motions-01-word")
        .expect("motions-01-word must exist");
    let original_task = entry.lesson.exercises[0].task.clone();

    // 2. Modify lesson TOML file externally (simulating curriculum author change)
    let lesson_file = test_lessons.join("motions-01-word.toml");
    let content = fs::read_to_string(&lesson_file)?;
    let modified_content = content.replace(
        &original_task,
        "MODIFIED TASK: Jump precisely across word boundaries",
    );
    assert_ne!(content, modified_content, "Content must have changed");
    fs::write(&lesson_file, modified_content)?;

    // 3. Reload catalog without any binary re-compilation
    let reloaded_catalog = Catalog::load_from_dir(&test_lessons)?;
    let reloaded_entry = reloaded_catalog
        .get("motions-01-word")
        .expect("motions-01-word must exist");
    assert_eq!(
        reloaded_entry.lesson.exercises[0].task,
        "MODIFIED TASK: Jump precisely across word boundaries",
        "Modified external lesson file must be loaded immediately without binary recompilation"
    );

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

#[test]
fn test_optional_agent_does_not_block_local_drills() -> Result<()> {
    let (temp_dir, session_path) = setup_temp_workspace("agent_independence_test");
    let staged_lessons = staged_dist_dir().join("lessons");

    // Verify session does not exist initially
    assert!(!session_path.exists());

    // 1. Initialize local TUI App using staged lessons, completely decoupled from agent
    let mut app = App::load(session_path.clone(), staged_lessons.clone());

    // 2. Select initial exercise
    let starter_idx = app
        .lessons()
        .iter()
        .position(|l| l.lesson.id == "modal-00-baseline")
        .expect("modal-00-baseline must be present in catalog");
    app.selected_lesson_idx = starter_idx;
    app.selected_exercise_idx = 0;
    assert!(app.activate_selected().is_ok());
    assert!(
        session_path.exists(),
        "Session file should be created after activate_selected"
    );

    assert_eq!(app.session.current.lesson_id, "modal-00-baseline");
    assert_eq!(app.session.current.exercise_index, 0);

    // 3. Run feedback engine (LSP observer) using staged lessons catalog
    let catalog = Catalog::load_from_dir(&staged_lessons)?;
    let mut feedback = FeedbackEngine::new(session_path.clone(), catalog.clone());

    let ex_id = app
        .current_exercise()
        .expect("Exercise 0 must exist")
        .id
        .clone();
    let start_fixture_path = catalog.resolve_fixture("modal-00-baseline", &ex_id, true)?;
    let start_content = fs::read_to_string(&start_fixture_path)?;
    let start_uri = zedkata_lsp::feedback::path_to_uri(&start_fixture_path);

    let goal_fixture_path = catalog.resolve_fixture("modal-00-baseline", &ex_id, false)?;
    let goal_content = fs::read_to_string(&goal_fixture_path)?;

    // Open document in feedback engine (starts attempt)
    let open_notif = feedback.on_document_open(&start_uri, &start_content)?;
    assert!(open_notif.is_some());

    // Simulate student editing in Zed to match the goal content
    let change_notif = feedback.on_document_change(&start_uri, &goal_content, &[])?;
    assert!(change_notif.is_some());

    // Verify session persistence has passed attempt
    let updated_session = SessionState::load_from_file(&session_path)?;
    assert_eq!(updated_session.history.len(), 1);
    assert!(updated_session.history[0].passed);
    assert_eq!(updated_session.history[0].lesson_id, "modal-00-baseline");
    assert_eq!(updated_session.history[0].exercise_id, ex_id);

    cleanup_temp_workspace(&temp_dir);
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
