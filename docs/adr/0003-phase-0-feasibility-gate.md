# ADR 0003 — Phase 0 Feasibility Gate Decision

**Status:** Accepted  
**Date:** 2026-09-13  

## Context

Because Zed provides no API for inspecting the cursor, selection, or keystrokes, `zedkata`'s technique verification relies on inferring user actions from Language Server Protocol (`textDocument/didChange`) notifications.

The feasibility of this entire project depended on empirical verification of OQ-01: does Zed send granular, incremental edit notifications per operation, or does it debounce and coalesce edits into full-buffer replacements? If edits were coalesced, technique scoring would be impossible and the project would have to fall back to simple goal-matching.

## Decision

Accept **Finding 1** from the backlog gate criteria:
1. **Granular incremental deltas verified:** In real Zed 1.19.2 on Windows, operator deletions (`dw`) produce a single incremental notification spanning the entire word `(1,17)–(1,23)` (Cases 01 & 02). In contrast, typing 5 backspaces produces 5 distinct single-character notifications arriving ~180 ms apart without coalescing (Case 03). Technique discrimination on the wire is confirmed.
2. **Fast feedback verified:** Probe processing time from `didChange` receipt to `publishDiagnostics` dispatch is sub-millisecond (79 µs to 1.04 ms), easily meeting the <100 ms target and the 300 ms feedback-channel gate. Visible diagnostic rendering in Zed was confirmed.
3. **Vim mode documentation accepted:** Detailed vim mode commands, motions, text objects, and visual selections are verified against official Zed documentation (`zed.dev/docs/vim.md`) rather than requiring manual execution of all 20 matrix fixtures.

**The Phase 0 Feasibility Gate is PASSED.**
Production work is unblocked. We proceed with evidence-supported scoring rules (`par_edits`, `forbid`, and goal matching). The disposable TypeScript probe is retired, while retained raw captures in `tests/fixtures/lsp/` become golden integration test fixtures for the production Rust language server.

## Consequences

- Phase 1 scaffolding commences immediately for the native Rust workspace (`zedkata-core`, `zedkata-lsp`, `zedkata-tui`).
- Technique scoring rules and edit budgets are supported by wire evidence.
- The disposable probe in `spikes/lsp-probe` is decommissioned.
