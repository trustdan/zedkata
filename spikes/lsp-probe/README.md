# Phase 0 LSP probe (RETIRED)

> **Status: Retired (2026-09-13).**  
> The Phase 0 Feasibility Gate has been passed (see [ADR 0003](../../docs/adr/0003-phase-0-feasibility-gate.md)).  
> Real editor captures are permanently retained under `tests/fixtures/lsp/` for production integration testing.  
> This disposable probe code is preserved for historical reference. Production components will be implemented in Rust under `crates/`.

## Install and check

From the repository root, in PowerShell (Node 22 or newer):

```powershell
Set-Location spikes/lsp-probe
npm.cmd ci --ignore-scripts --no-audit --no-fund --cache ./npm-cache
npm.cmd test
```

If the environment forces npm offline and packages are not cached, dependency
installation needs network access with `--offline=false`. No global installs are
required. The lockfile pins the dependency tree.

`npm.cmd test` compiles the probe and runs two checks: a framed stdio lifecycle with
Unicode/CRLF, multi-change and full-replacement payloads, diagnostics, close, and
shutdown; and failure to open the capture file. Tests write ignored synthetic captures
under `test-output/`. These files are **not evidence of Zed behavior**.

## Start and stop

Build and launch directly:

```powershell
npm.cmd run build
$env:ZEDKATA_PROBE_LOG = Join-Path (Get-Location) 'captures/manual-01.jsonl'
node dist/probe.js --stdio
```

It waits for LSP frames on stdin, so typing plain text is not a useful smoke test.
Use `npm.cmd test` for an automated client. Stop a foreground launch with Ctrl+C;
an LSP client should request `shutdown`, await its response, then notify `exit`.
After a foreground run, remove the optional override with:

```powershell
Remove-Item Env:ZEDKATA_PROBE_LOG -ErrorAction SilentlyContinue
```

For Zed, launch `node` directly with the absolute `dist/probe.js` path and `--stdio`
as separate arguments. Do not use `npm start` as the LSP command: npm prints script
banners to stdout. `npm.cmd start` is only a foreground convenience command.
The [dev extension and real-editor checklist](extension/README.md) are prepared for
ZK-003. Its `launch.cjs` wrapper uses an absolute script path and captures directory.
The synthetic lifecycle test also checks that wrapper from a different working directory.

Without an override, captures go to `captures/probe-<timestamp>-<pid>.jsonl` relative
to the launch working directory. Set an absolute `ZEDKATA_PROBE_LOG` for editor runs.
Each launch requires a new filename: existing files are rejected, preserving earlier
captures. Startup reports the path on stderr; stdout contains protocol frames only.
Capture creation/write failures terminate the probe rather than silently losing data.

## Capture format and limits

Each JSONL entry contains `sequence`, UTC `at`, monotonic `monotonicNs`, `direction`
(`meta`, `in`, or `out`), and `message`. The first entry records process metadata;
incoming and outgoing records retain the entire parsed JSON-RPC object, including
initialization capabilities and every document notification. JSON whitespace and
transport headers are not retained; payload values and change-array order are.

The server advertises incremental synchronization and UTF-16 positions. Inspect the
actual incoming messages to establish what the client sends. It records full-document
replacements too and does not merge, normalize, or infer edits. Each open/change emits
an informational diagnostic at the start of the document; close clears diagnostics.
The change counter is process-wide and counts notifications, not keystrokes or edits.

Timestamps are taken after message parsing and before outbound writes. Synchronous
capture writes add overhead, and outbound timestamps do not mean a diagnostic is
already rendered. Monotonic time is useful for intervals within a run; UTC labels
support correlation. ZK-004 must measure visible latency separately and report the
measurement method and instrumentation overhead. Captures include full document text.

Protocol reference: Microsoft's
[LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
and [Node server library](https://github.com/microsoft/vscode-languageserver-node),
checked 2026-09-13. This probe has only been tested with its synthetic client.
