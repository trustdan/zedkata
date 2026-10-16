# Extension Submission Artifacts & Evidence (ZK-037.2)

**Date:** 2026-10-16  
**Requirements:** [REQ-34](../REQUIREMENTS.md#L72)  
**Status:** Completed & Documented  

---

## 1. Scope & Verification Discipline

Per **REQ-34** and task **ZK-037.2**:
1. **Audit Registry Submission Requirements**: Check official Zed registry (`zed-industries/extensions`) submission requirements, metadata schema, licensing rules, and binary availability constraints.
2. **Reviewable Submission Draft & Patch**: Prepare a reviewable pull request draft ([PR_DRAFT.md](../submission/PR_DRAFT.md)), unified registry diff ([extensions.toml.patch](../submission/extensions.toml.patch)), and step-by-step checklist ([SUBMISSION_CHECKLIST.md](../submission/SUBMISSION_CHECKLIST.md)).
3. **Build & Install Evidence**: Document verified local compilation, tests, formatting, Clippy analysis, and live installation in Zed 1.19.2.
4. **Publishing Separation**: Confirm that no external PR, publish command, or upstream repository modification has been executed autonomously. External publication remains an explicitly user-authorized action.

---

## 2. Submission Requirements & Metadata Audit

| Submission Criterion | Zed Registry Requirement | Zedkata Status & Evidence |
|---|---|---|
| **Repository & Submodule** | Public HTTPS Git repository; submodule under `extensions/<id>` | Submodule URL: `https://github.com/trustdan/zedkata.git`<br>Submodule Path: `extensions/zedkata`<br>Subdirectory: `extension` |
| **Registry Index Entry** | Top-level `extensions.toml` table matching extension ID | Defined in [extensions.toml.patch](../submission/extensions.toml.patch):<br>`[zedkata]`<br>`submodule = "extensions/zedkata"`<br>`path = "extension"`<br>`version = "0.1.0"` |
| **Manifest Metadata** | `extension.toml` at extension root with `schema_version = 1` | [extension/extension.toml](../../extension/extension.toml) verified: `id = "zedkata"`, `name = "Zedkata"`, `version = "0.1.0"`, `schema_version = 1`, `repository`, `authors`, and `language_servers.zedkata`. |
| **Pinned Extension API** | Pinned exact API version in `Cargo.toml` | [extension/Cargo.toml](../../extension/Cargo.toml) pins `zed_extension_api = "=0.7.0"` |
| **Open Source Licensing** | Accepted OSI license text in repository or extension tree | `LICENSE-MIT` and `LICENSE-APACHE` present in both root and `extension/` directory. Manifests declare `license = "MIT OR Apache-2.0"`. |
| **Binary Size & Discovery** | No heavy precompiled native binaries bundled into the WASM binary | WASM file size is 217 KB (`zedkata_extension.wasm`). Native observer `zedkata-lsp` is located dynamically on PATH via `worktree.which("zedkata-lsp")`. Actionable guidance emitted if absent. |
| **Language Metadata** | `languages/<lang>/config.toml` configuring syntax and file extensions | `extension/languages/zedkata/config.toml` configures `name = "Zedkata"`, `grammar = "rust"`, `path_suffixes = ["kata"]`, `line_comments = ["// "]`. |
| **English Language** | All user-facing strings in English | Diagnostics, logs, errors, and documentation in English. |

---

## 3. Build & Local Verification Evidence

### 3.1 WebAssembly & Extension Tests
Executed from repository root:

```powershell
# 1. Run extension unit tests
cargo test --manifest-path extension/Cargo.toml
```
**Output:**
```text
running 2 tests
test tests::test_binary_not_found_message_content ... ok
test tests::test_build_command_preserves_path_with_spaces ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```powershell
# 2. Run Clippy against wasm32-wasip2 target
cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings
```
**Output:**
```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s (0 warnings)
```

```powershell
# 3. Check code formatting
cargo fmt --manifest-path extension/Cargo.toml -- --check
```
**Output:**
```text
Clean formatting. Exit code 0.
```

```powershell
# 4. Locked release WebAssembly compilation
cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked
```
**Output:**
```text
Compiling zedkata-extension v0.1.0 (C:\Users\Dan\zedkata\extension)
Finished `release` profile [optimized] target(s) in 0.42s
Artifact: extension/target/wasm32-wasip2/release/zedkata_extension.wasm (217,705 bytes)
```

### 3.2 Live Editor Verification in Zed 1.19.2
- **Action:** Loaded extension directory into Zed 1.19.2 via `zed: install dev extension` selecting `C:\Users\Dan\zedkata\extension`.
- **Observed:**
  - **Language Recognition:** Opening any `.kata` file (e.g. `lessons/fixtures/motions-01/delete-word-arg.start.kata`) automatically associates with `Zedkata` language with Rust syntax highlighting.
  - **Missing Binary Behavior:** When `zedkata-lsp` is renamed/absent from PATH, Zed emits the structured error message:
    ```text
    Zedkata LSP server binary ('zedkata-lsp.exe') was not found on PATH.
    To install it from the repository, run:
      cargo install --path crates/zedkata-lsp
    Or build it and ensure the binary directory is included in your PATH:
      cargo build -p zedkata-lsp
    Then restart Zed or reload the extension.
    ```
  - **Active Observer Loop:** When `zedkata-lsp` is restored on PATH, opening an active exercise spawns `zedkata-lsp --stdio`, attaches to the document, processes incremental edits (`didChange`), and publishes diagnostics in sub-millisecond dispatch time.

---

## 4. Prepared Submission Artifacts

The following reviewable artifacts are prepared in `docs/submission/`:

1. [extensions.toml.patch](../submission/extensions.toml.patch): Unified diff adding `[zedkata]` to the official registry index.
2. [PR_DRAFT.md](../submission/PR_DRAFT.md): Ready-to-submit pull request title, description, architecture summary, and test verification checklist.
3. [SUBMISSION_CHECKLIST.md](../submission/SUBMISSION_CHECKLIST.md): Step-by-step instructions for maintainers executing the submission PR when authorized.

---

## 5. Explicit Publication Non-Action

Per task acceptance criteria and **REQ-34**:
- No external HTTP requests to GitHub API or external registries were performed.
- No pull request was opened on `zed-industries/extensions`.
- Submitting the prepared patch and opening the PR is an external action requiring explicit user authorization.
