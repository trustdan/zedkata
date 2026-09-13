# Phase 0 matrix results — 2026-09-13

Status: in progress; one case observed. Follow the
[matrix](phase-0-matrix.md). Zed 1.19.2 on Windows; same probe process and setup as
the [connection check](2026-09-13-connection.md). Editor settings such as autosave,
format-on-save and edit prediction have not been fully recorded for this run.
No visible-render timing measurement is available.

## 01-dw — first run

User acknowledged performing the requested `dw` on the `a` of `alpha`, then closing
and discarding changes. The raw capture confirms the edit and document close.

[Raw snapshot through sequence 37](../../tests/fixtures/lsp/2026-09-13-matrix-01-dw.jsonl)
retains initialization and earlier connection traffic, followed by this case.
Original capture: `zed-1789309314240-16252.jsonl`.
Document URI: `file:///C:/Users/Dan/zedkata/spikes/lsp-probe/fixtures/matrix/01-dw.kata`.

| Sequence | Event | Observation |
|----------|-------|-------------|
| 30 | didOpen, version 0 | Exact matrix baseline, LF newlines |
| 32 | didChange, version 1 | One contentChanges entry deleting range (1,17)–(1,23), empty insertion |
| 33 | publishDiagnostics | Probe change 11 sent; process-wide counter includes earlier smoke test |
| 34 | didChange, version 2 | Inserts `alpha ` at (1,17) before closing |
| 36 | didClose | Document closed |
| 37 | publishDiagnostics | Empty diagnostics array |

Positions are zero-based UTF-16, as advertised during initialization. Comparing the
range to the baseline confirms deletion of exactly `alpha `; the resulting words
value is `bravo charlie`. The measured action therefore produced **one incremental
notification containing one deletion**, not six single-character deletions.

The restoration at sequence 34 occurred about 9.89 seconds later, immediately before
close. It is consistent with the requested discard flow; it is not evidence that an
undo command was performed. The document lifecycle has two change notifications in
total, but only sequence 32 is assigned to the requested `dw` action. Final disk
content remains the baseline.

Sequence 32 was received at 14:33:58.619Z. Its capture timestamp precedes the outgoing
diagnostic record by 79.1 microseconds. This is instrumented server processing time,
not end-to-end or visible-render latency and does not satisfy OQ-03.

Interpretation: this single sample is compatible with useful incremental observation.
Repeatability and discrimination against alternative techniques remain untested.
Next: 02-dw-repeat. The Phase 0 gate remains open.
