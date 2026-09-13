# First real probe connection

Observed 2026-09-13 with Zed 1.19.2. User confirmed the language indicator reads
Zedkata Probe and reported changing `hello probe` to `hello Zed` and saving with `:w`.

[Raw snapshot](../../tests/fixtures/lsp/2026-09-13-connection-partial.jsonl) retains
27 JSONL records from `zed-1789309314240-16252.jsonl`. It is a partial snapshot of
the live capture, not a completed matrix run.

- Zed.log records successful dev-extension compilation and Node startup through
  `spikes/lsp-probe/launch.cjs` at 09:21:54 local time.
- Sequence 2 is initialize; sequence 6 is didOpen for `connection.kata`.
- Sequences 8–26 contain ten didChange notifications: deletion of `probe`, insertion
  of lowercase letters, subsequent single-character deletions and insertion of `Zed`.
- Each change in this sample has one incremental range. Each was followed by an
  outgoing publishDiagnostics notification, ending with `Probe change 10`.
- No didClose is present in this snapshot. Visible diagnostic rendering has not
  been confirmed, and no render latency has been measured.

This demonstrates real editor-to-probe traffic and outbound feedback for this edit.
It does not establish general granularity, repeatability, technique distinguishability,
undo behavior or the Phase 0 gate. ZK-003 remains pending close/visibility checks.

The open file's directory listing reported zero bytes while reading it returned
records; direct content inspection was used rather than treating cached size metadata
as evidence of an empty capture.
