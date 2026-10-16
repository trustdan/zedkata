# Zedkata Extension Submission Checklist & Guide

**Target Registry:** [zed-industries/extensions](https://github.com/zed-industries/extensions)  
**Extension ID:** `zedkata`  
**Extension Version:** `0.1.0`  
**License:** MIT OR Apache-2.0  
**PR Draft:** [PR_DRAFT.md](PR_DRAFT.md)  
**Registry Patch:** [extensions.toml.patch](extensions.toml.patch)  

> [!IMPORTANT]
> External publication and PR submission must NOT be executed autonomously.
> This document provides the reviewable steps, commands, and verification criteria for when the user explicitly authorizes publication.

---

## 1. Pre-Submission Metadata & Architecture Audit

| Requirement | Zed Registry Guideline | Zedkata Implementation | Status |
|---|---|---|---|
| **Unique ID** | Kebab-case, alphanumeric, descriptive | `id = "zedkata"` | **COMPLIANT** |
| **Manifest Format** | `extension.toml` with `schema_version = 1` | `extension/extension.toml` defines `id`, `name`, `version`, `schema_version = 1`, `authors`, `description`, `repository`, and `language_servers` | **COMPLIANT** |
| **API Pinned Version** | Exact pinned `zed_extension_api` version | `zed_extension_api = "=0.7.0"` in `extension/Cargo.toml` | **COMPLIANT** |
| **Open Source License** | Accepted OSI license at repo or extension root | `LICENSE-MIT` and `LICENSE-APACHE` present at both repo root and `extension/` directory | **COMPLIANT** |
| **No Heavy Binaries in WASM** | Do not bundle native binaries directly into WASM | Extension binary is ~217 KB; dynamically discovers `zedkata-lsp` on PATH via `worktree.which` | **COMPLIANT** |
| **Language Metadata** | `languages/<lang>/config.toml` present | `extension/languages/zedkata/config.toml` configures grammar (`rust`), extension (`.kata`), and comments | **COMPLIANT** |
| **English Language** | All user-facing text in English | Extension documentation, logs, and error messages in English | **COMPLIANT** |

---

## 2. Local Validation Verification

Before submitting, ensure all local validation commands pass:

```powershell
# 1. Verify extension unit tests
cargo test --manifest-path extension/Cargo.toml

# 2. Verify WASM Clippy linter
cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings

# 3. Verify formatting
cargo fmt --manifest-path extension/Cargo.toml -- --check

# 4. Compile locked release WASM artifact
cargo build --manifest-path extension/Cargo.toml --target wasm32-wasip2 --release --locked
```

---

## 3. Pull Request Submission Execution Steps (User-Authorized)

When authorized to submit the extension to the registry:

### Step 1: Fork and Clone `zed-industries/extensions`
```bash
git clone https://github.com/<your-github-username>/extensions.git zed-extensions
cd zed-extensions
git checkout -b add-zedkata-extension
```

### Step 2: Add Submodule via HTTPS
Add the public Zedkata repository as a submodule under `extensions/zedkata`:
```bash
git submodule add https://github.com/trustdan/zedkata.git extensions/zedkata
```

### Step 3: Register in `extensions.toml`
Add the following entry to the root `extensions.toml` (or apply `docs/submission/extensions.toml.patch`):
```toml
[zedkata]
submodule = "extensions/zedkata"
path = "extension"
version = "0.1.0"
```

### Step 4: Sort and Format (if pnpm is installed)
```bash
pnpm install
pnpm sort-extensions
```

### Step 5: Commit and Push
```bash
git add .gitmodules extensions/zedkata extensions.toml
git commit -m "Add zedkata extension (v0.1.0)"
git push -u origin add-zedkata-extension
```

### Step 6: Open Pull Request
- Navigate to `https://github.com/zed-industries/extensions/compare`.
- Title: `Add Zedkata extension (v0.1.0)`.
- Description: Copy contents from [docs/submission/PR_DRAFT.md](PR_DRAFT.md).
