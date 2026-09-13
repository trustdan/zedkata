# Development setup

Checked 2026-09-13 on Dan's Windows machine. Start with the
[backlog](BACKLOG.md); Phase 0 still gates production code.

## Observed tools

| Tool | Observed version or state |
|------|---------------------------|
| Zed | 1.19.2, commit `df181c6f58d02677b385fa947d6bfde6d3530078` |
| Rust compiler | 1.97.1 (`8bab26f4f`, 2026-07-14) |
| Cargo | 1.97.1 (`c980f4866`, 2026-06-30) |
| Active toolchain | `stable-x86_64-pc-windows-msvc` |
| Installed Rust targets | `x86_64-pc-windows-msvc` only |
| Node.js | 22.21.1 |
| npm | 10.9.4 |
| pnpm | 11.20.0; optional, not selected for the probe |
| Git | 2.55.0.windows.3 |
| Visual Studio | 2022 Community; `vswhere` reports the x86/x64 C++ tools component |

Zed resolves to `%LOCALAPPDATA%\Programs\Zed\bin\zed.exe`, Rust tools to
`%USERPROFILE%\.cargo\bin`, and Node/npm to `C:\nvm4w\nodejs` on this machine.
Use PATH discovery rather than embedding these paths in shipped code.

## Recheck the setup

Run from the repository root in PowerShell:

```powershell
Get-Command zed,rustc,cargo,rustup,node,npm.cmd,git
zed --version
rustc --version
cargo --version
rustup show active-toolchain
rustup toolchain list
rustup target list --installed
node --version
npm.cmd --version
git --version
```

Use `npm.cmd` to invoke npm explicitly from PowerShell. The probe will use npm and
a committed lockfile; package selection and installation belong to ZK-002.

## Remaining prerequisites

- `wasm32-wasip2` was installed during ZK-003 on 2026-09-13 with
  `rustup target add wasm32-wasip2`; the disposable extension release build passed.
- A grammar-bearing extension may need wasi-sdk; Zed documents automatic download.
  Grammar choice and the actual extension build remain for ZK-003.
- Rust 1.85 is verified as the native MSRV (checked 2026-09-13). Rust 1.75 is
  incompatible because modern 2026 dependencies use `edition2024` (stabilized in 1.85).
  Native workspace compilation and unit tests pass on both 1.85.0 and 1.97.1.
- Native CI workflow (`.github/workflows/ci.yml`) runs formatting, clippy, tests, and
  MSRV checks on Windows.

Target and SDK behavior comes from the current, unversioned
[extension development guide](https://zed.dev/docs/extensions/developing-extensions),
checked 2026-09-13; validate it against installed Zed during ZK-003.

## Planned probe connection

Use the local dev extension, with the following files now prepared in ZK-003 under
`spikes/lsp-probe/extension/`:

```text
extension.toml
Cargo.toml
src/lib.rs
languages/zedkata/config.toml
```

Define the language name and `.kata` suffix in language metadata; register a server
for that same language name in the manifest. Implement `language_server_command`
to launch Node with the probe's script path and stdio argument as separate arguments.
The spike pins extension API 0.7.0 and references the Rust grammar; actual loading
in Zed remains pending. See the [exact install/checklist](../spikes/lsp-probe/extension/README.md).
The registration mechanism is documented in
[Language Extensions](https://zed.dev/docs/extensions/languages#language-servers).

Once those files and the probe exist:

1. Open the repository with `zed .`.
2. Run `zed: install dev extension` in the command palette and select the extension
   directory above. Publishing is unnecessary for this development route.
3. Open the probe's `.kata` fixture and verify its selected language.
4. Confirm the capture contains initialization and document-open messages, edit the
   file, and check that a change arrives and the probe diagnostic renders.
5. Use `zed: open log` to investigate registration, build, or startup failures.

These are preparation instructions, **not completed runtime checks**. ZK-002 creates
the probe; ZK-003 validates the connection; ZK-004 measures actual traffic.

The settings documentation covers file associations and binary overrides for known
servers; it directs new language support to extensions. It does not establish a
settings-only registration route for our new language/server pair.
[Configuring Languages](https://zed.dev/docs/configuring-languages)

## Task handoff

ZK-001 and ZK-002 are complete: inventory and the documented connection route are
recorded, and the [probe](../spikes/lsp-probe/README.md) builds and passes its synthetic
client checks. Dependencies are pinned in its npm lockfile.
ZK-003 is complete: [real connection evidence](evidence/2026-09-13-connection.md)
includes open/change/close and user-confirmed visible feedback. Next is the
[Phase 0 matrix](evidence/phase-0-matrix.md). OQ-01 general granularity and OQ-03
latency remain unmeasured; earlier pending-check notes above describe preparation.
