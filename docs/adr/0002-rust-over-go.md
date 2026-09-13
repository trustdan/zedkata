# ADR 0002 — Rust over Go

**Status:** Accepted
**Date:** 2026-09-13

## Context

Both Rust and Go were live options. Performance is irrelevant at this scale — the
question is integration surface.

Three of the four components touch an ecosystem with an official Rust path and no
official Go one:

- **ACP** ships official SDKs for Rust, TypeScript, Python, Kotlin, and Java. No Go SDK.
  In Go the agent would mean hand-rolling JSON-RPC against the published schema and
  re-chasing it at every protocol revision.
- **Zed extensions** must be Rust compiled to WebAssembly. Go closes that door entirely,
  which means the LSP could never be packaged for anyone else.
- **The TUI** reuses existing ratatui + crossterm patterns from a sibling project,
  along with the `clap` / `thiserror` / `anyhow` conventions.

Go's real advantages were acknowledged: the LSP half is protocol-level and
language-agnostic (`go.lsp.dev/protocol` would be fine), single static binaries
cross-compile with far less friction, and iteration is faster.

## Decision

Rust for all shipped components.

TypeScript is explicitly permitted for throwaway spikes — the Phase 0 `didChange` probe
should be written with `vscode-languageserver` because it is an hour of work instead of
an evening. Spikes are deleted after their finding is recorded; they are never promoted
into the codebase.

## Consequences

- Tokio enters the dependency tree. The LSP and the ACP agent are both genuinely
  concurrent request/notification workloads and the mature libraries assume it. A
  no-Tokio rule from a sibling project does not transfer here and should not be applied
  by pattern-matching. `zedkata-core` stays sync and runtime-agnostic regardless.
- Slower iteration than Go, particularly on the extension rebuild loop.
- Cross-compilation needs deliberate setup in Phase 5 rather than being free.
- If the project ever scoped down to TUI + LSP only, with no agent and no published
  extension, this decision should be revisited rather than inherited.
