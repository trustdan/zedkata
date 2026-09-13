# Build plan

Status: in use for step-by-step execution. The full backlog expansion was completed
2026-09-13: 35 requirement groups map to 95 executable leaf tasks, including the two
completed setup/probe tasks. The 38 parent/standalone IDs retain the original roadmap
plus optional inline hints. Production implementation still awaits the Phase 0 gate.

## First useful release

Run a TUI in Zed's terminal, choose an exercise, edit a real `.kata` buffer, see
feedback, retry, and resume progress after restarting. Ship three calibrated
exercises first. Technique scoring depends on the feasibility findings; goal matching
is the fallback. Add the ACP tutor after this drill loop works.

## Working together

- [The backlog](../BACKLOG.md) owns task status and ordering.
- [Requirements and coverage](../REQUIREMENTS.md) links documented scope to task IDs.
  Read the detailed execution cards for dependencies and acceptance checks. A parent
  task completes only when all required children complete or have a justified skip.
- Start with “Let's do ZK-001” or “Pick the next ready task.” Work on one task at a
  time. Split anything that grows beyond a focused coding session.
- Before coding, restate scope, dependencies, and the completion check. For changes
  spanning more than two files, record a short implementation plan under `docs/plans/`
  and agree it according to the repository working rules.
- Codex implements and runs automated checks. Dan performs real-editor input when
  needed; Codex supplies exact steps and records the observations.
- Leave tasks unchecked while manual checks are pending. Each completed task should
  be a reviewable, commit-sized change; commit when requested.
- End with changes, validation, unresolved issues, and the next task ID.
- Update architecture and open questions when evidence changes the design.

## Milestones

| Milestone | Tasks | Demonstration |
|-----------|-------|---------------|
| Feasibility decision | ZK-001–005 and their detailed children | Captured edits and a recorded scoring/feedback decision |
| Rust foundation | ZK-006–012 | Load a lesson, verify a buffer, persist a session |
| Real-editor loop | ZK-013–016 | One exercise passes and fails in Zed |
| Local MVP | ZK-017–023 | Complete three exercises, retry, restart, resume |
| Curriculum | ZK-024–028 | Verified and calibrated lessons |
| Conversational tutor | ZK-029–033 | Explain an attempt and offer validated practice |
| Distribution | ZK-034–037 | Install and run from a clean supported machine |
| Optional follow-up | ZK-038.1 | Inline task hints, only if selected |

Current position: ZK-001–003 are complete, including real editor connection and visible
feedback. ZK-004.1 has completed
the [observation matrix](../evidence/phase-0-matrix.md) and 20 baseline fixtures.
ZK-004.2 and ZK-004.3 are ready for manual observations. Production work is blocked on ZK-005. Parent
numbers group work rather than impose a total order; ACP can begin after the local MVP
while unrelated curriculum work continues. See explicit dependencies in the backlog.

## Intended layout

Create these paths when their task needs them, after the Phase 0 gate:

```text
Cargo.toml                   # native Rust workspace
Cargo.lock
crates/
  zedkata-core/              # lesson model, verification, state helpers; sync
  zedkata-lsp/               # stdio observer and diagnostics
  zedkata-tui/               # curriculum and progress UI
  zedkata-agent/             # added at the ACP milestone
extension/                  # zedkata-ext; separate WASM build
lessons/
  fixtures/                 # fixture paths relative to lessons/
tests/fixtures/lsp/          # retained protocol captures
docs/evidence/              # versions, capture notes, timing methodology
spikes/lsp-probe/            # disposable; removed after the gate
```

Package lessons as external data files. Generated lessons need a writable user data
directory for installed releases; settle that path before implementing generation.

## Decisions to settle before implementation

1. **Observation limits:** granular notifications do not prove keystrokes. Compare
   intended techniques with alternatives and implement only evidence-supported rules.
   The draft's `visual_line_select` rule needs special scrutiny.
2. **State ownership:** the architecture assigns parts of `current` to both LSP and
   TUI. Atomic replacement alone does not establish how competing updates preserve
   progress. ZK-006 must define a policy within the single-file/no-lock constraint,
   including its limitations. Propose an ADR if the constraint must change.
3. **Toolchain:** Rust 1.75 is the current target. Check dependency compatibility
   before pinning versions; record any proposed minimum-version change.
4. **Evidence:** several VERIFIED entries lack direct source links. Recheck the claims
   needed for each milestone. This plan does not certify existing claims or select
   current SDK versions.
5. **Attempt lifecycle:** an unfinished buffer is not automatically a failed attempt.
   Define finalization, retry, and rejection of late edits from old exercises.

## Validation

Test parsing, verification, transitions, and persistence with meaningful unit cases.
Replay captured traffic for LSP integration tests, including Unicode positions, CRLF,
multiple changes, undo, stale versions, and exercise switching where relevant.
Record date and Zed version for manual checks, including terminal cleanup and resize.

After scaffolding, run formatting, affected tests, and clippy for Rust changes. CI
checks the native workspace separately from the WASM extension. Automated tests do
not replace the feasibility observations or the manual MVP demonstration.

Keep core synchronous, network access in the agent, and document editing in Zed.
Defer inlay hints and packaging polish until the drill loop works. Accounts,
telemetry, leaderboards, browser UI, and other editors remain out of scope.
