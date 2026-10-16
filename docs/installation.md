# Zedkata Installation & Troubleshooting Guide

This guide provides end-to-end instructions for installing, configuring, verifying, and troubleshooting **Zedkata** across supported environments.

---

## 1. System Prerequisites

Before installing Zedkata, verify your environment meets the following requirements:

| Component | Minimum Requirement | Recommended | Verification Command |
|---|---|---|---|
| **Operating System** | Windows 10/11 x64, Linux x64 (glibc >= 2.31), macOS 13+ | Windows 11 (24H2) / Ubuntu 24.04 / macOS 14+ | `[System.Environment]::OSVersion` or `uname -a` |
| **Zed Editor** | Zed >= 0.140.0 | Zed 1.19+ | `zed --version` |
| **Zed CLI** | Installed on system PATH | Active CLI integration | `where.exe zed` / `Get-Command zed` (Windows) or `which zed` (Unix) |
| **Terminal** | Minimum 60x15 cols (warns if < 80x24) | Windows Terminal, Alacritty, iTerm2, WezTerm | `stty size` or `$Host.UI.RawUI.WindowSize` |
| **Rust Toolchain** | Rust 1.85.0 (only if compiling from source) | Rust stable (`1.85+`) | `rustc --version` |

> [!IMPORTANT]
> **Zed CLI is required:** Zedkata launches exercise buffers directly in Zed using the editor's command-line interface. If `zed` is not found in your terminal:
> 1. Open Zed.
> 2. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) to open the Command Palette.
> 3. Search for and execute **`zed: install cli`**.
> 4. Restart your terminal and verify with `zed --version`.

---

## 2. Installing Zedkata

### Option A: Install from Pre-Packaged Release Archive (Recommended)

Release distributions are self-contained archives containing the three native binaries (`zedkata-tui`, `zedkata-lsp`, `zedkata-agent`), the external curriculum lessons (`lessons/`), and the precompiled extension (`extension/`):

1. **Download and Extract:**
   - **Windows:** Download `zedkata-v<version>-x86_64-pc-windows-msvc.zip`. Extract to your preferred location, e.g.:
     ```powershell
     Expand-Archive -Path zedkata-v0.1.0-x86_64-pc-windows-msvc.zip -DestinationPath "$env:LOCALAPPDATA\Programs\zedkata"
     ```
   - **Linux / macOS:** Download `zedkata-v<version>-<target>.tar.gz`. Extract to `~/.local/opt/zedkata`:
     ```bash
     mkdir -p ~/.local/opt/zedkata
     tar -xzf zedkata-v0.1.0-x86_64-unknown-linux-gnu.tar.gz -C ~/.local/opt/zedkata --strip-components=1
     ```

2. **Add Binaries to System PATH:**
   - **Windows (PowerShell):**
     ```powershell
     $InstallBin = "$env:LOCALAPPDATA\Programs\zedkata\bin"
     [Environment]::SetEnvironmentVariable("Path", $env:Path + ";$InstallBin", [EnvironmentVariableTarget]::User)
     $env:Path += ";$InstallBin"
     ```
   - **Linux / macOS (Bash / Zsh):**
     ```bash
     echo 'export PATH="$HOME/.local/opt/zedkata/bin:$PATH"' >> ~/.bashrc
     export PATH="$HOME/.local/opt/zedkata/bin:$PATH"
     ```

3. **Verify Binary Installation:**
   ```bash
   zedkata-tui --version
   zedkata-lsp --version
   zedkata-agent --version
   ```
   All three commands should report `0.1.0`.

---

### Option B: Build and Install from Source Checkout

If you are developing or compiling from source:

1. **Clone and Build Locked Release Binaries:**
   ```powershell
   git clone https://github.com/trustdan/zedkata.git
   cd zedkata
   cargo build --workspace --release --locked
   ```

2. **Package Distribution:**
   - On Windows:
     ```powershell
     powershell -ExecutionPolicy Bypass -File scripts/package.ps1 -Archive
     ```
   - On Linux / macOS:
     ```bash
     bash scripts/package.sh --archive
     ```
   This creates a fully staged release layout in `dist/zedkata-v0.1.0/`.

3. **Install Binaries to Cargo Bin:**
   ```powershell
   cargo install --path crates/zedkata-tui --locked
   cargo install --path crates/zedkata-lsp --locked
   cargo install --path crates/zedkata-agent --locked
   ```

---

## 3. Installing the Zed Extension

Zedkata includes a lightweight WebAssembly extension that registers the **Zedkata** language for `.kata` drill files, attaches syntax highlighting, and hooks the `zedkata-lsp` feedback engine.

1. **Install Dev Extension in Zed:**
   - Open Zed.
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) to open the Command Palette.
   - Type and select **`zed: install dev extension`**.
   - In the file dialog, navigate to and select the `extension/` directory inside your installed Zedkata folder (or repository checkout).
   - Zed will load the extension manifest (`extension.toml`) and precompiled WebAssembly component (`extension.wasm`).

2. **Verify Language Association:**
   - In Zed, open any `.kata` file (e.g. from `lessons/fixtures/motions-01/delete-word-arg.start.kata`).
   - Check the status bar at the bottom right of the Zed window: the active language should display **Zedkata**.
   - Hovering over code elements or making edits will connect to `zedkata-lsp` and provide inline feedback diagnostics.

---

## 4. Curriculum Lessons & Discovery Layout

Lessons are external data files (TOML files and companion fixture files) and never require re-compiling code:

```text
<install_root>/
├── bin/
│   ├── zedkata-tui.exe (or zedkata-tui)
│   ├── zedkata-lsp.exe (or zedkata-lsp)
│   └── zedkata-agent.exe (or zedkata-agent)
├── lessons/
│   ├── modal-00-baseline.toml
│   ├── motions-01-word.toml
│   ├── motions-02-find.toml
│   ├── motions-03-textobjects.toml
│   ├── operators-01-edit.toml
│   ├── zed-01-lsp-actions.toml
│   ├── zed-02-project-panel.toml
│   ├── zed-03-multibuffer.toml
│   ├── zed-04-tasks-and-debugger.toml
│   ├── zed-05-agent-panel.toml
│   └── fixtures/
├── extension/
│   ├── extension.toml
│   └── extension.wasm
├── MANIFEST.json
└── README.md
```

### Lesson Discovery Order:
When `zedkata-tui`, `zedkata-lsp`, or `zedkata-agent` runs, lessons are resolved in the following priority order:
1. Explicit CLI argument: `--lessons-dir <PATH>`
2. Environment variable: `ZEDKATA_LESSONS_DIR=<PATH>`
3. Relative installed layout: `<exe_dir>/../lessons` (automatically discovers lessons when binaries reside in `<install>/bin/`)
4. Platform standard data directory:
   - Windows: `%LOCALAPPDATA%\zedkata\lessons`
   - Linux: `$XDG_DATA_HOME/zedkata/lessons` (fallback `~/.local/share/zedkata/lessons`)
   - macOS: `~/Library/Application Support/zedkata/lessons`
5. Development repository root: `./lessons`

---

## 5. State Paths & Write Isolation

Zedkata guarantees strict write isolation: the installation directory can be marked completely read-only. All runtime state, progress tracking, and exercise files are created strictly in user-writable paths:

| State Item | Description | Default Windows Path | Default Linux Path | Default macOS Path |
|---|---|---|---|---|
| **Session State** | Current exercise, history, and weakness tracking | `%LOCALAPPDATA%\zedkata\session.json` | `$XDG_STATE_HOME/zedkata/session.json` | `~/Library/Application Support/zedkata/session.json` |
| **Exercise Buffers** | Active writable `.kata` files for Zed editing | `%LOCALAPPDATA%\zedkata\exercises\` | `$XDG_STATE_HOME/zedkata/exercises\` | `~/Library/Application Support/zedkata/exercises\` |
| **Generated Practice** | AI-generated custom drills targeting weak spots | `%LOCALAPPDATA%\zedkata\lessons\generated\` | `$XDG_DATA_HOME/zedkata/lessons/generated\` | `~/Library/Application Support/zedkata/lessons/generated\` |

**Custom Overrides:**
- `--session <PATH>`: Explicit path for session state JSON file.
- `ZEDKATA_SESSION_PATH` (or `ZEDKATA_SESSION`): Environment variable for session state path.
- `ZEDKATA_EXERCISES_DIR`: Environment variable for exercise buffer directory.
- `ZEDKATA_DATA_DIR`: Environment variable for user data parent directory (`.../zedkata`).

---

## 6. Optional ACP Agent & Model Configuration

> [!NOTE]
> The core local drill loop (TUI, Zed launch, and LSP feedback) is **100% offline** and requires zero API keys or external models.

To enable conversational features (attempt explanations, weakness analysis, and custom drill generation) through Zed's Agent Panel:

1. **Register Custom Agent in Zed:**
   Open your Zed user settings (`settings.json` via `Ctrl+,` or `Cmd+,` or `~/.config/zed/settings.json`):
   ```json
   {
     "assistant": {
       "version": "2",
       "custom_agents": [
         {
           "name": "zedkata",
           "command": "zedkata-agent",
           "args": ["acp"]
         }
       ]
     }
   }
   ```

2. **Configure Model Provider in `config.toml`:**
   Create or edit the configuration file:
   - Windows: `%APPDATA%\zedkata\config.toml`
   - Linux / macOS: `~/.config/zedkata/config.toml` (or `$XDG_CONFIG_HOME/zedkata/config.toml`)

   Example configuration:
   ```toml
   [provider]
   name = "anthropic" # Options: "anthropic", "openai", "ollama", "fake"
   model = "claude-3-5-sonnet-20241022"
   api_key = "sk-ant-..." # Or set the ANTHROPIC_API_KEY environment variable

   [agent]
   max_context_chars = 4000
   timeout_ms = 30000
   ```

3. **Connect in Zed:**
   Press `Ctrl+?` (or `Cmd+?` on macOS) to open the Agent Panel. In the agent selector dropdown, choose **zedkata**.

---

## 7. First Passing Exercise Walkthrough

To verify your installation from clean setup to first completed drill:

1. **Start the TUI:**
   Open your terminal and run:
   ```bash
   zedkata-tui
   ```
   Navigate to **Baseline Vim Emulation (`modal-00-baseline`)** and select Exercise 1 (**`insert-visibility`**). Press `Enter` or type `:open`.

2. **Practice in Zed:**
   Zed opens the writable exercise buffer:
   `<workspace>/exercises/modal-00-baseline/insert-visibility.kata`
   (or `%LOCALAPPDATA%\zedkata\exercises\modal-00-baseline\insert-visibility.kata`)
   with the cursor automatically positioned at line 2.

3. **Execute the Motion:**
   The task asks you to make the `timeout_ms` field public.
   - In normal mode, navigate to the start of `timeout_ms`.
   - Press `i` to enter insert mode, type `pub `, and press `Esc` to return to normal mode.
   - Save the buffer (`:w` or `Ctrl+S`).

4. **Observe Instant Feedback:**
   - Look at the diagnostics in Zed: a green pass indicator confirms success.
   - Look at `zedkata-tui`: the exercise badge turns to `[PASS]`, your score (100) is recorded, and the next drill unlocks.
   - In the TUI, press `n` to advance to the next exercise.

---

## 8. Common Failure Recovery & Troubleshooting

### Problem 1: `Command not found: zed`
- **Symptom:** Selecting an exercise in the TUI fails with: `Failed to launch Zed CLI: The system cannot find the file specified.`
- **Cause:** Zed CLI is not installed or not present on your system `PATH`.
- **Exact Recovery:**
  1. Open Zed.
  2. Press `Ctrl+Shift+P` (or `Cmd+Shift+P`).
  3. Execute `zed: install cli`.
  4. Test in a new terminal: `zed --version`.
  5. *Alternative:* Provide a custom runner executable to the TUI:
     ```bash
     zedkata-tui --runner "C:\Users\<User>\AppData\Local\Programs\Zed\Zed.exe"
     ```

### Problem 2: Missing Language Server Binary on PATH
- **Symptom:** Zed displays an error when opening a `.kata` file:
  `Zedkata LSP server binary ('zedkata-lsp.exe') was not found on PATH.`
- **Cause:** Zed's extension host searches system PATH for `zedkata-lsp`, but the binary's directory is not in your environment.
- **Exact Recovery:**
  - On Windows:
    ```powershell
    $env:Path += ";C:\path\to\zedkata\bin"
    [Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)
    ```
  - On Linux / macOS:
    ```bash
    export PATH="/path/to/zedkata/bin:$PATH"
    ```
  - Restart Zed so it inherits the updated PATH.

### Problem 3: Corrupt or Incompatible `session.json`
- **Symptom:** Starting `zedkata-tui` shows a warning banner:
  `Warning: Corrupted session file detected. Backup saved at session.json.corrupt.<timestamp>`
- **Cause:** Power interruption, manual file editing, or schema version migration.
- **Exact Recovery:**
  Zedkata automatically protects your data:
  - It creates a timestamped backup of the damaged file (`session.json.corrupt.<timestamp>`).
  - It initializes a clean session with zero progress data loss to other files.
  - To inspect or restore historical attempts from the backup:
    ```powershell
    Get-Content "$env:LOCALAPPDATA\zedkata\session.json.corrupt.*" | ConvertFrom-Json
    ```

### Problem 4: Stale Buffer / File Missing on Disk
- **Symptom:** The TUI status reports `Buffer missing on disk` or `Dirty buffer mismatch`.
- **Cause:** An exercise buffer was deleted or edited outside the course session.
- **Exact Recovery:**
  - Press `r` in `zedkata-tui` to reset the active exercise.
  - The working buffer is immediately re-copied from the clean baseline fixture (`*.start.kata`), diagnostics are cleared, and a fresh attempt is issued.

### Problem 5: Terminal Window Too Small
- **Symptom:** TUI shows a prominent warning:
  `TERMINAL TOO SMALL: Current (WxH) is below recommended 80x24`
- **Cause:** Terminal emulator window is below the minimum required boundary.
- **Exact Recovery:**
  - Resize your terminal window or press `F11` for fullscreen.
  - Once the window reaches at least 80 columns by 24 rows (minimum boundary 60x15), the warning banner disappears instantly without restarting the application.

### Problem 6: ACP Agent Model Timeout or Invalid Key
- **Symptom:** Agent Panel displays: `Explanation failed: Provider error: Unauthorized` or `Request timed out`.
- **Cause:** Invalid or missing API key in `config.toml` or environment variable.
- **Exact Recovery:**
  1. Check your configuration file (`%APPDATA%\zedkata\config.toml` or `~/.config/zedkata/config.toml`).
  2. Verify that your API key is valid, or set the environment variable:
     ```powershell
     $env:ANTHROPIC_API_KEY = "sk-ant-..."
     ```
  3. For offline testing without network access, set the provider to `fake` in `config.toml`:
     ```toml
     [provider]
     name = "fake"
     ```
  4. Notice that local drills in `zedkata-tui` and inline LSP diagnostics continue functioning 100% offline regardless of agent status.

---

## 9. Updating Zedkata

Because Zedkata enforces strict separation between application code and user data (see Section 5), updating Zedkata never endangers your drill history, weaknesses, or customized settings.

### 9.1 Updating Application Binaries

To update to a new release version:

1. **Download and Extract Over Existing Installation:**
   - **Windows (PowerShell):**
     ```powershell
     Expand-Archive -Path zedkata-v<new_version>-x86_64-pc-windows-msvc.zip -DestinationPath "$env:LOCALAPPDATA\Programs\zedkata" -Force
     ```
   - **Linux / macOS (Bash / Zsh):**
     ```bash
     tar -xzf zedkata-v<new_version>-<target>.tar.gz -C ~/.local/opt/zedkata --strip-components=1
     ```

2. **Verify Updated Versions:**
   ```bash
   zedkata-tui --version
   zedkata-lsp --version
   zedkata-agent --version
   ```

### 9.2 Updating External Curriculum Lessons

Curriculum lessons are external data files (`lessons/*.toml` and `lessons/fixtures/`).
- **Zero Recompile:** When an update adds new lessons or refines existing exercises, the new TOML and fixture files are immediately recognized by `zedkata-tui`, `zedkata-lsp`, and `zedkata-agent` upon the next launch without compiling code.
- **Progress Preservation:** Your completion records for existing lessons are matched by immutable lesson ID (`id = "modal-00-baseline"`) and remain intact in `session.json`.

### 9.3 Schema Versioning & Downgrade / Incompatibility Recovery

Zedkata session state uses a versioned schema (`CURRENT_SCHEMA_VERSION = 1`).
- **Forward Compatibility:** If you update to a new version of Zedkata that supports additional session fields, existing `session.json` files from prior versions are safely loaded and forward-migrated.
- **Incompatible or Newer Schema Detection:** If a session file contains a schema version newer than what the running binary supports (e.g. `version > 1` from a downgraded binary) or contains malformed JSON:
  1. Zedkata automatically renames the incompatible file to:
     ```text
     session.json.corrupt.<timestamp_nanos>.bak
     ```
  2. Zedkata initializes a clean session with the supported schema version and displays a recovery warning banner.
  3. **Zero Data Destruction:** Your previous progress is never overwritten or deleted. You can restore or inspect historical attempts from the backup:
     ```powershell
     Get-Content "$env:LOCALAPPDATA\zedkata\session.json.corrupt.*" | ConvertFrom-Json
     ```

---

## 10. Uninstalling & Data Retention

Zedkata cleanly separates the application distribution from student data. You can completely remove the application while retaining your completed drill records, or execute an intentional full purge.

### 10.1 Application Removal (Preserving Student Progress)

Use these commands to remove Zedkata binaries and extension while keeping your progress and history intact for future reinstallation:

1. **Remove Application Binaries & Staged Lessons:**
   - **Windows (PowerShell):**
     ```powershell
     Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Programs\zedkata"
     ```
   - **Linux / macOS:**
     ```bash
     rm -rf ~/.local/opt/zedkata
     ```

2. **Remove PATH Entry (Optional):**
   - **Windows (PowerShell):**
     ```powershell
     $UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
     $CleanedPath = ($UserPath -split ';' | Where-Object { $_ -and $_ -notlike "*zedkata*" }) -join ';'
     [Environment]::SetEnvironmentVariable("Path", $CleanedPath, [EnvironmentVariableTarget]::User)
     ```
   - **Linux / macOS:**
     Remove the `export PATH="$HOME/.local/opt/zedkata/bin:$PATH"` line from your `~/.bashrc` or `~/.zshrc`.

3. **Uninstall Extension in Zed:**
   - Open Zed.
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) -> open **`zed: extensions`**.
   - Locate **Zedkata** under Dev Extensions and select **Uninstall** (or delete the dev extension link).

*(Your progress in `session.json` and working buffers in `exercises/` remain safely stored on disk.)*

### 10.2 Intentional Full Data Purge (Reset All Progress)

If and only if you intend to permanently delete all course progress, attempt history, weaknesses, custom drills, and agent credentials:

- **Windows (PowerShell):**
  ```powershell
  # Remove student session, exercise buffers, and generated drills
  Remove-Item -Recurse -Force "$env:LOCALAPPDATA\zedkata"
  # Remove agent configuration and credentials (if present)
  Remove-Item -Recurse -Force "$env:APPDATA\zedkata"
  ```

- **Linux (Bash):**
  ```bash
  rm -rf "${XDG_STATE_HOME:-$HOME/.local/state}/zedkata"
  rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/zedkata"
  rm -rf "${XDG_CONFIG_HOME:-$HOME/.config}/zedkata"
  ```

- **macOS (Bash / Zsh):**
  ```bash
  rm -rf ~/Library/Application\ Support/zedkata
  rm -rf ~/.config/zedkata
  ```
