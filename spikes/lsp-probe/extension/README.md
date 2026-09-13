# Zedkata probe dev extension

This is disposable ZK-003 code, built with Rust 1.97.1 and pinned
`zed_extension_api` 0.7.0. It is separate from the planned native Rust workspace;
it does not establish Rust 1.75 compatibility.

The language is named **Zedkata Probe**, associates `.kata` files, and references
Zed's Rust grammar for the Rust-shaped connection fixture. Grammar reuse and API
compatibility with installed Zed 1.19.2 still require the runtime check below.
No grammar download or wasi-sdk installation was needed for the Cargo build.

## Build from the repository root

```powershell
npm.cmd --prefix spikes/lsp-probe run build
rustup target add wasm32-wasip2
cargo build --manifest-path spikes/lsp-probe/extension/Cargo.toml --target wasm32-wasip2 --release --locked
cargo fmt --manifest-path spikes/lsp-probe/extension/Cargo.toml --check
cargo clippy --manifest-path spikes/lsp-probe/extension/Cargo.toml --target wasm32-wasip2 --release --locked -- -D warnings
```

Cargo produces `target/wasm32-wasip2/release/zedkata_probe_extension.wasm` inside
this extension directory. Installing a dev extension lets Zed perform its own build
and package the extension. The checked-in Cargo lockfile pins this spike's dependencies.

## Install and smoke test in Zed

1. Open `C:\Users\Dan\zedkata` as the project root (`zed .` from the repo).
2. Open the command palette with Ctrl+Shift+P. Run `zed: install dev extension`.
3. Choose `C:\Users\Dan\zedkata\spikes\lsp-probe\extension` and wait for installation.
4. Open [connection.kata](../fixtures/connection.kata). Confirm the language indicator
   says **Zedkata Probe**. If the fixture was already open, close and reopen it.
5. Look for a `zedkata-probe` diagnostic beginning with `Probe opened` at the start
   of the file. Use Zed's diagnostics view if the inline indication is subtle.
6. Change `hello probe` to `hello Zed` in the buffer. Look for `Probe change` feedback.
7. Close the fixture tab, discarding this smoke-test edit if prompted.
8. Report whether installation worked, whether the diagnostic was visible, and any
   error text. The capture should contain initialize, didOpen, didChange, didClose,
   and outgoing diagnostics. Codex can inspect those files after the interaction.

The extension locates Node using the worktree PATH and invokes `../launch.cjs` through
its absolute repository path. Open the repository root, not just the fixture's folder.
No personal Zed settings or hardcoded Node paths are required by the extension.

Each server launch normally writes `spikes/lsp-probe/captures/zed-*.jsonl`. An inherited
`ZEDKATA_PROBE_LOG` overrides that path; remove a stale override before launching Zed.
Captures contain complete document text. Synthetic test captures remain under
`test-output/`, separate from editor captures.

## Troubleshooting

- Installation/build failure: use `zed: open log` and report the error. A successful
  Cargo build alone does not prove Zed can load the component or find the grammar.
- Node not found: ensure Node 22+ is on PATH, then restart Zed so it sees that PATH.
- Probe not built: run the npm build command above and restart the language server.
- Wrong project root: reopen the repository folder; the extension checks for
  `spikes/lsp-probe/package.json` before starting.
- Capture-open failure: use a fresh filename or remove an inherited log override.
- Missing diagnostics: first inspect the capture for outgoing publishDiagnostics;
  sending and visible rendering are separate observations.

Remove **Zedkata Probe** from Zed's extensions UI when the spike is retired; retain
selected evidence before deleting spike code.

Sources checked 2026-09-13:
[local dev installation](https://zed.dev/docs/extensions/developing-extensions),
[server registration](https://zed.dev/docs/extensions/languages#language-servers),
and [worktree path/PATH API](https://docs.rs/zed_extension_api/0.7.0/zed_extension_api/struct.Worktree.html).

## Recorded status

2026-09-13: WASM target, release build, clippy and real Zed connection checks passed.
Dan confirmed the language and visible diagnostic. Captured open/change/close and
diagnostic clearing; see the [connection report](../../../docs/evidence/2026-09-13-connection.md).
Technique comparisons and render-latency measurements remain pending.
