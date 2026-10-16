# ZK-036.1 Write Complete Install and Troubleshooting Guide Implementation Plan

**Date:** 2026-10-15  
**Requirements:** REQ-33  
**Status:** In Progress  

---

## 1. Scope & Acceptance Criteria

Per **REQ-33** and task **ZK-036.1**:
- **Acceptance:** Guide covers prerequisites, CLI, extension, external lessons, state location, optional provider setup, and common failure recovery with exact commands.
- **Files/components:** `README.md`; `docs/installation.md`.
- **Core Topics:**
  1. **Prerequisites:** Supported OS, terminal emulators, Zed editor, and Zed CLI setup.
  2. **Installation Methods:** Pre-packaged release archive distribution vs. building from source.
  3. **Extension Registration:** Installing the dev/packaged extension in Zed, verifying `.kata` association and syntax highlighting.
  4. **Lessons Layout & Discovery:** External TOML lessons, fixtures tree, zero-recompile behavior, and discovery hierarchy.
  5. **State & Write Isolation:** Standard paths for `session.json`, `exercises/`, and `lessons/generated/` across Windows, Linux, and macOS; read-only install tree compatibility.
  6. **Optional ACP Agent Configuration:** Registering custom agent in Zed settings (`settings.json`), provider credentials (`config.toml`), and local offline independence.
  7. **Failure Recovery & Troubleshooting:** Step-by-step diagnosis and exact commands for missing CLI, missing LSP on PATH, corrupt session state, stale buffers, terminal size warnings, and model timeouts.

---

## 2. Implementation Steps

1. **Plan Formulation:** Author this implementation plan.
2. **Author `docs/installation.md`:**
   - Full, end-to-end installation manual, configuration reference, and comprehensive troubleshooting guide.
3. **Update Root `README.md`:**
   - Replace pre-alpha status and outdated placeholder with actual quickstart, installation link, architecture summary, and operational instructions.
4. **Validation:**
   - Verify every command documented in `docs/installation.md` against the active codebase.
   - Run affected checks and full workspace suite (`cargo test --workspace --locked`, clippy, fmt).
5. **Update Project Documentation & Backlog:**
   - Update `docs/DEVELOPMENT.md`.
   - Update `docs/BACKLOG.md` marking ZK-036.1 complete and advancing to ZK-036.2.
