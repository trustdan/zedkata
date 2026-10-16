use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR is crates/zedkata-core, parent of parent is repo root
    manifest_dir
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn test_workspace_and_extension_versions_reconciled() {
    let root = repo_root();

    // 1. Root Cargo.toml
    let root_cargo =
        fs::read_to_string(root.join("Cargo.toml")).expect("Failed to read root Cargo.toml");
    let root_toml: toml::Value =
        toml::from_str(&root_cargo).expect("Failed to parse root Cargo.toml");
    let root_version = root_toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .expect("Missing [workspace.package].version in root Cargo.toml");

    // 2. Extension Cargo.toml
    let ext_cargo = fs::read_to_string(root.join("extension").join("Cargo.toml"))
        .expect("Failed to read extension/Cargo.toml");
    let ext_cargo_toml: toml::Value =
        toml::from_str(&ext_cargo).expect("Failed to parse extension/Cargo.toml");
    let ext_cargo_version = ext_cargo_toml
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .expect("Missing [package].version in extension/Cargo.toml");

    // 3. Extension extension.toml
    let ext_meta = fs::read_to_string(root.join("extension").join("extension.toml"))
        .expect("Failed to read extension/extension.toml");
    let ext_meta_toml: toml::Value =
        toml::from_str(&ext_meta).expect("Failed to parse extension/extension.toml");
    let ext_meta_version = ext_meta_toml
        .get("version")
        .and_then(|v| v.as_str())
        .expect("Missing version in extension/extension.toml");

    assert_eq!(
        root_version, "0.1.0",
        "Root workspace version must be 0.1.0"
    );
    assert_eq!(
        ext_cargo_version, root_version,
        "extension/Cargo.toml version must match workspace version"
    );
    assert_eq!(
        ext_meta_version, root_version,
        "extension/extension.toml version must match workspace version"
    );
}

#[test]
fn test_explicit_license_files_present_and_reconciled() {
    let root = repo_root();

    // 1. LICENSE files exist
    let license_path = root.join("LICENSE");
    let license_mit_path = root.join("LICENSE-MIT");
    let license_apache_path = root.join("LICENSE-APACHE");

    assert!(license_path.exists(), "Root LICENSE file must exist");
    assert!(
        license_mit_path.exists(),
        "Root LICENSE-MIT file must exist"
    );
    assert!(
        license_apache_path.exists(),
        "Root LICENSE-APACHE file must exist"
    );

    let license_content = fs::read_to_string(&license_path).expect("Read LICENSE");
    let mit_content = fs::read_to_string(&license_mit_path).expect("Read LICENSE-MIT");
    let apache_content = fs::read_to_string(&license_apache_path).expect("Read LICENSE-APACHE");

    assert!(
        license_content.contains("MIT") && license_content.contains("Apache"),
        "LICENSE must declare dual MIT / Apache-2.0 licensing"
    );
    assert!(
        mit_content.contains("MIT License") && mit_content.contains("Zedkata Contributors"),
        "LICENSE-MIT must contain MIT terms and copyright notice"
    );
    assert!(
        apache_content.contains("Apache License") && apache_content.contains("Version 2.0"),
        "LICENSE-APACHE must contain Apache 2.0 terms"
    );

    // 2. Cargo declarations match
    let root_cargo = fs::read_to_string(root.join("Cargo.toml")).expect("Read root Cargo.toml");
    assert!(
        root_cargo.contains(r#"license = "MIT OR Apache-2.0""#),
        "Root Cargo.toml must declare MIT OR Apache-2.0 license"
    );

    let ext_cargo = fs::read_to_string(root.join("extension").join("Cargo.toml"))
        .expect("Read extension/Cargo.toml");
    assert!(
        ext_cargo.contains(r#"license = "MIT OR Apache-2.0""#),
        "extension/Cargo.toml must declare MIT OR Apache-2.0 license"
    );
}

#[test]
fn test_release_notes_present_and_document_known_limits() {
    let root = repo_root();
    let release_notes_path = root.join("RELEASE_NOTES.md");
    assert!(
        release_notes_path.exists(),
        "RELEASE_NOTES.md must exist in repo root"
    );

    let notes = fs::read_to_string(&release_notes_path).expect("Read RELEASE_NOTES.md");

    // Version
    assert!(notes.contains("0.1.0"), "Release notes must mention v0.1.0");

    // Disciplined platform support claims (REQ-04, REQ-32)
    assert!(
        notes.contains("x86_64-pc-windows-msvc") && notes.contains("VERIFIED"),
        "Release notes must record Windows 11 as VERIFIED"
    );
    assert!(
        notes.contains("x86_64-unknown-linux-gnu") && notes.contains("UNVERIFIED"),
        "Release notes must record Linux as UNVERIFIED"
    );
    assert!(
        notes.contains("apple-darwin") && notes.contains("UNVERIFIED"),
        "Release notes must record macOS as UNVERIFIED"
    );

    // Observability boundaries documented
    assert!(
        notes.contains("didChange"),
        "Release notes must explain LSP didChange technique inference"
    );
    assert!(
        notes.contains("walkthrough"),
        "Release notes must document walkthrough self-reported completion"
    );
}

#[test]
fn test_packaging_scripts_stage_licenses_and_release_notes() {
    let root = repo_root();

    let ps1 = fs::read_to_string(root.join("scripts").join("package.ps1"))
        .expect("Read scripts/package.ps1");
    let sh = fs::read_to_string(root.join("scripts").join("package.sh"))
        .expect("Read scripts/package.sh");

    for file in &[
        "LICENSE",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "RELEASE_NOTES.md",
    ] {
        assert!(
            ps1.contains(file),
            "scripts/package.ps1 must stage {}",
            file
        );
        assert!(sh.contains(file), "scripts/package.sh must stage {}", file);
    }
}

#[test]
fn test_extension_submission_artifacts_and_metadata_compliance() {
    let root = repo_root();

    // 1. Extension subdirectory licenses
    let ext_dir = root.join("extension");
    assert!(
        ext_dir.join("LICENSE-MIT").exists(),
        "extension/LICENSE-MIT must exist for monorepo submission compliance"
    );
    assert!(
        ext_dir.join("LICENSE-APACHE").exists(),
        "extension/LICENSE-APACHE must exist for monorepo submission compliance"
    );

    // 2. Extension manifest metadata
    let ext_meta_str =
        fs::read_to_string(ext_dir.join("extension.toml")).expect("Read extension.toml");
    let ext_meta: toml::Value = toml::from_str(&ext_meta_str).expect("Parse extension.toml");
    assert_eq!(ext_meta.get("id").and_then(|v| v.as_str()), Some("zedkata"));
    assert_eq!(
        ext_meta.get("schema_version").and_then(|v| v.as_integer()),
        Some(1)
    );
    assert_eq!(
        ext_meta.get("repository").and_then(|v| v.as_str()),
        Some("https://github.com/trustdan/zedkata")
    );

    // 3. Pinned API in extension/Cargo.toml
    let ext_cargo_str =
        fs::read_to_string(ext_dir.join("Cargo.toml")).expect("Read extension/Cargo.toml");
    assert!(
        ext_cargo_str.contains(r#"zed_extension_api = "=0.7.0""#),
        "extension/Cargo.toml must pin zed_extension_api to =0.7.0"
    );
    assert!(
        ext_cargo_str.contains(r#"crate-type = ["cdylib"]"#),
        "extension/Cargo.toml must declare cdylib crate-type"
    );

    // 4. Language config
    let lang_cfg_str = fs::read_to_string(
        ext_dir
            .join("languages")
            .join("zedkata")
            .join("config.toml"),
    )
    .expect("Read language config.toml");
    assert!(lang_cfg_str.contains(r#"name = "Zedkata""#));
    assert!(lang_cfg_str.contains(r#"path_suffixes = ["kata"]"#));

    // 5. Submission artifacts exist
    let sub_dir = root.join("docs").join("submission");
    assert!(
        sub_dir.join("PR_DRAFT.md").exists(),
        "docs/submission/PR_DRAFT.md must exist"
    );
    assert!(
        sub_dir.join("extensions.toml.patch").exists(),
        "docs/submission/extensions.toml.patch must exist"
    );
    assert!(
        sub_dir.join("SUBMISSION_CHECKLIST.md").exists(),
        "docs/submission/SUBMISSION_CHECKLIST.md must exist"
    );

    // 6. Registry patch content
    let patch_str = fs::read_to_string(sub_dir.join("extensions.toml.patch")).expect("Read patch");
    assert!(patch_str.contains("[zedkata]"));
    assert!(patch_str.contains(r#"submodule = "extensions/zedkata""#));
    assert!(patch_str.contains(r#"path = "extension""#));
    assert!(patch_str.contains(r#"version = "0.1.0""#));
}
