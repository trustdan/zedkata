# Plan: complete the implementation backlog

Status: executed 2026-09-13. Deliverables are [requirements coverage](../REQUIREMENTS.md),
the [expanded backlog](../BACKLOG.md), and the updated [build overview](2026-09-13-build-plan.md).
Completion history and the pending ZK-003 editor check were preserved. The text below
records the approved method; implementation of the product remains separate work.

Validation completed against the written files: 35 unique requirement groups, 95
executable leaf tasks, 131 unique task anchors, resolving task references, no dependency
cycles, and all production tasks transitively gated by ZK-005. Local documentation
links resolve. ZK-001/002 completion and ZK-003's pending manual status are preserved.

## Outcome

Expand the existing [backlog](../BACKLOG.md) into a complete, source-linked sequence
of small tasks covering the documented product from feasibility through release.
This is a plan for producing that backlog, not permission to skip gates or a claim
that the implementation is complete.

Keep the [build plan](2026-09-13-build-plan.md) as the milestone overview. Keep
`docs/BACKLOG.md` as the single task-status source of truth. Avoid a second competing
task list or an external tracker unless requested.

## Preserve the current baseline

- ZK-001: complete; local tooling and the documented development route recorded.
- ZK-002: complete; disposable probe and synthetic checks implemented.
- ZK-003: implementation prepared; real-editor install/open/edit/close and visible
  diagnostic checks still pending. Its checkbox must remain unchecked.
- ZK-004–037: pending. Preserve existing IDs, evidence, and completion notes.
- Production scaffolding still depends on the recorded Phase 0 decision.

## 1. Extract requirements from the repository

Read each source below and turn every required behavior, constraint, unresolved
decision, and explicit exclusion into an entry in `docs/REQUIREMENTS.md`.

| Source | Extract |
|--------|---------|
| [README](../../README.md) | Product promise, audience, components, first-run experience |
| [Working rules](../../CLAUDE.md) | Architecture restrictions, stack conventions, evidence discipline, phase gate |
| [Architecture](../ARCHITECTURE.md) | Component responsibilities, state fields, lesson fields, feedback loop |
| [Open questions](../OPEN-QUESTIONS.md) | All OQ-01–12 dependencies and required evidence |
| [ADRs](../adr/) | Accepted constraints and decisions requiring explicit revision |
| [Build plan](2026-09-13-build-plan.md) | Milestones, intended layout, lifecycle risks, MVP boundary |
| [Development setup](../DEVELOPMENT.md) | Observed tooling, actual build status, remaining manual checks |
| [Specialist instructions](../../.claude/agents/) | Curriculum calibration, protocol evidence, Rust boundaries |
| [Draft lesson](../../lessons/motions-03-textobjects.toml) | Missing fixtures/prerequisites, walkthrough fields, unsupported-rule questions |
| [Probe instructions](../../spikes/lsp-probe/README.md) | Implemented scope and measurement limitations |
| [Extension checklist](../../spikes/lsp-probe/extension/README.md) | Pending editor validation and cleanup |

Assign requirement IDs such as `REQ-CORE-01`. Record source section, requirement,
required/conditional/optional/excluded classification, implementing task IDs, and
validation evidence. New engineering work inferred from a requirement must be labeled
as inferred rather than attributed to the source as an explicit instruction.

**Output:** a coverage matrix where every documented requirement has an owner task
or a reason for deferral/exclusion.

## 2. Expose unresolved contracts before implementation tasks

Split the broad ZK-006 contract task into decision-sized subtasks. Do not silently
settle these issues while writing the backlog:

- Which process writes each state field, including core-computed history and weaknesses?
  How do competing updates behave under the current single-file/last-writer-wins rule?
- How does the LSP learn the active lesson and reload/reset it when the TUI navigates?
- When is an attempt unfinished, passed, failed, abandoned, retried, or resumed?
  How are late edits and duplicate completion handled?
- What are the lesson schema version, fixture path base, cursor-start representation,
  newline policy, position encoding, and walkthrough representation?
- What exactly do `par_edits`, `max_edits`, forbidden patterns, scores, and weakness
  values mean? Which survive a goal-only feasibility result?
- How do packaged and generated lessons share validation and catalog discovery?
  Reconcile `lessons/generated/` with writable user data paths for installed builds.
- Does the documented process count accurately distinguish Zed, native binaries,
  the core library, and the WASM extension? Clarify terminology without adding processes.
- How do the native minimum Rust version and extension/ACP dependency requirements
  coexist? The spike's modern compiler does not establish native MSRV compatibility.

**Output:** explicit decision tasks, each with affected requirements, blocked task IDs,
and required architecture/OQ/ADR updates. Preserve accepted constraints until revised.

## 3. Decompose the work by milestone

Use the following coverage checklist to expand existing tasks. These are areas to
decompose, not promises to add every optional feature to the MVP.

| Area / existing tasks | Required task coverage |
|-----------------------|------------------------|
| Feasibility / ZK-003–005 | Installation, captured lifecycle, repeatable edit matrix, encoding, undo, measurement overhead, visible latency, comparison of equivalent techniques, gate ADR, retained evidence, removal of disposable registration/code |
| Contracts / ZK-006 | State ownership, lifecycle, schemas, paths, failure semantics, scoring definitions, decision records |
| Scaffold / ZK-007–008 | Workspace boundaries, toolchain/dependency pins, CLI help/errors, native checks, separate WASM checks, CI parity with local commands |
| Lessons / ZK-009, 028 | Typed parsing, schema errors, fixture loading, ID uniqueness, prerequisite resolution/cycles, draft exclusion, generated-content validation, shipped catalog checks |
| Verification / ZK-010–011 | Goal comparison, newline policy, attempt finalization, conditional edit counting and forbid rules, undo policy, score formula, regression captures |
| Persistence / ZK-012 | Platform paths, versioned serialization, updates, Windows replacement, corrupt/incompatible data, retry/reset/resume, duplicate completion, competing writers, history and weakness updates |
| LSP / ZK-013–014 | Initialize/shutdown/exit, document identity/version tracking, encoding-aware replay, multi-change/full replacement, active exercise resolution, stale events, diagnostics publish/clear, unimplemented requests, failure reporting |
| Extension / ZK-015–016 | Production registration, grammar strategy, binary discovery, install/rebuild loop, missing binary recovery, one-exercise manual acceptance |
| TUI / ZK-017–021 | Layout and navigation, task/hint display, terminal cleanup/resize, fixture launch, CLI errors, polling/health, retry/next, progress/unlocking, restart and dirty-buffer behavior |
| MVP / ZK-022–023 | Three standalone complete exercises, manual calibration, user-flow acceptance, supported terminal routes, mistake/retry/restart demonstration |
| Curriculum / ZK-024–028 | Capability audit, module numbering, prerequisites, individual lesson/fixture authoring, calibration records, non-drillable walkthrough outlines, catalog validation |
| Agent / ZK-029–033 | SDK/provider decision, credentials/configuration, ACP lifecycle, bounded attempt context, explanation fidelity, model errors/cancellation, output validation, generated fixture pairs/catalog inclusion, walkthrough completion |
| Distribution / ZK-034–037 | Asset layout, binaries and extension compatibility, writable installed paths, platform CI/smoke matrix, clean install/update/uninstall, license/version/notes, reviewable publication artifacts |

Give optional inlay hints an explicit deferred task/disposition. Do not let the
architecture's optional feature disappear from coverage or become an MVP blocker.

For curriculum, separate baseline, word/find motions, text objects, native operators,
keymap contexts, LSP actions, project panel, multibuffer, tasks, debugger, and agent/ACP
walkthroughs. Decide which are drills from their observable outcomes. Do not invent
exercises for unverified capabilities. Each lesson needs fixtures, dependencies,
authoring review, and calibration where technique scoring applies.

**Output:** one reviewable behavior or decision per task. Split work that independently
needs design, implementation, and manual acceptance; avoid tasks such as “build the
entire agent” or “finish all curriculum.”

## 4. Standardize task cards and dependencies

Retain ZK-001–037 as stable IDs. Where necessary, add child tasks such as ZK-012.1,
ZK-012.2 under the existing parent. A parent completes only when all required children
complete. Optional/skipped children require a recorded reason and do not falsely
count as implemented. Allocate new top-level IDs only for work with no existing home.

Use this compact format:

```text
ID and title:
Status: ready / blocked / in progress / manual check / done / skipped / deferred
Milestone and priority: gate / MVP / full release / optional
Source requirements and OQ/ADR references:
Depends on: explicit task IDs, or none
Scope: one observable outcome
Expected files/components:
Acceptance: observable pass conditions, including relevant failure behavior
Validation: automated command/cases and/or exact manual steps
Evidence needed: artifacts, versions, observations
Completion notes and next task:
```

Dependency rules:

- Build an acyclic graph; numeric order alone is not a dependency specification.
- Put production tasks behind ZK-005 and contract-dependent work behind its decisions.
- Conditional scoring tasks name their gate result and skip path.
- Keep documentation/backlog refinement possible while manual ZK-003 checks are pending.
- Permit ACP work after the local MVP without waiting for unrelated curriculum tasks;
  walkthrough implementation still depends on its content.
- Distinguish the local MVP from full distribution, so the first useful version does
  not wait for every module, model integration, or release platform.
- List one next ready task plus the exact blocker for tasks waiting on Dan or evidence.

## 5. Reconcile documents and review coverage

Update the backlog and build-plan milestone references together. Record contradictory
statements as decision work; change architectural claims only after the corresponding
decision/evidence exists. Recheck current external documentation only for tasks that
need it; backlog preparation must not promote assumptions to VERIFIED.

Review the finished backlog against these user journeys:

1. Install tools and connect the feasibility probe; record a supported or fallback verdict.
2. Build and start one lesson; edit correctly and incorrectly; see feedback.
3. Select, get a hint, retry, advance, quit, and resume the starter course.
4. Recover from missing tools, malformed lesson/state, stale LSP, and interrupted attempts.
5. Explain an attempt, generate valid practice, and complete a guided walkthrough.
6. Install a release outside the repo, find lesson assets, update, and uninstall.

**Validation:** unique IDs, resolving references, explicit dependencies without cycles,
coverage for OQ-01–12 and each architecture responsibility, conditional branches with
skip reasons, preserved completion history, and an acceptance check on every leaf task.
Use a one-off consistency check rather than building a task-management application.

## Deliverables and completion criteria

- `docs/REQUIREMENTS.md`: source-to-task coverage matrix and explicit exclusions.
- `docs/BACKLOG.md`: expanded ordered task cards, preserved history, dependency IDs,
  milestone boundaries, and next-action status.
- `docs/plans/2026-09-13-build-plan.md`: refreshed overview linking to the expanded tasks.
- Architecture/OQ/ADR changes only where justified; unresolved conflicts remain visible.

The backlog is complete when every requirement has a disposition and every required
implementation outcome has a testable leaf task. Do not target an arbitrary task count
or invent deadlines. “Complete” means coverage of the documented scope with explicit
unknowns, not pretending that future feasibility results are already known.

Execution order for creating these deliverables: source inventory → coverage matrix →
decision tasks → task decomposition → dependencies → consistency and journey review.
No new production code or external issue publication is part of this planning work.
