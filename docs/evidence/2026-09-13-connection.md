# ZK-003 — Real Zed connection accepted

Date: 2026-09-13. Zed: 1.19.2, commit
`df181c6f58d02677b385fa947d6bfde6d3530078`. Windows; Node 22.21.1.

Installed `spikes/lsp-probe/extension` as a dev extension with the repository root
open in Zed. The extension used Node from `C:\nvm4w\nodejs\node.exe` to run
`spikes/lsp-probe/launch.cjs --stdio`. Its committed manifest and language config
record the registration; extension API is pinned to 0.7.0.

[Complete connection snapshot](../../tests/fixtures/lsp/2026-09-13-connection.jsonl):
29 JSONL records from `zed-1789309314240-16252.jsonl`.

| Observation | Evidence |
|-------------|----------|
| Extension installation and launch | Zed.log compiled the dev extension and started Node at 09:21:54 local time |
| Correct language | Dan confirmed Zedkata Probe in the language indicator |
| Initialization and open | Capture sequences 2 and 6 |
| Real buffer edits | Ten incremental didChange notifications, sequences 8–26 |
| Outgoing feedback | Each change followed by publishDiagnostics; last message was Probe change 10 |
| Visible feedback | Dan confirmed seeing the diagnostic after hovering at the beginning of the first line |
| Document close | Sequence 28, 2026-09-13T14:29:59.353Z, connection.kata |
| Diagnostic clearing | Sequence 29 publishes an empty diagnostics array |

ZK-003.2 and ZK-003.3 pass, completing ZK-003. The earlier
[partial report](2026-09-13-connection-partial.md) remains a historical snapshot.

This is one successful connection smoke test, not a calibrated technique trial.
Visible rendering was confirmed by the user; render latency was not measured.
OQ-01 granularity across techniques, OQ-03 latency and OQ-11 undo still require the
[observation matrix](phase-0-matrix.md). Production remains gated by ZK-005.
