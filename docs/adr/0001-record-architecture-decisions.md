# ADR 0001 — Record architecture decisions

**Status:** Accepted
**Date:** 2026-09-13

## Context

This project depends on undocumented behavior in a fast-moving editor. Decisions made
today will look arbitrary in six months, and the reasoning — especially "we chose this
because Zed can't do X" — is the part most likely to be lost.

## Decision

Architecture decisions are recorded as numbered files in `docs/adr/`.

One decision per file. Short — if it runs past a page, it is probably two decisions.

Format: Context, Decision, Consequences. Status is Proposed, Accepted, or Superseded
by ADR NNNN. Never delete a superseded ADR; the history is the point.

Any decision that constrains future work belongs here. Implementation details that a
reader could recover from the code do not.

## Consequences

Slight overhead per decision. In exchange, when Zed changes and a constraint evaporates,
we can find every decision that was made because of it.
