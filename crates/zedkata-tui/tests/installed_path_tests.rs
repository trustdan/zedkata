use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use zedkata_core::{catalog::Catalog, session::SessionState};
use zedkata_lsp::feedback::FeedbackEngine;
use zedkata_tui::launcher::{launch_exercise, resolve_exercises_dir};

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

struct InstalledEnv {
    base_dir: PathBuf,
    install_dir: PathBuf,
    workspace_dir: PathBuf,
    session_path: PathBuf,
}

impl InstalledEnv {
    fn new(prefix: &str) -> Result<Self> {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let nonce = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let base_dir = std::env::temp_dir().join(format!(
            "zedkata_inst_{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nonce,
            count
        ));

        // Intentionally create paths containing spaces to verify quoting and whitespace handling
        let install_dir = base_dir.join("Zedkata Distribution Tools 2026");
        let workspace_dir = base_dir.join("My Student Workspace 2026");
        let session_path = workspace_dir.join("user state").join("session.json");

        fs::create_dir_all(&install_dir)?;
        fs::create_dir_all(&workspace_dir)?;
        if let Some(parent) = session_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy staged release into install directory
        let dist = staged_dist_dir();
        assert!(
            dist.is_dir(),
            "Staged directory {:?} must exist before running installed path tests",
            dist
        );
        copy_dir_recursive(&dist, &install_dir)?;

        Ok(Self {
            base_dir,
            install_dir,
            workspace_dir,
            session_path,
        })
    }

    fn tui_exe(&self) -> PathBuf {
        self.install_dir
            .join("bin")
            .join(format!("zedkata-tui{}", exe_suffix()))
    }

    fn lsp_exe(&self) -> PathBuf {
        self.install_dir
            .join("bin")
            .join(format!("zedkata-lsp{}", exe_suffix()))
    }

    fn agent_exe(&self) -> PathBuf {
        self.install_dir
            .join("bin")
            .join(format!("zedkata-agent{}", exe_suffix()))
    }

    fn lessons_dir(&self) -> PathBuf {
        self.install_dir.join("lessons")
    }

    fn make_install_dir_read_only(&self) -> Result<()> {
        set_dir_readonly(&self.install_dir, true)
    }

    fn mock_zed_bin(&self) -> PathBuf {
        let script = self.workspace_dir.join(if cfg!(windows) {
            "mock_zed.bat"
        } else {
            "mock_zed.sh"
        });
        let content = if cfg!(windows) {
            "@echo off\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nexit 0\n"
        };
        let _ = fs::write(&script, content);
        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&script) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&script, perms);
            }
        }
        script
    }

    fn inventory_install_dir(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        collect_files(&self.install_dir, &mut files)?;
        files.sort();
        Ok(files)
    }
}

impl Drop for InstalledEnv {
    fn drop(&mut self) {
        // Restore write permissions so recursive removal succeeds
        let _ = set_dir_readonly(&self.install_dir, false);
        let _ = fs::remove_dir_all(&self.base_dir);
    }
}

#[allow(clippy::permissions_set_readonly_false)]
fn set_dir_readonly(dir: &Path, readonly: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        if !readonly {
            let mut dir_perms = fs::metadata(dir)?.permissions();
            dir_perms.set_readonly(false);
            let _ = fs::set_permissions(dir, dir_perms);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                set_dir_readonly(&path, readonly)?;
            }
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_readonly(readonly);
            let _ = fs::set_permissions(&path, perms);
        }
        if readonly {
            let mut dir_perms = fs::metadata(dir)?.permissions();
            dir_perms.set_readonly(readonly);
            let _ = fs::set_permissions(dir, dir_perms);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        if !readonly {
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o755));
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                set_dir_readonly(&path, readonly)?;
            }
            let mode = if readonly {
                if path.is_dir() {
                    0o555
                } else {
                    0o444
                }
            } else if path.is_dir() {
                0o755
            } else {
                0o644
            };
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(mode));
        }
        if readonly {
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o555));
        }
    }
    Ok(())
}

fn collect_files(dir: &Path, list: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        list.push(path.clone());
        if path.is_dir() {
            collect_files(&path, list)?;
        }
    }
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

#[test]
fn test_installed_tree_binary_discovery_from_external_cwd_with_spaces() -> Result<()> {
    let env = InstalledEnv::new("discovery")?;
    let tui = env.tui_exe();
    let lsp = env.lsp_exe();
    assert!(tui.is_file(), "zedkata-tui must exist in installed bin dir");
    assert!(lsp.is_file(), "zedkata-lsp must exist in installed bin dir");

    // Verify zedkata-lsp reports version when invoked from external cwd
    let lsp_ver = Command::new(&lsp)
        .current_dir(&env.workspace_dir)
        .arg("--version")
        .output()?;
    assert!(lsp_ver.status.success());
    assert!(String::from_utf8_lossy(&lsp_ver.stdout).contains("0.1.0"));

    // Execute 'select modal-00-baseline 0' with working directory set to external workspace
    let status = Command::new(&tui)
        .current_dir(&env.workspace_dir)
        .arg("select")
        .arg("modal-00-baseline")
        .arg("0")
        .arg("--session")
        .arg(&env.session_path)
        .status()?;

    assert!(
        status.success(),
        "zedkata-tui select must succeed from external cwd"
    );

    // Verify session state was created in writable workspace
    assert!(
        env.session_path.is_file(),
        "Session file must be written in the designated workspace path"
    );
    let session = SessionState::load_from_file(&env.session_path)?;
    assert_eq!(session.current.lesson_id, "modal-00-baseline");
    assert_eq!(session.current.exercise_index, 0);

    Ok(())
}

#[test]
fn test_read_only_install_tree_integrity() -> Result<()> {
    let env = InstalledEnv::new("readonly")?;
    let tui = env.tui_exe();
    let initial_inventory = env.inventory_install_dir()?;

    // Mark install tree strictly read-only
    env.make_install_dir_read_only()?;

    // 1. Run 'select' from external working directory
    let select_status = Command::new(&tui)
        .current_dir(&env.workspace_dir)
        .arg("select")
        .arg("motions-01-word")
        .arg("0")
        .arg("--session")
        .arg(&env.session_path)
        .status()?;
    assert!(
        select_status.success(),
        "zedkata-tui select must succeed with read-only install tree"
    );

    // 2. Run 'status' from external working directory
    let status_out = Command::new(&tui)
        .current_dir(&env.workspace_dir)
        .arg("status")
        .arg("--session")
        .arg(&env.session_path)
        .output()?;
    assert!(
        status_out.status.success(),
        "zedkata-tui status must succeed with read-only install tree"
    );
    let stdout = String::from_utf8_lossy(&status_out.stdout);
    assert!(stdout.contains("motions-01-word"));

    // 3. Verify zero files added or modified in the read-only install tree
    let current_inventory = env.inventory_install_dir()?;
    assert_eq!(
        initial_inventory, current_inventory,
        "No files or directories should be created or deleted inside the read-only install tree"
    );

    Ok(())
}

#[test]
fn test_exercise_buffer_written_to_agreed_writable_path_with_spaces() -> Result<()> {
    let env = InstalledEnv::new("buffer_write")?;
    env.make_install_dir_read_only()?;

    let catalog = Catalog::load_from_dir(&env.lessons_dir())?;
    let exercises_dir = resolve_exercises_dir(&env.session_path);

    // Verify resolved exercises dir is inside the writable workspace, not install dir
    assert!(
        exercises_dir.starts_with(&env.workspace_dir),
        "Exercises directory must be located inside the writable workspace"
    );

    let mock_bin = env.mock_zed_bin();
    // Launch exercise with a mock command runner (no GUI process spawned)
    let launch_res = launch_exercise(
        &env.session_path,
        &catalog,
        "modal-00-baseline",
        0,
        Some(&exercises_dir),
        Some(&mock_bin),
    )?;

    // Verify exercise buffer exists on disk in writable exercises directory
    assert!(
        launch_res.buffer_path.is_file(),
        "Exercise buffer must exist at {:?}",
        launch_res.buffer_path
    );
    assert!(
        launch_res.buffer_path.starts_with(&exercises_dir),
        "Exercise buffer must reside inside writable exercises dir"
    );

    // Verify target argument contains formatted position
    assert!(
        launch_res.target_arg.contains(":2:5"),
        "Target argument must contain line:col coordinates"
    );

    // Verify session.json recorded active buffer path
    let session = SessionState::load_from_file(&env.session_path)?;
    assert_eq!(
        session.current.buffer_path,
        Some(zedkata_tui::launcher::clean_path_for_cli(
            &launch_res.buffer_path
        ))
    );

    Ok(())
}

#[test]
fn test_lsp_feedback_engine_with_read_only_install_and_external_paths() -> Result<()> {
    let env = InstalledEnv::new("lsp_feedback")?;
    env.make_install_dir_read_only()?;

    let catalog = Catalog::load_from_dir(&env.lessons_dir())?;
    let mut feedback = FeedbackEngine::new(env.session_path.clone(), catalog.clone());

    let mock_bin = env.mock_zed_bin();
    let exercises_dir = resolve_exercises_dir(&env.session_path);
    // Select exercise first to prime session state
    let _ = launch_exercise(
        &env.session_path,
        &catalog,
        "modal-00-baseline",
        0,
        Some(&exercises_dir),
        Some(&mock_bin),
    )?;

    let session = SessionState::load_from_file(&env.session_path)?;
    let buffer_path_str = session
        .current
        .buffer_path
        .expect("Buffer path must be present in session");
    let buffer_path = PathBuf::from(buffer_path_str);
    let buffer_uri = zedkata_lsp::feedback::path_to_uri(&buffer_path);

    // Read start content from buffer
    let start_text = fs::read_to_string(&buffer_path)?;
    let open_notif = feedback.on_document_open(&buffer_uri, &start_text)?;
    assert!(open_notif.is_some());

    // Read goal content from read-only catalog fixture
    let goal_fixture_path =
        catalog.resolve_fixture("modal-00-baseline", "insert-visibility", false)?;
    let goal_text = fs::read_to_string(&goal_fixture_path)?;

    // Simulate student editing buffer to match goal text
    let change_notif = feedback.on_document_change(&buffer_uri, &goal_text, &[])?;
    assert!(change_notif.is_some());

    // Verify session updated with passing verdict
    let updated_session = SessionState::load_from_file(&env.session_path)?;
    assert_eq!(updated_session.history.len(), 1);
    assert!(updated_session.history[0].passed);
    assert_eq!(updated_session.history[0].lesson_id, "modal-00-baseline");
    assert_eq!(updated_session.history[0].exercise_id, "insert-visibility");

    Ok(())
}

#[test]
fn test_agent_cli_from_external_cwd_with_spaces() -> Result<()> {
    let env = InstalledEnv::new("agent_cli")?;
    let agent = env.agent_exe();
    assert!(
        agent.is_file(),
        "zedkata-agent must exist in installed bin dir"
    );

    // Run 'zedkata-agent walkthrough list --session "<session_path>"'
    let output = Command::new(&agent)
        .current_dir(&env.workspace_dir)
        .arg("walkthrough")
        .arg("list")
        .arg("--session")
        .arg(&env.session_path)
        .output()?;

    assert!(
        output.status.success(),
        "zedkata-agent walkthrough list must succeed from external cwd"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Zedkata Guided Walkthroughs"));
    assert!(stdout.contains("mini-vs-any"));
    assert!(stdout.contains("lsp-actions-walkthrough"));
    assert!(stdout.contains("agent-panel-walkthrough"));

    Ok(())
}
