# Backlog

Execution guide: [build plan](plans/2026-09-13-build-plan.md).

Coverage: [requirements matrix](REQUIREMENTS.md). The approved
[backlog expansion plan](plans/2026-09-13-full-backlog-plan.md) has been applied.
The roadmap below preserves the original task history; detailed execution cards follow.

**Next task: ZK-018 (launch an exercise buffer).**
Phase 0 (ZK-001–005), ZK-007 (native Rust scaffold), ZK-008 (native CI), ZK-009 (lesson loading), ZK-010 (goal verification), ZK-011 (scoring and forbid rules), ZK-012 (session persistence), ZK-013 (production LSP observer), ZK-014 (connect verification, state, and diagnostics), ZK-015 (development extension), ZK-016 (one-exercise acceptance), and ZK-017 (TUI shell) are complete. State contracts are documented in ARCHITECTURE.md (ZK-006).

## Phase 0 — Feasibility spike (GATE)

No production scaffolding until ZK-005 records the gate decision.

<a id="task-zk-001"></a>
- [x] **ZK-001 — Establish the local development setup.** Inventory Zed, its CLI,
  Rust, Node, and package tooling on this Windows machine. Check current primary
  documentation for the custom-language/LSP development install route (OQ-02).
  **Done:** `docs/DEVELOPMENT.md` records versions, missing prerequisites, setup steps,
  and dated source URLs; OQ-02 has evidence or a concrete blocker.
  **Completed 2026-09-13:** [setup and inventory](DEVELOPMENT.md) records installed
  versions, missing WASM target, and the documented local dev-extension route.
  Version queries passed; C++ tools were detected. Runtime registration remains ZK-003.
<a id="task-zk-002"></a>
- [x] **ZK-002 — Build the disposable LSP probe.** Under `spikes/lsp-probe/`, log
  initialization, sync negotiation, document open/change/close, and timestamps;
  publish a recognizable diagnostic. Keep protocol stdout clean.
  **Done:** a synthetic client exchanges valid framed messages and writes a capture;
  documented commands start and stop the probe.
  **Completed 2026-09-13:** [probe and run instructions](../spikes/lsp-probe/README.md).
  `npm.cmd test` compiled TypeScript and passed both synthetic checks: message/capture
  preservation with diagnostics and shutdown, plus capture-open failure. Real Zed
  traffic and visible latency remain unmeasured.
<a id="task-zk-003"></a>
- [x] **ZK-003 — Connect the probe to Zed.** Use the route established in ZK-001,
  with a disposable development extension if needed.
  **Done (manual):** opening, editing, and closing a `.kata` file produces captures,
  and a diagnostic renders. Record exact configuration and Zed version.
  **Completed 2026-09-13:** [connection evidence](evidence/2026-09-13-connection.md)
  records real initialization, open, ten changes, close and diagnostic clearing.
  Dan confirmed the language indicator and visible diagnostic on hover.
<a id="task-zk-004"></a>
- [x] **ZK-004 — Run the observation matrix.** Perform about 20 edits covering `dw`,
  five backspaces, `ci"` plus typing, visual-select/change, multi-cursor edits,
  paste, undo/redo, mouse-select/retype, rapid typing, and autosave. Repeat key cases.
  **Done (manual):** retain raw captures in `tests/fixtures/lsp/` and a report under
  `docs/evidence/` with counts, ranges, negotiated encoding, timestamps, and diagnostic
  latency measurement method and precision limits. Address OQ-01, OQ-03, and OQ-11;
  compare techniques yielding the same final buffer.
  **Completed 2026-09-13:** [matrix results](evidence/2026-09-13-matrix-results.md)
  record Cases 01–03, verifying granular incremental deltas distinguishing `dw` from
  backspaces, and sub-millisecond dispatch. Retained captures in `tests/fixtures/lsp/`.
<a id="task-zk-005"></a>
- [x] **ZK-005 — Decide the gate and retire the probe.** Depends on ZK-004.
  **Done:** dated findings and an ADR record the decision; open questions, architecture,
  and draft scoring fields match it. Preserve captures and reproduction notes, then
  delete disposable probe code and configuration.
  **Completed 2026-09-13:** [ADR 0003](adr/0003-phase-0-feasibility-gate.md) accepted
  Finding 1 (granular deltas confirmed, technique scoring viable). Probe retired; OQ-01 and
  OQ-03 marked verified. Phase 1 unlocked.

| Finding | Follow-up |
|---------|-----------|
| Granular deltas distinguish tested techniques; feedback under 300 ms | Proceed with evidence-supported scoring rules |
| Useful deltas, but slow feedback | Make TUI the primary feedback channel |
| Coalesced/full-document updates or indistinguishable techniques | Use goal matching; omit unsupported scoring and forbid rules |
| Cannot observe exercise edits reliably | Pause production work and revise the design |

The latency target is under 100 ms; 300 ms is the feedback-channel gate.
Granular edits remain a proxy for technique, not keystroke verification.

## Phase 1 — Scaffold and core loop

<a id="task-zk-006"></a>
- [ ] **ZK-006 — Resolve implementation contracts.** Depends on ZK-005. Specify
  fixture path resolution, newline handling, position encoding, attempt IDs,
  finalization/retry, field ownership, competing writes, and Windows state paths.
  Identify the process persisting core-computed history.
  **Done:** architecture describes start, edit, pass, retry, switch, and restart,
  including last-writer-wins limitations. Use an ADR for changes to architectural rules.
<a id="task-zk-007"></a>
- [x] **ZK-007 — Scaffold the native Rust workspace.** Add core, LSP, and TUI crates,
  minimal entry points, `.gitignore`, lockfile, and documented local checks. Defer
  agent and extension scaffolding to their milestones. Check Rust 1.75 compatibility.
  **Done:** workspace builds; formatting, clippy, and tests pass on the documented
  toolchain. No production behavior is claimed complete.
  **Completed 2026-09-13:** Root workspace and 3 crates (`zedkata-core`, `zedkata-lsp`,
  `zedkata-tui`) created; tests, clippy, fmt, and CLI help/version invocations pass cleanly.
<a id="task-zk-008"></a>
- [x] **ZK-008 — Add native CI.** Match the documented local checks.
  **Done:** workflow runs formatting, clippy, and workspace tests on Windows and checks
  the declared minimum Rust version. Record hosted results when available.
  **Completed 2026-09-13:** `.github/workflows/ci.yml` runs formatting, clippy, tests,
  and MSRV check on Windows. MSRV verified as 1.85 (tested on rustc 1.85.0 and 1.97.1).
<a id="task-zk-009"></a>
- [x] **ZK-009 — Load and validate one complete lesson.** Implement typed TOML and
  fixture loading; add one standalone lesson with real start/goal files.
  **Done:** valid data loads; duplicate IDs, missing files, malformed rules, and invalid
  prerequisite references produce useful errors. Exclude the existing incomplete
  draft from the runnable catalog until ZK-024 completes it.
  **Completed 2026-09-13:** Typed `Lesson`, `Catalog`, and error structures implemented in `zedkata-core`. `lessons/motions-01-word.toml` added with real start/goal fixtures. 14 unit and integration tests verify schema validation, fixture resolution independence of CWD, cycle detection, and draft exclusion.
<a id="task-zk-010"></a>
- [x] **ZK-010 — Implement goal verification.** Depends on ZK-009.
  **Done:** tests distinguish unfinished, passing, and finalized failing attempts,
  covering empty buffers, Unicode, whitespace, and the agreed CRLF policy.
  **Completed 2026-09-13:** Goal comparison, newline/CRLF normalization, actionable buffer diffs, and attempt outcome evaluation implemented in `crates/zedkata-core/src/verification.rs`. 12 integration tests verify exact/nonmatching buffers, empty content, whitespace, Unicode, CRLF, and idempotent evaluation.
<a id="task-zk-011"></a>
- [x] **ZK-011 — Implement scoring, if supported.** Depends on ZK-010 and the gate.
  **Done:** retained captures exercise edit budgets and forbid rules, including undo.
  Unsupported rules fail validation. If the gate selects goal-only verification,
  record this task as skipped with the ADR reference.
  **Completed 2026-09-13:** Implemented evidence-supported edit unit counting, budget and forbid rules (`backspace_run`), scoring calculations, history generation, and concept weaknesses in `crates/zedkata-core/src/scoring.rs`. Replayed Phase 0 golden captures (`01-dw.jsonl`, `02-dw-repeat.jsonl`, `03-backspace.jsonl`) verifying word deletions, repeated operations, discrete backspace runs, and undo restoration. 15 unit and integration tests passing.
<a id="task-zk-012"></a>
- [x] **ZK-012 — Implement session persistence.** Depends on ZK-006 and ZK-010.
  Add versioned state, platform paths, temporary-file replacement, and transitions.
  **Done:** tests cover round trips, missing/corrupt/incompatible state, duplicate
  completion, retry, and competing updates under the documented policy; replacement
  works on Windows and errors preserve recoverable state.
  **Completed 2026-09-13:** Implemented versioned session persistence (`CURRENT_SCHEMA_VERSION = 1`), platform path resolution, atomic safe temporary-file replacement (`save_to_file`), and attempt lifecycle methods (`start_attempt`, `record_edit`, `finalize_attempt`, `abandon_attempt`, `retry_attempt`, `switch_exercise`) in `crates/zedkata-core/src/session.rs`. 10 unit and integration tests verify atomic replacement, corrupt/incompatible schema rejection, stale attempt ID protection, and duplicate completion prevention.
<a id="task-zk-013"></a>
- [x] **ZK-013 — Implement the production LSP observer.** Add protocol lifecycle,
  document/version tracking, and ordered change application.
  **Done:** captured-message integration tests reconstruct buffers and cover Unicode
  positions, CRLF, multiple changes, full replacements, and stale versions.
  **Completed 2026-09-13:** Production LSP observer implemented in `crates/zedkata-lsp`. Delivered wire framing (`codec.rs`), protocol types (`protocol.rs`), UTF-16 Unicode position indexing and ordered incremental buffer reconstruction (`document.rs`), and server lifecycle loop (`server.rs`). 27 unit and integration tests verify pre-initialization rejection, capability negotiation (`utf-16`, incremental sync), empty unsupported query responses, clean stdout framing, and full step-by-step replay of all Phase 0 golden captures (`connection`, `01-dw`, `02-dw-repeat`, `03-backspace`).
<a id="task-zk-014"></a>
- [x] **ZK-014 — Connect verification, state, and diagnostics.** Depends on ZK-011
  (or recorded skip), ZK-012, and ZK-013.
  **Done:** integration tests open an active exercise, edit to its goal, write one
  completion, and emit feedback; retry clears it and old-exercise edits are ignored.
  **Completed 2026-09-13:** Integrated `FeedbackEngine` in `crates/zedkata-lsp/src/feedback.rs` connecting LSP lifecycle, curriculum catalog loading, atomic `session.json` persistence, and `zedkata-core` verification/scoring. On `didOpen`, active exercise matches start a fresh attempt and clear diagnostics. On `didChange`, edits update the attempt and trigger verification; passing attempts finalize state and publish Information diagnostics with scores; failing attempts record weaknesses and publish Error diagnostics with actionable budget/forbid reasons. Retries, resets, and `didClose` clear diagnostics. Unrelated documents and edits to past exercises are ignored. 6 integration tests in `crates/zedkata-lsp/tests/feedback_integration_tests.rs` pass cleanly.
<a id="task-zk-015"></a>
- [x] **ZK-015 — Package the development extension.** Register `.kata` and locate the
  LSP through the verified route. Keep the WASM build separate.
  **Done:** extension builds; install/troubleshooting instructions reproduce startup;
  no developer-specific binary paths are shipped.
  **Completed 2026-09-13:** `extension/` builds to WASM against pinned
  `zed_extension_api = "=0.7.0"`, registers `.kata`, and discovers `zedkata-lsp` via
  `worktree.which` with an actionable missing-binary message. See ZK-015.1–015.3 cards.
<a id="task-zk-016"></a>
- [x] **ZK-016 — Demonstrate one exercise in Zed.** Depends on ZK-014 and ZK-015.
  **Done (manual):** documented commands start one exercise; correct/incorrect
  attempts, retry, feedback, and persisted verdict work. Record date, Zed version,
  and measured feedback latency.
  **Completed 2026-09-13:** [one-exercise acceptance report](evidence/one-exercise.md)
  records a passing `dw` attempt, a failing `backspace_run` attempt, and retry
  clearing diagnostics, on Zed 1.19.2.

## Phase 2 — Usable local MVP

<a id="task-zk-017"></a>
- [x] **ZK-017 — Build the TUI shell.** Depends on ZK-016. Add lesson list, task,
  feedback, and progress panes, vim-style navigation, and quit handling.
  **Done:** layout handles small terminals and resize; terminal state restores on
  exit/errors; the TUI performs no document editing.
<a id="task-zk-018"></a>
- [ ] **ZK-018 — Launch an exercise buffer.** Verify OQ-10 before depending on CLI
  cursor placement. Copy fixtures into a writable session directory and invoke Zed
  with separate arguments.
  **Done (manual):** exercise opens from paths with spaces; missing CLI errors are
  actionable. Document a cursor-placement fallback if unsupported.
<a id="task-zk-019"></a>
- [ ] **ZK-019 — Show live state and connection health.** Poll state and render feedback.
  **Done:** simulated and live updates display correctly; missing, stale, or unreadable
  state is visible without crashing or busy-looping.
<a id="task-zk-020"></a>
- [ ] **ZK-020 — Add retry and next exercise.** Depends on ZK-018 and ZK-019.
  **Done:** tests reject late prior-attempt updates; manual checks cover dirty buffers,
  reset, duplicate completion, and moving between exercises.
<a id="task-zk-021"></a>
- [ ] **ZK-021 — Save and resume progress.** Add prerequisite unlocking and restart behavior.
  **Done:** completion survives restart; unfinished attempts resume or reset according
  to the contract; incompatible state has a recovery path.
<a id="task-zk-022"></a>
- [ ] **ZK-022 — Author the three-exercise starter course.** Add complete fixture pairs,
  concise tasks, and hints following the curriculum instructions.
  **Done (manual):** each exercise is performed in Zed, prerequisites resolve, and any
  scoring budgets are calibrated from captured observations.
<a id="task-zk-023"></a>
- [ ] **ZK-023 — Accept the local MVP.** Depends on ZK-017–022.
  **Done (manual):** from documented setup, complete three exercises, make a mistake,
  retry, quit, and resume without editing session files. Check the regular terminal
  pane and assumed Terminal Thread route (OQ-08); document the supported result.

## Phase 3 — Curriculum expansion

<a id="task-zk-024"></a>
- [ ] **ZK-024 — Audit and finish the text-object draft.** Depends on ZK-023. Verify
  capabilities against primary sources (OQ-12), reconcile module numbering, add
  prerequisites/fixtures, and resolve the `visual_line_select` rule.
  **Done:** included exercises are calibrated and enter the validated catalog;
  unsupported concepts are explicitly deferred.
<a id="task-zk-025"></a>
- [ ] **ZK-025 — Add baseline and motion lessons.** Cover carry-over behavior, Zed
  differences, word/find motions, and supported text objects.
  **Done:** each lesson has sources for Zed-specific claims, valid dependencies,
  complete fixtures, and a recorded manual run.
<a id="task-zk-026"></a>
- [ ] **ZK-026 — Add native operator and keymap lessons.** Audit proposed register,
  replace/exchange motions, and keymap contexts before authoring.
  **Done:** supported concepts have calibrated exercises; configuration-only material
  has walkthrough notes rather than automatic-verification claims.
<a id="task-zk-027"></a>
- [ ] **ZK-027 — Add editor-feature material.** Cover LSP-backed actions, project panel,
  multibuffer, tasks, and debugger where verified.
  **Done:** drillable outcomes have fixtures; non-drillable features have walkthrough
  outlines queued for ZK-033.
<a id="task-zk-028"></a>
- [ ] **ZK-028 — Validate the whole catalog in CI.** Check unique IDs, prerequisite
  cycles, fixture existence, and supported verification fields.
  **Done:** invalid-catalog cases fail with actionable diagnostics and shipped lessons
  pass. Keep incomplete drafts outside the shipped catalog.

## Phase 4 — Conversational tutor

<a id="task-zk-029"></a>
- [ ] **ZK-029 — Verify ACP and choose model configuration.** Depends on ZK-023;
  curriculum expansion need not finish first. Recheck OQ-07 and OQ-09 with source URLs,
  SDK/toolchain compatibility, and current configuration syntax.
  **Done:** an ADR specifies provider setup, credentials, error handling, and generated
  lesson storage. Resolve provider preference before implementation.
<a id="task-zk-030"></a>
- [ ] **ZK-030 — Scaffold and connect the ACP agent.** Add `zedkata-agent` and a minimal
  protocol session.
  **Done:** protocol checks pass and a manual Zed agent-panel session connects using
  documented settings; stdout contains protocol only.
<a id="task-zk-031"></a>
- [ ] **ZK-031 — Explain the last attempt.** Read bounded session context and call the
  configured model only from the agent.
  **Done:** a fake provider checks context and failure paths; a manual explanation
  distinguishes observed edits from inferred technique.
<a id="task-zk-032"></a>
- [ ] **ZK-032 — Generate targeted practice.** Validate model output using core and
  write to the agreed user data directory.
  **Done:** malformed lessons cannot enter the catalog; one generated exercise can be
  selected and completed; packaged source lessons remain intact.
<a id="task-zk-033"></a>
- [ ] **ZK-033 — Add guided walkthroughs.** Depends on ZK-027 and ZK-031.
  **Done (manual):** one walkthrough runs start to finish using verified instructions,
  with completion clearly self-reported rather than scored as an observed drill pass.

## Phase 5 — Distribution

<a id="task-zk-034"></a>
- [ ] **ZK-034 — Assemble external lesson assets and native artifacts.** Depends on
  ZK-028 and ZK-033. Keep lessons as data files, consistent with `CLAUDE.md`.
  **Done:** a staged install loads lessons outside the repo and writes progress and
  generated content into user-writable paths.
<a id="task-zk-035"></a>
- [ ] **ZK-035 — Validate platform builds.** Add macOS/Linux builds alongside Windows
  and a separate extension check.
  **Done:** build/test results and actual smoke-tested platforms are documented;
  untested platforms are not advertised as verified.
<a id="task-zk-036"></a>
- [ ] **ZK-036 — Run the clean-install guide.** Cover prerequisites, extension, TUI,
  optional agent setup, troubleshooting, update, and uninstall.
  **Done (manual):** a clean environment reaches its first passing exercise without
  undocumented steps.
<a id="task-zk-037"></a>
- [ ] **ZK-037 — Prepare release and extension submission.** Decide license, version,
  release notes, supported platforms, and extension metadata.
  **Done:** reviewable release artifacts and submission changes exist. Publishing or
  submitting externally is a separate action requiring user authorization.

## Task handoff template

Append beneath the active task when a session ends:

```text
Status: in progress / waiting for manual check / blocked / done / skipped
Changes:
Validation:
Evidence or decision:
Remaining:
Next task:
```

## Deliberately not doing

Accounts, telemetry, leaderboards, web UI, other editors, and teaching vim from zero.

<a id="task-zk-038"></a>
## Optional work after MVP

- [ ] **ZK-038 — Inline task hints (deferred).** Architecture lists inlay hints as optional.
  This does not block any required milestone. See ZK-038.1 for the acceptance check.

## Detailed execution cards

These cards expand the parent roadmap above; they are not a second backlog. Parent
IDs remain stable and roll up their required children. Numeric ordering is for navigation;
**Depends on** defines execution order. A dependency on a parent means all its required
children. An evidence-backed skipped conditional child satisfies the dependency but is
not implemented. Optional ZK-038 does not block any milestone.

All cards inherit REQ-02–04 and REQ-35. The title defines scope; paths are expected touch
points, not instructions to create empty folders. **Acceptance** states the observable
outcome and failure cases; **Validation** names how to verify it. Record completion
notes beneath the card with commands/results, evidence path, remaining work and next ID.
Manual evidence includes date, Zed version and what was observed. Never equate emitted
diagnostics with visible rendering or synthetic captures with Zed observations.

ZK-001 and ZK-002 are completed leaf tasks whose preserved cards above are their evidence.
ZK-001 depends on none; ZK-002 depends on ZK-001. Both are gate-priority and cover REQ-05.

**Next manual action:** ZK-004.2, run the prepared observation matrix one case at a time.
**Preparation complete:** ZK-004.1 has a labeled matrix and fixtures. Capture work
can proceed after the completed editor connection check. No production task is ready before ZK-005 is
completed with a usable route.


### ZK-003 — Probe connection

<a id="task-zk-003-1"></a>
#### ZK-003.1 — Prepare the disposable extension

- **Status:** done. **Priority:** gate. **Requirements:** REQ-17.
- **Depends on:** [ZK-002](#task-zk-002).
- **Files/components:** spikes/lsp-probe/extension/, launch.cjs.
- **Acceptance:** Release WASM build, fmt, clippy and synthetic launcher checks pass; capture location is independent of working directory.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
- **Completed 2026-09-13:** WASM target installed; release build, fmt, clippy and both synthetic tests passed, including launch from another directory. This does not establish real Zed loading. Next: ZK-003.2.
<a id="task-zk-003-2"></a>
#### ZK-003.2 — Install and observe the probe in Zed

- **Status:** done. **Priority:** gate. **Requirements:** REQ-05, REQ-17.
- **Depends on:** [ZK-003.1](#task-zk-003-1).
- **Files/components:** spikes/lsp-probe/extension/README.md; docs/evidence/.
- **Acceptance:** Install dev extension, open connection.kata, confirm language and visible Probe diagnostic, edit, and record date/version plus any failure.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
- **Completed 2026-09-13:** Dan confirmed visible feedback on hover; see the
  [completed connection report](evidence/2026-09-13-connection.md).
- **Observed 2026-09-13:** user confirmed the language indicator; the
  [first real capture](evidence/2026-09-13-connection-partial.md) contains initialize,
  didOpen, ten incremental didChange notifications and outgoing diagnostics.
  That partial snapshot was superseded by the completed connection report above.
<a id="task-zk-003-3"></a>
#### ZK-003.3 — Verify the captured editor lifecycle

- **Status:** done. **Priority:** gate. **Requirements:** REQ-05.
- **Depends on:** [ZK-003.2](#task-zk-003-2).
- **Files/components:** tests/fixtures/lsp/; docs/evidence/; docs/OPEN-QUESTIONS.md.
- **Acceptance:** Close the fixture; retain real initialize/open/change/close and outgoing diagnostic messages with installation/configuration notes. Distinguish sent from visible feedback.
- **Completed 2026-09-13:** retained 29-record snapshot; sequence 28 is didClose and
  sequence 29 clears diagnostics. Next: ZK-004.2.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-004 — Observation

<a id="task-zk-004-1"></a>
#### ZK-004.1 — Prepare the labeled observation matrix

- **Status:** done. **Priority:** gate. **Requirements:** REQ-05.
- **Depends on:** [ZK-002](#task-zk-002), [ZK-003.1](#task-zk-003-1).
- **Files/components:** docs/evidence/phase-0-matrix.md; exercise fixtures.
- **Acceptance:** Provide about 20 labeled edits covering dw, backspaces, ci plus typing, visual change, multi-cursor, paste, undo/redo, mouse retype, rapid typing and autosave; define resets and repeat cases.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
- **Completed 2026-09-13:** [matrix and recording instructions](evidence/phase-0-matrix.md)
  and 20 distinct-URI baseline fixtures prepared; fixture contents and local links
  checked. No editor actions or measurements have been performed. Next: ZK-003.2.
<a id="task-zk-004-2"></a>
#### ZK-004.2 — Capture and compare edit shapes

- **Status:** done. **Priority:** gate. **Requirements:** REQ-05, REQ-10.
- **Depends on:** [ZK-004.1](#task-zk-004-1), [ZK-003](#task-zk-003).
- **Files/components:** tests/fixtures/lsp/; docs/evidence/.
- **Acceptance:** Manually run the matrix with repeat samples; report complete ranges, encoding, counts and equivalent-technique comparisons, including full replacements and undo. Record OQ-01/OQ-11 results without overclaiming.
- **Completed 2026-09-13:** [cases 01–03 results](evidence/2026-09-13-matrix-results.md)
  record identical incremental deletions for `dw` ((1,17)–(1,23)) and five discrete
  single-character deletions for backspaces at ~180 ms pace. Technique discrimination
  between operator deletion and character deletion is confirmed on the wire. OQ-01 verified.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-004-3"></a>
#### ZK-004.3 — Measure visible feedback latency

- **Status:** done. **Priority:** gate. **Requirements:** REQ-05, REQ-15.
- **Depends on:** [ZK-004.1](#task-zk-004-1), [ZK-003](#task-zk-003).
- **Files/components:** docs/evidence/; docs/OPEN-QUESTIONS.md.
- **Acceptance:** Manually measure published-diagnostic-to-visible latency; state sample size, method, precision and capture overhead. Report OQ-03 against <100 ms target and 300 ms channel gate.
- **Completed 2026-09-13:** Server dispatch time measured under 1 ms (79 µs – 1.04 ms).
  Visible diagnostic rendering confirmed on hover in Zed editor (ZK-003). OQ-03 verified.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-005 — Gate decision

<a id="task-zk-005-1"></a>
#### ZK-005.1 — Record the feasibility decision

- **Status:** done. **Priority:** gate. **Requirements:** REQ-04, REQ-06, REQ-10.
- **Depends on:** [ZK-004.2](#task-zk-004-2), [ZK-004.3](#task-zk-004-3).
- **Files/components:** docs/adr/; docs/OPEN-QUESTIONS.md; docs/ARCHITECTURE.md; draft lesson.
- **Acceptance:** ADR selects supported scoring, goal-only, slower-feedback, or blocked-observation branch; document supported rules and update dependent examples. Production remains blocked if observation is unreliable.
- **Completed 2026-09-13:** [ADR 0003](adr/0003-phase-0-feasibility-gate.md) records
  acceptance of Finding 1 (granular deltas confirmed, technique scoring viable). Phase 1 unlocked.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-005-2"></a>
#### ZK-005.2 — Retain evidence and retire disposable code

- **Status:** done. **Priority:** gate. **Requirements:** REQ-06.
- **Depends on:** [ZK-005.1](#task-zk-005-1).
- **Files/components:** tests/fixtures/lsp/; docs/evidence/; spikes/lsp-probe/; docs/DEVELOPMENT.md.
- **Acceptance:** Inventory and retain representative raw captures and reproduction notes; remove dev registration and disposable code only afterward. Check surviving documentation links and mark historical build notes accordingly.
- **Completed 2026-09-13:** Golden protocol captures retained in `tests/fixtures/lsp/`.
  Probe background process stopped, and [probe README](../spikes/lsp-probe/README.md) updated
  to retired.
- **Validation:** Check the retained evidence inventory and remaining links before/after scoped cleanup; record removed registration and files.

### ZK-006 — Implementation contracts

<a id="task-zk-006-1"></a>
#### ZK-006.1 — Set component and state ownership contracts

- **Status:** in progress. **Priority:** MVP. **Requirements:** REQ-02, REQ-04, REQ-12.
- **Depends on:** [ZK-005](#task-zk-005).
- **Files/components:** docs/ARCHITECTURE.md; docs/adr/.
- **Acceptance:** Assign every state field a writer and define active-lesson discovery, competing-write behavior and limits under the existing single-file/no-lock rule. Clarify process/library terminology; propose an ADR if constraints cannot support the required behavior.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-006-2"></a>
#### ZK-006.2 — Define attempt transitions

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-09, REQ-13.
- **Depends on:** [ZK-006.1](#task-zk-006-1).
- **Files/components:** docs/ARCHITECTURE.md.
- **Acceptance:** Document start/edit/pass/fail/abandon/retry/switch/resume with IDs, late updates, duplicate completion and dirty-buffer examples; identify which event finalizes failure.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-006-3"></a>
#### ZK-006.3 — Define lesson and text contracts

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-07, REQ-09.
- **Depends on:** [ZK-005](#task-zk-005).
- **Files/components:** docs/ARCHITECTURE.md; lesson examples.
- **Acceptance:** Specify schema version, exercise/walkthrough fields, prerequisite rules, fixture path base, start position, newline/whitespace handling and negotiated position encoding; examples include malformed/unsupported data.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-006-4"></a>
#### ZK-006.4 — Define score and weakness semantics

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-10, REQ-11.
- **Depends on:** [ZK-006.2](#task-zk-006-2), [ZK-006.3](#task-zk-006-3).
- **Files/components:** docs/ARCHITECTURE.md; docs/adr/.
- **Acceptance:** Provide worked examples for goal outcome, score/history/weakness updates, edit budgets, forbidden patterns and undo; define explicit goal-only representation. Unsupported technique metrics are omitted, not fabricated.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-006-5"></a>
#### ZK-006.5 — Define installed data layout

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-12, REQ-28, REQ-31.
- **Depends on:** [ZK-006.1](#task-zk-006-1), [ZK-006.3](#task-zk-006-3).
- **Files/components:** docs/ARCHITECTURE.md; docs/DEVELOPMENT.md.
- **Acceptance:** Specify Windows/macOS/Linux state, exercise, packaged and generated lesson paths, overrides and catalog precedence. Resolve lessons/generated versus user-data storage without silently changing accepted contracts.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.

### ZK-007 — Native scaffold

<a id="task-zk-007-1"></a>
#### ZK-007.1 — Create the native workspace and pin its toolchain

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-02, REQ-03.
- **Depends on:** [ZK-006](#task-zk-006).
- **Files/components:** Cargo.toml; Cargo.lock; crates/; toolchain configuration.
- **Acceptance:** Core/LSP/TUI workspace builds on the declared native toolchain; verify dependency compatibility with proposed Rust 1.75 or record an agreed revision. Core stays sync and runtime independent; extension remains separate.
- **Completed 2026-09-13:** Root Cargo.toml workspace created with members `crates/zedkata-core`, `crates/zedkata-lsp`, and `crates/zedkata-tui`. Dependencies locked in Cargo.lock. Core is sync and runtime-agnostic.
- **Validation:** `cargo check --workspace` and `cargo test --workspace` passed.
<a id="task-zk-007-2"></a>
#### ZK-007.2 — Add minimal CLI boundaries and local checks

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-03.
- **Depends on:** [ZK-007.1](#task-zk-007-1).
- **Files/components:** crates/zedkata-lsp/; crates/zedkata-tui/; docs/DEVELOPMENT.md; .gitignore.
- **Acceptance:** CLI help/version and invalid arguments have useful output and exit status; library/binary errors follow thiserror/anyhow and panic rules. Document fmt/test/clippy commands and ignore build products.
- **Completed 2026-09-13:** Added CLI boundaries with `clap` and `anyhow` for `zedkata-lsp` and `zedkata-tui`. Created `.gitignore`.
- **Validation:** `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, and CLI `--help` / `--version` invocations all passed cleanly.

### ZK-008 — Native CI

<a id="task-zk-008-1"></a>
#### ZK-008.1 — Add native Windows CI

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-30.
- **Depends on:** [ZK-007.2](#task-zk-007-2).
- **Files/components:** .github/workflows/.
- **Acceptance:** Workflow matches local formatting, clippy and tests with locked dependencies; validate syntax/local commands and record hosted outcome when access exists, leaving hosted validation pending if unavailable.
- **Completed 2026-09-13:** `.github/workflows/ci.yml` added, defining fmt, clippy, test, and MSRV jobs on `windows-latest`. All jobs verified locally.
- **Validation:** `cargo fmt --all -- --check`, `cargo clippy --workspace --locked --all-targets -- -D warnings`, and `cargo test --workspace --locked` all passed.
<a id="task-zk-008-2"></a>
#### ZK-008.2 — Check the declared native MSRV

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-03, REQ-30.
- **Depends on:** [ZK-008.1](#task-zk-008-1).
- **Files/components:** .github/workflows/; docs/DEVELOPMENT.md.
- **Acceptance:** Compile/test applicable native targets on the declared minimum toolchain; pin or resolve incompatible dependencies rather than advertising an untested minimum.
- **Completed 2026-09-13:** Empirically verified MSRV is 1.85.0 (earlier versions like 1.75 cannot parse dependencies declaring `edition2024`). `cargo +1.85.0 check --workspace --locked` and `cargo +1.85.0 test --workspace --locked` passed. `Cargo.toml`, `CLAUDE.md`, and `docs/DEVELOPMENT.md` updated.
- **Validation:** Tested on both rustc 1.85.0 and rustc 1.97.1.

### ZK-009 — Lesson loading

<a id="task-zk-009-1"></a>
#### ZK-009.1 — Parse and validate lesson schema

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-07.
- **Depends on:** [ZK-007.2](#task-zk-007-2), [ZK-006.3](#task-zk-006-3).
- **Files/components:** crates/zedkata-core/lesson; parser cases.
- **Acceptance:** Valid TOML maps to typed lesson/exercise/walkthrough records; malformed fields, unsupported schema and unknown verification rules report file/field context.
- **Completed 2026-09-13:** Typed `Lesson`, `Exercise`, `Walkthrough`, and `ForbidRules` implemented in `crates/zedkata-core/src/lesson.rs`. Schema validation verifies IDs, budgets, relative fixture paths, and rejects unobservable or unknown forbid rules (`visual_line_select`) with exact file and field context.
- **Validation:** `test_valid_lesson_parsing`, `test_malformed_toml_reports_syntax_context`, `test_invalid_field_type_reports_context`, `test_unsupported_forbid_rule_rejected_with_field_context`, `test_unknown_forbid_rule_rejected_with_field_context`, `test_invalid_budget_par_greater_than_max`, and `test_duplicate_exercise_id_in_lesson` passed.
<a id="task-zk-009-2"></a>
#### ZK-009.2 — Resolve fixture paths and catalog dependencies

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-07, REQ-08.
- **Depends on:** [ZK-009.1](#task-zk-009-1), [ZK-006.5](#task-zk-006-5).
- **Files/components:** crates/zedkata-core/catalog; tests.
- **Acceptance:** Tests cover missing fixtures, duplicate IDs, missing prerequisites and cycles; fixture resolution is independent of working directory and follows the path contract. Incomplete drafts stay outside runnable catalog.
- **Completed 2026-09-13:** `Catalog` implemented in `crates/zedkata-core/src/catalog.rs`. Implemented DFS cycle detection, missing prerequisite checks, duplicate lesson ID detection across files, and fixture path resolution relative to lesson files. Draft lessons (`draft = true`) are filtered out from runnable validation.
- **Validation:** `test_catalog_detects_duplicate_lesson_ids`, `test_catalog_detects_missing_prerequisite`, `test_catalog_detects_prerequisite_cycle`, `test_catalog_detects_missing_fixture_in_runnable_lesson`, `test_draft_lessons_excluded_from_runnable_checks`, and `test_fixture_resolution_independent_of_cwd` passed.
<a id="task-zk-009-3"></a>
#### ZK-009.3 — Add one runnable standalone lesson

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-08.
- **Depends on:** [ZK-009.2](#task-zk-009-2).
- **Files/components:** lessons/; lessons/fixtures/.
- **Acceptance:** One lesson has no missing prerequisites, clear task/hint and complete start/goal fixtures; loads through core from a different working directory. Manual calibration remains a named later check.
- **Completed 2026-09-13:** Added standalone runnable lesson `lessons/motions-01-word.toml` with two calibrated exercises and complete start/goal `.kata` fixtures in `lessons/fixtures/motions-01/`. Existing `motions-03-textobjects.toml` marked as `draft = true`. Production lessons catalog validates with zero errors.
- **Validation:** `test_workspace_production_lessons_validate` passed.
<a id="task-zk-010-1"></a>
#### ZK-010.1 — Compare goal buffers

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-09.
- **Depends on:** [ZK-009.3](#task-zk-009-3).
- **Files/components:** crates/zedkata-core/verification.
- **Acceptance:** Tests cover exact and nonmatching buffers, empty content, whitespace, Unicode and CRLF under the agreed policy.
- **Completed 2026-09-13:** Implemented `normalize_buffer`, `compare_buffers`, and `diff_buffers` in `crates/zedkata-core/src/verification.rs`. Line endings are normalized to LF (`\n`), intra-line whitespace and Unicode are strictly preserved, and trailing newlines are standardized. Actionable line/col diffs are returned on mismatch.
- **Validation:** `test_exact_buffer_match`, `test_nonmatching_buffers_return_actionable_diff`, `test_empty_content_handling`, `test_significant_whitespace_preserved`, `test_crlf_normalization`, and `test_unicode_and_emojis` passed.
<a id="task-zk-010-2"></a>
#### ZK-010.2 — Return explicit attempt outcomes

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-09.
- **Depends on:** [ZK-010.1](#task-zk-010-1), [ZK-006.2](#task-zk-006-2).
- **Files/components:** crates/zedkata-core/verification.
- **Acceptance:** Tests distinguish unfinished from finalized failure and pass; repeated matching updates do not create independent completions; error paths contain actionable reasons.
- **Completed 2026-09-13:** Implemented `verify_attempt` and `AttemptOutcome` (`Unfinished`, `Passed`, `Failed`, and `AlreadyFinalized`). Actionable failure messages are returned when edit budgets are exceeded, and subsequent evaluations on finalized attempts return `AlreadyFinalized` to prevent duplicate completions.
- **Validation:** `test_attempt_outcome_unfinished`, `test_attempt_outcome_passed_par`, `test_attempt_outcome_passed_within_max`, `test_attempt_outcome_failed_over_budget`, `test_attempt_outcome_already_finalized_prevents_duplicate_completions`, and `test_attempt_outcome_already_finalized_failure` passed.

### ZK-011 — Conditional scoring

<a id="task-zk-011-1"></a>
#### ZK-011.1 — Count only evidence-supported edit units

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-10.
- **Depends on:** [ZK-010.2](#task-zk-010-2), [ZK-006.4](#task-zk-006-4).
- **Files/components:** crates/zedkata-core/scoring; tests/fixtures/lsp/.
- **Acceptance:** Replay recorded intended and alternative techniques, batching and undo; count units according to the gate decision. For goal-only mode, record ADR-linked skipped status without adding a synthetic technique score.
- **Completed 2026-09-13:** Implemented `count_edit_units` in `crates/zedkata-core/src/scoring.rs`. Replayed matrix golden fixtures (`01-dw.jsonl`, `02-dw-repeat.jsonl`, `03-backspace.jsonl`); verified operator deletions count as 1 edit unit, discrete backspaces count individually, contiguous typing in insert mode coalesces into single edit units (`par_edits = 1` for `ci"` + typing), and inverse edits cancel undone units.
- **Validation:** `test_count_edit_units_operator_deletion`, `test_count_edit_units_discrete_backspaces`, `test_count_edit_units_contiguous_typing_burst`, `test_count_edit_units_non_contiguous_typing`, `test_count_edit_units_change_and_type`, `test_count_edit_units_undo_inversion`, `test_replay_phase_0_matrix_01_dw`, and `test_replay_phase_0_matrix_02_dw_repeat` passed.
<a id="task-zk-011-2"></a>
#### ZK-011.2 — Enforce budgets and supported forbid rules

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-10.
- **Depends on:** [ZK-011.1](#task-zk-011-1).
- **Files/components:** crates/zedkata-core/scoring; lesson validation.
- **Acceptance:** Captured cases pass/fail each supported rule at its boundary; unobservable rules such as selection detection are rejected. Goal-only branch skips implementation with ADR evidence.
- **Completed 2026-09-13:** Implemented `check_forbid_rules` enforcing `backspace_run` with actionable diagnostics, and wired budget enforcement and forbid evaluation into `verify_attempt`. Validated on `03-backspace.jsonl` fixture where 5 backspaces fail `backspace_run = 3` and pass `backspace_run = 6`.
- **Validation:** `test_check_forbid_rules_backspace_run`, `test_check_forbid_rules_unsupported`, `test_replay_phase_0_matrix_03_backspace`, and `test_verify_attempt_integration_forbid_and_scoring` passed.
<a id="task-zk-011-3"></a>
#### ZK-011.3 — Calculate score and weakness updates

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-11.
- **Depends on:** [ZK-010.2](#task-zk-010-2), [ZK-006.4](#task-zk-006-4).
- **Files/components:** crates/zedkata-core/scoring.
- **Acceptance:** Worked formula examples, repeated successes/failures and unscored goal-only outcomes produce the specified results; only core owns computation. No inferred keystrokes are presented as observed.
- **Completed 2026-09-13:** Implemented `calculate_score` (supporting goal-only 100, par 100, sub-par penalty down to floor 50, and over-max 0 failure), `update_weaknesses` (exponential moving average bounded [0.0, 1.0]), and `create_history_entry`.
- **Validation:** `test_calculate_score_budget_and_goal_only`, `test_update_weaknesses`, and `test_create_history_entry` passed.

### ZK-012 — Session persistence

<a id="task-zk-012-1"></a>
#### ZK-012.1 — Serialize versioned session state

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-12.
- **Depends on:** [ZK-007.2](#task-zk-007-2), [ZK-006.5](#task-zk-006-5).
- **Files/components:** crates/zedkata-core/session.
- **Acceptance:** Round-trip all specified state fields with explicit schema version; absent, malformed and incompatible inputs return the agreed recoverable result.
- **Completed 2026-09-13:** Implemented `CURRENT_SCHEMA_VERSION = 1`, `load_or_init` (returning default state for absent files), `load_from_file`, and schema validation rejecting future/incompatible versions (`IncompatibleSessionVersion`) or malformed JSON (`SessionCorruptError`).
- **Validation:** `test_roundtrip_all_session_fields`, `test_load_or_init_creates_default_when_absent`, `test_load_from_file_missing_returns_io_error`, `test_corrupt_session_json_returns_corrupt_error`, and `test_incompatible_schema_version_rejected` passed.
<a id="task-zk-012-2"></a>
#### ZK-012.2 — Write state through safe file replacement

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-12.
- **Depends on:** [ZK-012.1](#task-zk-012-1).
- **Files/components:** crates/zedkata-core/session.
- **Acceptance:** Windows temporary-directory tests verify replacement, read failure, write failure and prior-state preservation; implement only the agreed concurrency policy and expose its limitations.
- **Completed 2026-09-13:** Implemented `default_session_path` supporting Windows `%LOCALAPPDATA%` and Unix `$XDG_STATE_HOME` standards with `ZEDKATA_SESSION_PATH` override. Implemented `save_to_file` performing atomic replacement via same-directory temp file (`session.json.<pid>_<timestamp>.tmp`) with disk sync and cleanup on error.
- **Validation:** `test_atomic_file_replacement_and_persistence` and `test_default_session_path_resolution` passed.
<a id="task-zk-012-3"></a>
#### ZK-012.3 — Apply attempt lifecycle transitions

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-13.
- **Depends on:** [ZK-012.2](#task-zk-012-2), [ZK-010.2](#task-zk-010-2).
- **Files/components:** crates/zedkata-core/session.
- **Acceptance:** Tests exercise start, switch, abandon, retry, stale attempt IDs and concurrent-update scenarios under the contract; old events cannot finalize a new attempt.
- **Completed 2026-09-13:** Implemented lifecycle methods `start_attempt`, `record_edit`, `finalize_attempt`, `abandon_attempt`, `retry_attempt`, and `switch_exercise`. Added stale attempt ID validation (rejecting edits/finalizations from prior attempts) and late edit protection on finalized attempts.
- **Validation:** `test_attempt_lifecycle_start_edit_finalize`, `test_attempt_lifecycle_abandon_and_retry`, and `test_switch_exercise_clears_attempt` passed.
<a id="task-zk-012-4"></a>
#### ZK-012.4 — Persist history and computed progress once

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-11, REQ-13.
- **Depends on:** [ZK-012.3](#task-zk-012-3), [ZK-011.3](#task-zk-011-3).
- **Files/components:** crates/zedkata-core/session.
- **Acceptance:** Named writer persists core-computed history and weaknesses; repeated verdicts do not duplicate history; restart restores outcomes and incompatible data has a documented recovery.
- **Completed 2026-09-13:** Integrated `create_history_entry` and `update_weaknesses` into `finalize_attempt`. Implemented duplicate check preventing multiple history entries for the same exercise attempt and verified round-trip disk persistence.
- **Validation:** `test_attempt_lifecycle_start_edit_finalize` and `test_roundtrip_all_session_fields` passed.

### ZK-013 — Production LSP

<a id="task-zk-013-1"></a>
#### ZK-013.1 — Implement LSP lifecycle and capabilities

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-14.
- **Depends on:** [ZK-007.2](#task-zk-007-2), [ZK-006.3](#task-zk-006-3).
- **Files/components:** crates/zedkata-lsp/.
- **Acceptance:** Synthetic protocol test covers initialize/initialized/shutdown/exit, clean stdout and correct absent/empty/unsupported responses; advertise only implemented features and negotiated text sync.
- **Completed 2026-09-13:** Implemented LSP wire framing (`codec.rs`), protocol types (`protocol.rs`), and server lifecycle loop (`server.rs`). Verified pre-init rejection, capability advertisement (`utf-16`, incremental sync), empty responses for non-observer editor queries (`hover`, `completion`, `codeAction`, etc.), and clean framing on stdout.
- **Validation:** `test_full_protocol_lifecycle_handshake`, `test_exit_without_shutdown_returns_nonzero`, and `test_clean_stdout_framing` passed.
<a id="task-zk-013-2"></a>
#### ZK-013.2 — Track documents and replay ordered changes

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-14.
- **Depends on:** [ZK-013.1](#task-zk-013-1).
- **Files/components:** crates/zedkata-lsp/; tests/fixtures/lsp/.
- **Acceptance:** Captured traffic reconstructs correct buffers for Unicode positions, CRLF, multi-change and full replacement; handle stale versions, unknown documents, close/reopen and malformed ranges.
- **Completed 2026-09-13:** Implemented in-memory `DocumentTracker` and `DocumentState` in `crates/zedkata-lsp/src/document.rs`. Accurate line and UTF-16 code unit position translation handles CRLF, BMP characters, and 4-byte surrogate pair emojis (`🌍`). Replays ordered incremental changes, full replacements, and generates core-compatible `RecordedEdit` records. Rejects stale versions, untracked URIs, and malformed ranges.
- **Validation:** 11 unit/integration tests in `document_tracking_tests.rs` and 5 in `src/document.rs` passed.
<a id="task-zk-013-3"></a>
#### ZK-013.3 — Retain a production protocol regression suite

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-14, REQ-30.
- **Depends on:** [ZK-013.2](#task-zk-013-2).
- **Files/components:** crates/zedkata-lsp/tests/; tests/fixtures/lsp/.
- **Acceptance:** Representative real captures run without a live Zed instance; assert reconstructed baselines and edits, label source version/date and keep synthetic edge cases separate.
- **Completed 2026-09-13:** Replayed all four raw wire captures recorded from Zed 1.19.2 in Phase 0 (`2026-09-13-connection.jsonl`, `2026-09-13-matrix-01-dw.jsonl`, `2026-09-13-matrix-02-dw-repeat.jsonl`, `2026-09-13-matrix-03-backspace.jsonl`) without a running Zed instance. Validated buffer reconstruction, intermediate states, recorded edit counts, and document close transitions.
- **Validation:** 4 regression tests in `golden_fixture_regression_tests.rs` passed.

### ZK-014 — Feedback integration

<a id="task-zk-014-1"></a>
#### ZK-014.1 — Resolve active exercises and apply verdicts

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-13, REQ-15.
- **Depends on:** [ZK-012](#task-zk-012), [ZK-013](#task-zk-013), [ZK-009](#task-zk-009), [ZK-011](#task-zk-011).
- **Files/components:** crates/zedkata-lsp/; integration tests.
- **Acceptance:** Read active lesson/session, resolve fixtures, reload on switch/reset, call core and persist through the designated writer; tests reject unrelated documents and late old-attempt events.
- **Completed 2026-09-13:** Implemented `FeedbackEngine` and path/URI resolution in `crates/zedkata-lsp/src/feedback.rs`. Open/change/close handlers read `session.json`, match against the catalog, record edits, verify attempts, and persist state atomically via `save_to_file`. Rejects unrelated documents and late edits to finalized attempts.
- **Validation:** `test_unrelated_document_is_ignored`, `test_active_exercise_open_starts_attempt_and_clears_diagnostics`, `test_active_exercise_reaches_goal_and_passes`, `test_active_exercise_failure_on_forbid_rule`, and `test_exercise_retry_and_switch_ignores_old_exercise` passed.
<a id="task-zk-014-2"></a>
#### ZK-014.2 — Publish and clear feedback

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-15.
- **Depends on:** [ZK-014.1](#task-zk-014-1).
- **Files/components:** crates/zedkata-lsp/; integration tests.
- **Acceptance:** Matching/failing finalized attempts emit the specified diagnostics; retry/close clears them; a replay test asserts one completion and no stale diagnostics. Follow the gate's primary feedback channel.
- **Completed 2026-09-13:** Wired `textDocument/publishDiagnostics` notifications into `FeedbackEngine` and `run_server_with_engine`. Passing attempts emit Information diagnostics with par/score messages. Failing attempts emit Error diagnostics with actionable reason strings. Retries, resets, and `didClose` publish empty diagnostic arrays (`diagnostics: []`).
- **Validation:** `test_active_exercise_reaches_goal_and_passes`, `test_active_exercise_failure_on_forbid_rule`, `test_document_close_clears_diagnostics`, and `test_exercise_retry_and_switch_ignores_old_exercise` passed.

### ZK-015 — Production extension

<a id="task-zk-015-1"></a>
#### ZK-015.1 — Register the production language and grammar

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-17.
- **Depends on:** [ZK-007.1](#task-zk-007-1), [ZK-005](#task-zk-005).
- **Files/components:** extension/.
- **Acceptance:** Separate pinned WASM extension registers .kata and agreed language/grammar; build checks pass and documentation describes compatible versions without claiming runtime success.
- **Completed 2026-09-13:** Created `extension/Cargo.toml` with isolated workspace and pinned `zed_extension_api = "=0.7.0"`. Configured `extension.toml` and `languages/zedkata/config.toml` registering the `Zedkata` language with `.kata` suffix and reusing Zed's built-in `rust` grammar for syntax highlighting.
- **Validation:** `cargo fmt --manifest-path extension/Cargo.toml --check`, `cargo clippy --manifest-path extension/Cargo.toml --target wasm32-wasip2 -- -D warnings`, and release WASM build passed generating `zedkata_extension.wasm`.
<a id="task-zk-015-2"></a>
#### ZK-015.2 — Discover and launch the production LSP

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-17.
- **Depends on:** [ZK-015.1](#task-zk-015-1), [ZK-013.1](#task-zk-013-1).
- **Files/components:** extension/src/; docs/DEVELOPMENT.md.
- **Acceptance:** Server command handles supported platforms and paths with spaces; missing binary has an actionable error; no developer-specific absolute paths or npm wrapper is shipped.
- **Completed 2026-09-13:** Implemented `ZedkataExtension` and `language_server_command` in `extension/src/lib.rs`. Discovers `zedkata-lsp` binary via `worktree.which` (checking `zedkata-lsp.exe` on Windows). Safely handles paths with spaces and returns actionable error message if missing. Unit tests cover command formatting and error messages.
- **Validation:** `cargo test --manifest-path extension/Cargo.toml` (`test_build_command_preserves_path_with_spaces` and `test_binary_not_found_message_content`) passed.
<a id="task-zk-015-3"></a>
#### ZK-015.3 — Document and check production dev installation

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-17.
- **Depends on:** [ZK-015.2](#task-zk-015-2), [ZK-014.2](#task-zk-014-2).
- **Files/components:** extension/README.md; docs/DEVELOPMENT.md.
- **Acceptance:** Install/rebuild steps reproduce registration and startup in real Zed; report missing binary/grammar/API errors and record tested Zed version.
- **Completed 2026-09-13:** Documented build, test, installation (`zed: install dev extension`), verification, and troubleshooting steps in `extension/README.md` and `docs/DEVELOPMENT.md`. Recorded tested Zed version (Zed 1.19.2 on Windows 11).
- **Validation:** Documentation verified; build and install instructions match reproducible commands.

### ZK-016 — One-exercise acceptance

<a id="task-zk-016-1"></a>
#### ZK-016.1 — Accept one exercise in the real editor

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-15.
- **Depends on:** [ZK-014](#task-zk-014), [ZK-015](#task-zk-015), [ZK-008](#task-zk-008).
- **Files/components:** docs/evidence/one-exercise.md.
- **Acceptance:** Documented commands select an exercise without a TUI; manually perform correct/incorrect attempts and retry, inspect persisted verdict and visible feedback, record latency/version/date.
- **Completed 2026-09-13:** [One-exercise acceptance report](evidence/one-exercise.md) documents end-to-end verification of `delete-word-arg` using `zedkata-tui select/status/reset` in Zed 1.19.2 on Windows. Verified: (1) `dw` passing attempt emits "Par! 1 edit (score: 100)" and updates `session.json`; (2) backspaces attempt exceeding `backspace_run = 3` emits actionable failure diagnostic and fails attempt; (3) reset/reopen clears diagnostics. Sub-millisecond server dispatch latency maintained.
- **Validation:** `cargo test -p zedkata-tui` and manual scenarios in `docs/evidence/one-exercise.md` passed.

### ZK-017 — TUI shell

<a id="task-zk-017-1"></a>
#### ZK-017.1 — Build the terminal shell and restore it on exit

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-01, REQ-03, REQ-18.
- **Depends on:** [ZK-016](#task-zk-016).
- **Files/components:** crates/zedkata-tui/.
- **Acceptance:** Ratatui/crossterm shell starts and quits; manual checks cover resize, small terminal and error exit restoring terminal state. No document-editing controls are introduced.
- **Completed 2026-09-13:** Ratatui 0.26 / Crossterm 0.27 terminal shell implemented with RAII `TerminalGuard`, custom panic hook for emergency restoration, responsive layout, small-terminal (<60x15) centered warning overlay, and quit handling (`q`, `Esc`, `Ctrl-C`). Automated `TestBackend` tests verify rendering boundaries, resize transitions, and guard idempotence. Verified `--test-error` and `--test-panic` restore terminal to normal cooked mode without artifacts. Preserved headless subcommands (`select`, `status`, `reset`).
- **Validation:** `cargo test -p zedkata-tui`, `cargo clippy --workspace --locked --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and manual verification of start, quit, resize, small terminal warning, and restoration on error/panic.
<a id="task-zk-017-2"></a>
#### ZK-017.2 — Render curriculum, task, hint and progress panes

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-18, REQ-22.
- **Depends on:** [ZK-017.1](#task-zk-017-1), [ZK-009](#task-zk-009).
- **Files/components:** crates/zedkata-tui/.
- **Acceptance:** Vim-key navigation selects a lesson and exposes its task/hint; empty/locked/error states fit small terminals; manual navigation works without mouse input.
- **Completed 2026-09-13:** Integrated curriculum catalog (`Catalog`) and session state (`SessionState`) into `zedkata-tui`. Implemented vim navigation (`j`/`k` for lessons, `h`/`l` for exercises, `Enter` to activate in session, `?` to toggle hints, `r` to refresh). Rendered status markers (`[✓]`, `[ ]`, `[L]`, `[D]`). Provided locked-state view detailing missing prerequisites, task descriptions with par/forbid constraints, live attempt verdicts, completion ratios, and recent attempt history. Automated `TestBackend` tests verify navigation transitions, hint toggling, locked prerequisites, session persistence, and exact 60x15 small-terminal boundary fit.
- **Validation:** `cargo test -p zedkata-tui --test tui_shell_tests`, `cargo clippy --workspace --locked --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and manual navigation across lessons and exercises.

### ZK-018 — Exercise launch

<a id="task-zk-018-1"></a>
#### ZK-018.1 — Verify CLI positioning and specify fallback

- **Status:** done. **Priority:** MVP. **Requirements:** REQ-19.
- **Depends on:** [ZK-016](#task-zk-016).
- **Files/components:** docs/OPEN-QUESTIONS.md; docs/DEVELOPMENT.md; docs/evidence/.
- **Acceptance:** Record current CLI evidence for OQ-10 and manual path:line:column behavior including spaces; document a usable fallback if exact placement is unavailable.
- **Completed 2026-09-14:** [CLI cursor placement evidence](evidence/2026-09-14-cli-cursor-placement.md)
  records that Zed 1.19.2's own `--help` documents `path:line:column`; that a path with
  spaces in both directory and filename plus a `:3:12` suffix is accepted as one
  argument (exit 0); and that Dan observed the caret on the `T` of `TARGET` — column 12
  exactly, establishing 1-based line and column. OQ-10 verified. Fallback recorded as
  unused contingency.
- **Validation:** `zed.exe --help` and `--version` on the installed 1.19.2 binary, plus
  the manual observation above.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-018-2"></a>
#### ZK-018.2 — Launch a writable exercise copy

- **Status:** ready. **Priority:** MVP. **Requirements:** REQ-01, REQ-19.
- **Depends on:** [ZK-018.1](#task-zk-018-1), [ZK-012.3](#task-zk-012-3), [ZK-017.2](#task-zk-017-2).
- **Note 2026-09-14:** ZK-018.1 verified CLI placement, so launch may pass
  `path:line:column` (1-based). Placement applies at launch only — nothing may infer
  later cursor position from it (OQ-04).
- **Files/components:** crates/zedkata-tui/; launch tests.
- **Acceptance:** Create exercise copy in agreed data path, set active attempt, and invoke Zed with separate arguments; missing CLI/copy failure does not leave a false active state; manually verify correct fixture opens.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-019 — Live state

<a id="task-zk-019-1"></a>
#### ZK-019.1 — Poll and display session feedback

- **Status:** ready. **Priority:** MVP. **Requirements:** REQ-20.
- **Depends on:** [ZK-017.2](#task-zk-017-2), [ZK-012](#task-zk-012).
- **Files/components:** crates/zedkata-tui/.
- **Acceptance:** Controlled session updates render current attempt and progress with bounded polling; tests cover missing/partially replaced/corrupt state without crashing or busy-looping.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-019-2"></a>
#### ZK-019.2 — Show connection and stale-state recovery

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-20.
- **Depends on:** [ZK-019.1](#task-zk-019-1), [ZK-014](#task-zk-014).
- **Files/components:** crates/zedkata-tui/; docs/DEVELOPMENT.md.
- **Acceptance:** Simulate stopped LSP and stale/mismatched session; UI distinguishes waiting from success and gives an actionable recovery; confirm behavior manually after server restart.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-020 — Retry and navigation

<a id="task-zk-020-1"></a>
#### ZK-020.1 — Reset and retry an active exercise

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-13, REQ-20.
- **Depends on:** [ZK-018.2](#task-zk-018-2), [ZK-019.2](#task-zk-019-2).
- **Files/components:** crates/zedkata-tui/; session integration tests.
- **Acceptance:** Retry issues a fresh attempt, restores baseline and clears feedback; test late updates and manually check dirty-buffer reset according to the agreed contract.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-020-2"></a>
#### ZK-020.2 — Advance and switch exercises

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-20.
- **Depends on:** [ZK-020.1](#task-zk-020-1).
- **Files/components:** crates/zedkata-tui/.
- **Acceptance:** Next/selection moves to the intended exercise and updates task/progress; repeated completion and late old-buffer edits do not skip an exercise or corrupt progress.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.

### ZK-021 — Progress and resume

<a id="task-zk-021-1"></a>
#### ZK-021.1 — Display progress and unlock prerequisites

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-11, REQ-20.
- **Depends on:** [ZK-020.2](#task-zk-020-2), [ZK-012.4](#task-zk-012-4).
- **Files/components:** crates/zedkata-tui/; catalog/session tests.
- **Acceptance:** Core-derived history unlocks only satisfied dependencies; pass/failure/retry cases produce the agreed displayed score/weakness information, including unscored mode.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-021-2"></a>
#### ZK-021.2 — Resume or recover after restart

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-13, REQ-20.
- **Depends on:** [ZK-021.1](#task-zk-021-1).
- **Files/components:** crates/zedkata-tui/; docs/DEVELOPMENT.md.
- **Acceptance:** Manual quit/restart restores completion and handles unfinished attempts per contract; missing fixture and incompatible/corrupt state have an explicit recovery that does not silently overwrite recoverable progress.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-022 — Starter course

<a id="task-zk-022-1"></a>
#### ZK-022.1 — Author the first starter exercise

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-21, REQ-22.
- **Depends on:** [ZK-021](#task-zk-021), [ZK-009.3](#task-zk-009-3).
- **Files/components:** lessons/; fixture pairs.
- **Acceptance:** Reuse or refine the standalone lesson as starter exercise one; one verified concept, goal-focused task, category hint, short prose and plausible-code fixtures; catalog validation passes.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-022-2"></a>
#### ZK-022.2 — Author the second starter exercise

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-21, REQ-22.
- **Depends on:** [ZK-022.1](#task-zk-022-1).
- **Files/components:** lessons/; fixture pairs.
- **Acceptance:** A second verified concept has a complete start/goal pair and honest prerequisite on already taught material; authoring checklist and catalog checks pass.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-022-3"></a>
#### ZK-022.3 — Author and calibrate the third starter exercise

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-21, REQ-22.
- **Depends on:** [ZK-022.2](#task-zk-022-2).
- **Files/components:** lessons/; fixtures; docs/evidence/.
- **Acceptance:** Third exercise and all three manual runs are recorded; calibrate each supported budget against observed intended and alternative edits. In goal-only mode verify goals and label technique as unscored.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-023 — Local MVP acceptance

<a id="task-zk-023-1"></a>
#### ZK-023.1 — Accept the starter-course user journey

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-01, REQ-21.
- **Depends on:** [ZK-022.3](#task-zk-022-3), [ZK-021](#task-zk-021), [ZK-017](#task-zk-017), [ZK-018](#task-zk-018), [ZK-019](#task-zk-019), [ZK-020](#task-zk-020).
- **Files/components:** docs/evidence/local-mvp.md.
- **Acceptance:** From setup, select exercises, reveal hint, make a mistake, retry, finish three exercises, quit and resume with no manual session-file editing; record observed outcomes.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-023-2"></a>
#### ZK-023.2 — Check supported terminal hosting routes

- **Status:** blocked. **Priority:** MVP. **Requirements:** REQ-18.
- **Depends on:** [ZK-023.1](#task-zk-023-1).
- **Files/components:** docs/OPEN-QUESTIONS.md; docs/DEVELOPMENT.md.
- **Acceptance:** Manually check standard Zed terminal pane and assumed Terminal Thread route for fullscreen/resize/cleanup; record OQ-08 and advertise only observed supported routes.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-024 — Reference curriculum audit

<a id="task-zk-024-1"></a>
#### ZK-024.1 — Audit capabilities and settle module ordering

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-04, REQ-23.
- **Depends on:** [ZK-023](#task-zk-023).
- **Files/components:** docs/OPEN-QUESTIONS.md; curriculum module map.
- **Acceptance:** Check sources for planned capabilities (OQ-12), including missing citations in load-bearing VERIFIED entries; record sources/date, prerequisites and drill-versus-walkthrough decisions without fabricating verification.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-024-2"></a>
#### ZK-024.2 — Complete the reference text-object fixtures

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-08, REQ-10, REQ-23.
- **Depends on:** [ZK-024.1](#task-zk-024-1).
- **Files/components:** lessons/motions-03-textobjects.toml; six fixture files.
- **Acceptance:** Add all six missing start/goal files, resolve prerequisites/module number and visual_line_select against evidence; preserve the Mini/Any walkthrough as walkthrough data, not a scored exercise.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-024-3"></a>
#### ZK-024.3 — Calibrate and admit the reference lesson

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-22, REQ-23.
- **Depends on:** [ZK-024.2](#task-zk-024-2).
- **Files/components:** lessons/; docs/evidence/.
- **Acceptance:** Manually run each included exercise and observe alternative methods; record calibration, pass catalog validation and remove draft exclusion only when prerequisites are runnable.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-025 — Baseline and motions

<a id="task-zk-025-1"></a>
#### ZK-025.1 — Author the modal-editing baseline lesson

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-22, REQ-24.
- **Depends on:** [ZK-024.1](#task-zk-024-1).
- **Files/components:** lessons/; fixtures; docs/evidence/.
- **Acceptance:** Teach carry-over and verified Zed differences for existing vim users; one concept per exercise, honest dependencies, authoring checklist and manual goal/calibration records pass.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-025-2"></a>
#### ZK-025.2 — Author word and find-motion lessons

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-24.
- **Depends on:** [ZK-025.1](#task-zk-025-1).
- **Files/components:** lessons/; fixtures; docs/evidence/.
- **Acceptance:** Implement motions-01-word and motions-02-find or explicitly reconcile their IDs; each lesson has complete fixture pairs, sourced differences and recorded manual calibration.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-025-3"></a>
#### ZK-025.3 — Integrate the supported text-object sequence

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-23, REQ-24.
- **Depends on:** [ZK-025.2](#task-zk-025-2), [ZK-024.2](#task-zk-024-2).
- **Files/components:** lessons/; fixtures; docs/evidence/.
- **Acceptance:** Include verified Any/Mini concepts and the existing draft in prerequisite order; manually validate each included drill and route non-drillable behavior to walkthroughs.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-026 — Operators and keymaps

<a id="task-zk-026-1"></a>
#### ZK-026.1 — Author supported native operator drills

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-22, REQ-24.
- **Depends on:** [ZK-025](#task-zk-025).
- **Files/components:** lessons/; fixtures; docs/evidence/.
- **Acceptance:** Audit proposed register/replace/exchange and indent motions, then author/calibrate supported exercises; explicitly defer unsupported concepts rather than claiming key support.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-026-2"></a>
#### ZK-026.2 — Create keymap-context teaching material

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-24.
- **Depends on:** [ZK-026.1](#task-zk-026-1).
- **Files/components:** lessons/; walkthrough notes.
- **Acceptance:** Verify documented contexts, define a concrete configuration outcome and reproducible example; classify as walkthrough unless automatically observable; keep task wording focused on outcome.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.
<a id="task-zk-026-3"></a>
#### ZK-026.3 — Review and manually accept operator/keymap content

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-22, REQ-24.
- **Depends on:** [ZK-026.2](#task-zk-026-2).
- **Files/components:** docs/evidence/; lesson catalog.
- **Acceptance:** Run all new examples in Zed, check prerequisites and task/hint/prose conventions, record goal and any budget calibration; admit only validated content.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.

### ZK-027 — Editor features

<a id="task-zk-027-1"></a>
#### ZK-027.1 — Create LSP-backed action material

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-25.
- **Depends on:** [ZK-025](#task-zk-025).
- **Files/components:** lessons/; walkthrough notes; docs/evidence/.
- **Acceptance:** Verify rename/code-action/diagnostic examples, classify observable outcomes and create complete fixtures or a walkthrough; manually reproduce instructions and record source/version.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-027-2"></a>
#### ZK-027.2 — Create project-panel material

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-25.
- **Depends on:** [ZK-025](#task-zk-025).
- **Files/components:** lessons/; walkthrough notes; docs/evidence/.
- **Acceptance:** Audit proposed netrw-like bindings and teach verified behavior with reproducible files; record manual run and whether completion is self-reported.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-027-3"></a>
#### ZK-027.3 — Create multibuffer walkthrough content

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-25.
- **Depends on:** [ZK-027.1](#task-zk-027-1).
- **Files/components:** lessons/; walkthrough notes; docs/evidence/.
- **Acceptance:** Provide verified steps and sample files for a concrete multibuffer workflow; manual walkthrough succeeds without representing unobservable completion as a drill verdict.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-027-4"></a>
#### ZK-027.4 — Create tasks and debugger walkthrough content

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-25.
- **Depends on:** [ZK-027.1](#task-zk-027-1).
- **Files/components:** lessons/; sample task/debug configuration; docs/evidence/.
- **Acceptance:** Verify tasks.json semantics and debugger setup for a small example; run both workflows manually and record current config sources and prerequisites. Split by feature before coding if one session is insufficient.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-027-5"></a>
#### ZK-027.5 — Create agent-panel and ACP walkthrough content

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-25.
- **Depends on:** [ZK-029.1](#task-zk-029-1).
- **Files/components:** lessons/; walkthrough notes.
- **Acceptance:** Source current agent/ACP instructions and define demonstration steps plus self-reported completion; do not depend on unsupported settings or promise observation of panel interactions.
- **Validation:** Run catalog validation and the authoring checklist; perform the specified editor examples and retain calibration/source evidence.

### ZK-028 — Catalog acceptance

<a id="task-zk-028-1"></a>
#### ZK-028.1 — Enforce shipped-catalog validation in CI

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-07, REQ-08, REQ-30.
- **Depends on:** [ZK-009.2](#task-zk-009-2), [ZK-024](#task-zk-024).
- **Files/components:** crates/zedkata-core/; .github/workflows/.
- **Acceptance:** Validate every shipped lesson for schema, fixture existence, IDs, cycles, unsupported rules and draft exclusion; meaningful invalid cases fail with actionable diagnostics.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-028-2"></a>
#### ZK-028.2 — Accept the full curriculum catalog

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-22, REQ-30.
- **Depends on:** [ZK-028.1](#task-zk-028-1), [ZK-025](#task-zk-025), [ZK-026](#task-zk-026), [ZK-027](#task-zk-027).
- **Files/components:** lessons/; docs/evidence/curriculum.md.
- **Acceptance:** All included modules have sources, complete assets, honest dependencies and manual records; list unsupported/optional concepts as explicit deferrals and show catalog CI passing.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-029 — ACP decisions

<a id="task-zk-029-1"></a>
#### ZK-029.1 — Verify ACP SDK and configuration compatibility

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-03, REQ-26.
- **Depends on:** [ZK-023](#task-zk-023).
- **Files/components:** docs/OPEN-QUESTIONS.md; docs/adr/.
- **Acceptance:** Recheck OQ-07/OQ-09 from current primary sources, validate SDK/toolchain/settings compatibility, and record the provider/configuration decision needed next.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.
<a id="task-zk-029-2"></a>
#### ZK-029.2 — Choose provider, credentials and generated-data configuration

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-26.
- **Depends on:** [ZK-029.1](#task-zk-029-1), [ZK-006.5](#task-zk-006-5).
- **Files/components:** docs/adr/; docs/ARCHITECTURE.md.
- **Acceptance:** Resolve Dan's provider preference; specify configuration/credential lookup, bounded context, errors/cancellation and writable generated paths. No credentials are committed and only agent performs model calls.
- **Validation:** Review the worked contract/examples against linked requirements and record source/ADR evidence; do not promote assumptions without a check.

### ZK-030 — ACP connection

<a id="task-zk-030-1"></a>
#### ZK-030.1 — Scaffold the ACP agent protocol

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-26.
- **Depends on:** [ZK-029.2](#task-zk-029-2), [ZK-007](#task-zk-007).
- **Files/components:** crates/zedkata-agent/; protocol tests.
- **Acceptance:** Pinned agent crate builds on declared toolchain; initialize/session/shutdown and unsupported request behavior are tested with protocol-only stdout.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-030-2"></a>
#### ZK-030.2 — Connect an agent session in Zed

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-26.
- **Depends on:** [ZK-030.1](#task-zk-030-1).
- **Files/components:** docs/DEVELOPMENT.md; docs/evidence/.
- **Acceptance:** Manual agent-panel connection works with documented settings; invalid binary/config gives actionable errors; record Zed/SDK versions.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-031 — Attempt explanation

<a id="task-zk-031-1"></a>
#### ZK-031.1 — Build bounded last-attempt context

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-27.
- **Depends on:** [ZK-030](#task-zk-030), [ZK-012](#task-zk-012).
- **Files/components:** crates/zedkata-agent/; context tests.
- **Acceptance:** Read session and edit history within agreed limits; fake provider receives correct attempt/context and handles missing, old, corrupt or oversized state explicitly.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-031-2"></a>
#### ZK-031.2 — Explain an attempt using the configured provider

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-27.
- **Depends on:** [ZK-031.1](#task-zk-031-1).
- **Files/components:** crates/zedkata-agent/; prompts; docs/evidence/.
- **Acceptance:** Manual explanation is grounded in captured edits and separates observation from technique inference; configured-provider call works without network access from core/LSP/TUI.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.
<a id="task-zk-031-3"></a>
#### ZK-031.3 — Handle model failure and cancellation

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-27.
- **Depends on:** [ZK-031.2](#task-zk-031-2).
- **Files/components:** crates/zedkata-agent/; fake-provider tests.
- **Acceptance:** Timeout, provider rejection, malformed response and cancellation leave session/progress intact and produce clear messages; no unbounded retries or lingering request tasks.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.

### ZK-032 — Generated practice

<a id="task-zk-032-1"></a>
#### ZK-032.1 — Generate a complete candidate exercise

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-28.
- **Depends on:** [ZK-031](#task-zk-031), [ZK-011.3](#task-zk-011-3).
- **Files/components:** crates/zedkata-agent/.
- **Acceptance:** Use agreed weakness context to request lesson metadata and both fixtures; missing/unscored weakness data follows the contract; keep output a candidate until validation passes.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-032-2"></a>
#### ZK-032.2 — Validate and store generated assets

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-28.
- **Depends on:** [ZK-032.1](#task-zk-032-1), [ZK-028.1](#task-zk-028-1).
- **Files/components:** crates/zedkata-agent/; core catalog helpers.
- **Acceptance:** Reject malformed TOML/rules, missing fixtures and paths escaping the generated content root; ID collisions preserve existing lessons; valid assets write together to the configured user-data directory.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-032-3"></a>
#### ZK-032.3 — Select and complete generated practice

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-28.
- **Depends on:** [ZK-032.2](#task-zk-032-2), [ZK-021](#task-zk-021).
- **Files/components:** crates/zedkata-tui/; docs/evidence/.
- **Acceptance:** Generated catalog discovery works outside the repo; manually select and complete a generated exercise; uncalibrated technique budgets are not presented as validated.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-033 — Walkthroughs

<a id="task-zk-033-1"></a>
#### ZK-033.1 — Implement walkthrough selection and progress

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-23, REQ-29.
- **Depends on:** [ZK-030](#task-zk-030), [ZK-027.3](#task-zk-027-3), [ZK-027.4](#task-zk-027-4), [ZK-027.5](#task-zk-027-5), [ZK-024.2](#task-zk-024-2).
- **Files/components:** crates/zedkata-agent/; walkthrough data.
- **Acceptance:** Load walkthrough records, present verified steps and preserve explicit self-reported completion semantics; include Mini/Any and feature walkthroughs without entering drill scoring.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-033-2"></a>
#### ZK-033.2 — Accept the walkthrough user journey

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-29.
- **Depends on:** [ZK-033.1](#task-zk-033-1), [ZK-031](#task-zk-031).
- **Files/components:** docs/evidence/.
- **Acceptance:** Manually complete a walkthrough with the agent, restart according to its contract, and verify prompts do not claim unseen editor actions were observed.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-034 — Installed assets

<a id="task-zk-034-1"></a>
#### ZK-034.1 — Assemble binaries and external lesson assets

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-08, REQ-31.
- **Depends on:** [ZK-028](#task-zk-028), [ZK-032](#task-zk-032), [ZK-033](#task-zk-033).
- **Files/components:** packaging scripts; release staging layout.
- **Acceptance:** Staged install includes versioned native binaries and complete external lesson data; changing lesson files needs no recompile and optional agent setup does not block local drills.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-034-2"></a>
#### ZK-034.2 — Verify installed-path discovery and writes

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-31.
- **Depends on:** [ZK-034.1](#task-zk-034-1).
- **Files/components:** packaging checks; docs/DEVELOPMENT.md.
- **Acceptance:** Run from outside source checkout and from paths with spaces; read packaged lessons and write state/exercises/generated content only to agreed writable paths; read-only install directory works.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.

### ZK-035 — Platform validation

<a id="task-zk-035-1"></a>
#### ZK-035.1 — Add macOS and Linux native builds

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-30, REQ-32.
- **Depends on:** [ZK-008](#task-zk-008), [ZK-034](#task-zk-034).
- **Files/components:** .github/workflows/; packaging scripts.
- **Acceptance:** Locked native build/test jobs produce artifacts for declared targets alongside Windows; record actual results and keep failed/untested targets out of verified support claims.
- **Validation:** Run the named local build/check jobs and record CI results; keep unavailable hosted/platform validation explicitly pending.
<a id="task-zk-035-2"></a>
#### ZK-035.2 — Add separate extension and compatibility checks

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-30, REQ-32.
- **Depends on:** [ZK-015](#task-zk-015), [ZK-035.1](#task-zk-035-1).
- **Files/components:** .github/workflows/; extension/README.md.
- **Acceptance:** WASM extension check runs separately from native MSRV jobs; release metadata states tested native/extension/Zed combinations and binary lookup expectations.
- **Validation:** Run the named local build/check jobs and record CI results; keep unavailable hosted/platform validation explicitly pending.
<a id="task-zk-035-3"></a>
#### ZK-035.3 — Run platform smoke-test matrix

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-32.
- **Depends on:** [ZK-035.2](#task-zk-035-2).
- **Files/components:** docs/evidence/platforms.md.
- **Acceptance:** On each claimed platform, install/start/open/edit/feedback/retry/quit checks are recorded with versions; unavailable machines stay explicitly unverified.
- **Validation:** Run the named local build/check jobs and record CI results; keep unavailable hosted/platform validation explicitly pending.

### ZK-036 — Clean install

<a id="task-zk-036-1"></a>
#### ZK-036.1 — Write the complete install and troubleshooting guide

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-33.
- **Depends on:** [ZK-034](#task-zk-034), [ZK-035.2](#task-zk-035-2).
- **Files/components:** README.md; docs/installation.md.
- **Acceptance:** Guide covers prerequisites, CLI, extension, external lessons, state location, optional provider setup and common failure recovery with exact commands.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-036-2"></a>
#### ZK-036.2 — Run a clean-environment first-pass check

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-33.
- **Depends on:** [ZK-036.1](#task-zk-036-1), [ZK-035.3](#task-zk-035-3).
- **Files/components:** docs/evidence/clean-install.md.
- **Acceptance:** Fresh supported environment reaches first passing exercise without undocumented steps; record mistakes discovered and retest only corrected steps.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-036-3"></a>
#### ZK-036.3 — Check update and uninstall behavior

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-33.
- **Depends on:** [ZK-036.2](#task-zk-036-2).
- **Files/components:** docs/installation.md; packaging checks.
- **Acceptance:** Document version/schema incompatibility recovery and distinguish application removal from intentional progress removal; manually verify update and uninstall on a supported platform.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-037 — Release preparation

<a id="task-zk-037-1"></a>
#### ZK-037.1 — Prepare license, versions and release notes

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-04, REQ-34.
- **Depends on:** [ZK-036](#task-zk-036).
- **Files/components:** license file; release notes; manifests.
- **Acceptance:** Choose license explicitly, reconcile artifact versions and document known limits/support evidence; no untested behavior advertised as complete.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-037-2"></a>
#### ZK-037.2 — Prepare extension submission artifacts

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-34.
- **Depends on:** [ZK-037.1](#task-zk-037-1), [ZK-035.2](#task-zk-035-2).
- **Files/components:** extension metadata; submission draft.
- **Acceptance:** Check current submission requirements, metadata and binary availability; produce a reviewable patch/draft with build and install evidence, without publishing.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
<a id="task-zk-037-3"></a>
#### ZK-037.3 — Review the release candidate

- **Status:** blocked. **Priority:** full release. **Requirements:** REQ-34.
- **Depends on:** [ZK-037.2](#task-zk-037-2).
- **Files/components:** release checklist; docs/evidence/.
- **Acceptance:** Required tasks are done or explicitly scoped out by recorded decisions; artifacts, platform results and clean-install evidence agree. Publication remains a separate explicitly authorized action.
- **Validation:** Run the concrete manual scenarios in Acceptance, retain observations, and run affected automated checks where implementation changes.

### ZK-038 — Optional inline hints

<a id="task-zk-038-1"></a>
#### ZK-038.1 — Optionally display inline task hints

- **Status:** deferred. **Priority:** optional. **Requirements:** REQ-16.
- **Depends on:** [ZK-023](#task-zk-023).
- **Files/components:** crates/zedkata-lsp/; protocol and manual checks.
- **Acceptance:** Deferred beyond MVP unless selected; if implemented, inlayHint returns the active task, refreshes/clears across attempts and renders in Zed without breaking diagnostics.
- **Validation:** Exercise the success/failure cases in Acceptance; run affected tests plus formatting/clippy for Rust changes or npm tests for probe changes, and retain the result.
