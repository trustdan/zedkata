# Requirements and coverage

This matrix traces the documented scope to the [implementation backlog](BACKLOG.md).
It records coverage, not completion; task status lives only in the backlog. Checked
against repository documents on 2026-09-13; no external capability was re-verified
by this documentation work.

## Source index

| Label | Source and relevant sections |
|-------|------------------------------|
| README | [Product overview](../README.md): The idea, Architecture, Planned crates |
| CLAUDE | [Working rules](../CLAUDE.md): Hard rules, Stack, Evidence discipline, Scope |
| ARCHITECTURE | [Architecture](ARCHITECTURE.md): Components, State contract, Lesson format, Feedback loop |
| OQ | [Open questions](OPEN-QUESTIONS.md): OQ-01–12 |
| ADR 0001 | [Record decisions](adr/0001-record-architecture-decisions.md) |
| ADR 0002 | [Rust over Go](adr/0002-rust-over-go.md) |
| Build plan | [Milestones and contracts](plans/2026-09-13-build-plan.md) |
| BACKLOG | [Existing roadmap and preserved task history](BACKLOG.md) |
| DEVELOPMENT | [Local tooling and setup evidence](DEVELOPMENT.md) |
| curriculum-author | [Authoring and calibration rules](../.claude/agents/curriculum-author.md) |
| rust-console | [Rust boundaries and tests](../.claude/agents/rust-console.md) |
| lsp-probe | [Observation matrix and retained evidence](../.claude/agents/lsp-probe.md) |
| zed-docs-scout | [Source verification rules](../.claude/agents/zed-docs-scout.md) |
| Draft lesson | [Text-object reference](../lessons/motions-03-textobjects.toml) |
| Probe README | [Capture behavior and limits](../spikes/lsp-probe/README.md) |
| Extension README | [Pending manual checks](../spikes/lsp-probe/extension/README.md) |

## Requirement matrix

Required means required by the documented scope, not necessarily by the first MVP.
Conditional technique features follow the Phase 0 ADR. Inferred details are engineering
work needed to deliver a stated outcome; they are not newly discovered source claims.
Acceptance evidence below is **needed evidence**, unless already recorded by a completed
task. Every task also inherits the working rules and exclusions.

| ID | Classification | Source | Requirement / disposition | Owning tasks | Evidence needed |
|----|----------------|--------|---------------------------|--------------|-----------------|
| REQ-01 | Required | README: The idea; CLAUDE: Hard rules | Real Zed buffers; terminal TUI conducts drills, never edits documents; no web UI | [ZK-017.1](BACKLOG.md#task-zk-017-1) [ZK-018.2](BACKLOG.md#task-zk-018-2) [ZK-023.1](BACKLOG.md#task-zk-023-1) | Manual three-exercise journey |
| REQ-02 | Required | CLAUDE: Hard rules; ARCHITECTURE: Components | Single shared session file; no direct interprocess coupling, no network in LSP/TUI; clarify process/library terminology | [ZK-006.1](BACKLOG.md#task-zk-006-1) [ZK-007.1](BACKLOG.md#task-zk-007-1) [ZK-014.1](BACKLOG.md#task-zk-014-1) | Contract review and dependency review |
| REQ-03 | Required | CLAUDE: Stack and conventions; ADR 0002; rust-console | Rust shipped components, declared MSRV/edition, sync core, Tokio boundaries, clap, ratatui/crossterm, thiserror/anyhow and panic conventions | [ZK-007.1](BACKLOG.md#task-zk-007-1) [ZK-007.2](BACKLOG.md#task-zk-007-2) [ZK-017.1](BACKLOG.md#task-zk-017-1) [ZK-029.1](BACKLOG.md#task-zk-029-1) | Toolchain build and code review |
| REQ-04 | Required | CLAUDE: Evidence discipline and How to work here; ADR 0001; zed-docs-scout | Date/source Zed claims, distinguish assumptions, record ADRs, retain superseded decisions, plan broad changes | [ZK-005.1](BACKLOG.md#task-zk-005-1) [ZK-006.1](BACKLOG.md#task-zk-006-1) [ZK-024.1](BACKLOG.md#task-zk-024-1) [ZK-029.1](BACKLOG.md#task-zk-029-1) | Dated evidence and decision records; applies to every task |
| REQ-05 | Required | BACKLOG: Phase 0; lsp-probe; probe README; DEVELOPMENT | Disposable captured stdio probe, local registration, open/change/close, repeated edit comparisons and visible latency measurement | [ZK-001](BACKLOG.md#task-zk-001) [ZK-002](BACKLOG.md#task-zk-002) [ZK-003.2](BACKLOG.md#task-zk-003-2) [ZK-003.3](BACKLOG.md#task-zk-003-3) [ZK-004.1](BACKLOG.md#task-zk-004-1) [ZK-004.2](BACKLOG.md#task-zk-004-2) [ZK-004.3](BACKLOG.md#task-zk-004-3) | Raw captures plus manual observations, never synthetic evidence |
| REQ-06 | Required | CLAUDE: Phase 0; BACKLOG: Gate; ADR 0002 | Gate production on observation; goal-only or slower-feedback fallback; retain evidence then retire spike and dev registration | [ZK-005.1](BACKLOG.md#task-zk-005-1) [ZK-005.2](BACKLOG.md#task-zk-005-2) | Gate ADR and cleanup inventory |
| REQ-07 | Required | ARCHITECTURE: Lesson format; draft lesson; build plan | Typed TOML lesson/exercise/walkthrough schema, IDs, prerequisites, prose, concepts, fixture paths and compatibility rules | [ZK-006.3](BACKLOG.md#task-zk-006-3) [ZK-009.1](BACKLOG.md#task-zk-009-1) [ZK-009.2](BACKLOG.md#task-zk-009-2) [ZK-028.1](BACKLOG.md#task-zk-028-1) | Valid and invalid catalog cases |
| REQ-08 | Required | ARCHITECTURE: core and Lesson format; curriculum-author | Complete plausible-code fixture pairs and honest prerequisite graph; external lesson data, drafts excluded from shipped catalog | [ZK-009.2](BACKLOG.md#task-zk-009-2) [ZK-009.3](BACKLOG.md#task-zk-009-3) [ZK-024.2](BACKLOG.md#task-zk-024-2) [ZK-028.1](BACKLOG.md#task-zk-028-1) [ZK-034.1](BACKLOG.md#task-zk-034-1) | Fixture loading and catalog validation |
| REQ-09 | Required | ARCHITECTURE: verification; build plan: Attempt lifecycle | Goal matching and unfinished/finalized outcomes; encoding/newline/whitespace policy | [ZK-006.2](BACKLOG.md#task-zk-006-2) [ZK-006.3](BACKLOG.md#task-zk-006-3) [ZK-010.1](BACKLOG.md#task-zk-010-1) [ZK-010.2](BACKLOG.md#task-zk-010-2) | Empty/Unicode/CRLF and lifecycle tests |
| REQ-10 | Conditional | ARCHITECTURE: par_edits/forbid; OQ-01,11; draft lesson | Evidence-supported edit count, budget, forbidden patterns and undo policy; no claim to observe keystrokes/selections | [ZK-006.4](BACKLOG.md#task-zk-006-4) [ZK-011.1](BACKLOG.md#task-zk-011-1) [ZK-011.2](BACKLOG.md#task-zk-011-2) [ZK-024.2](BACKLOG.md#task-zk-024-2) | Captured alternatives and gate-defined skip path |
| REQ-11 | Required / conditional technique component | ARCHITECTURE: core and State contract | Core owns verdict, history, scoring and weaknesses; specify formulas and goal-only representation instead of inventing them | [ZK-006.4](BACKLOG.md#task-zk-006-4) [ZK-011.3](BACKLOG.md#task-zk-011-3) [ZK-012.4](BACKLOG.md#task-zk-012-4) [ZK-021.1](BACKLOG.md#task-zk-021-1) | Formula examples and repeated-attempt tests |
| REQ-12 | Required | ARCHITECTURE: State contract; build plan | Versioned student/current/attempt/history/weaknesses state, platform paths and temp-file replacement; explicit competing-write limitations | [ZK-006.1](BACKLOG.md#task-zk-006-1) [ZK-006.5](BACKLOG.md#task-zk-006-5) [ZK-012.1](BACKLOG.md#task-zk-012-1) [ZK-012.2](BACKLOG.md#task-zk-012-2) [ZK-012.3](BACKLOG.md#task-zk-012-3) | Windows persistence and competing-update cases |
| REQ-13 | Required; recovery details inferred | Build plan: lifecycle and Validation; ARCHITECTURE: current/attempt | Attempt identity, switch/retry/resume, stale updates, corrupt/incompatible state and duplicate completion | [ZK-006.2](BACKLOG.md#task-zk-006-2) [ZK-012.3](BACKLOG.md#task-zk-012-3) [ZK-012.4](BACKLOG.md#task-zk-012-4) [ZK-014.1](BACKLOG.md#task-zk-014-1) [ZK-020.1](BACKLOG.md#task-zk-020-1) [ZK-021.2](BACKLOG.md#task-zk-021-2) | Transition tests and manual interrupted-session checks |
| REQ-14 | Required | ARCHITECTURE: LSP; rust-console: Testing | Stdio lifecycle, baseline and ordered changes, encoding-aware replay, version tracking, retained capture integration tests, correct empty/unsupported feature responses | [ZK-013.1](BACKLOG.md#task-zk-013-1) [ZK-013.2](BACKLOG.md#task-zk-013-2) [ZK-013.3](BACKLOG.md#task-zk-013-3) | Protocol fixture replay and lifecycle tests |
| REQ-15 | Required | ARCHITECTURE: LSP and Feedback loop; OQ-03 | Verification/state integration, publish and clear diagnostics, target <100 ms with 300 ms feedback-channel gate | [ZK-014.1](BACKLOG.md#task-zk-014-1) [ZK-014.2](BACKLOG.md#task-zk-014-2) [ZK-016.1](BACKLOG.md#task-zk-016-1) [ZK-004.3](BACKLOG.md#task-zk-004-3) | Capture assertions and measured visible feedback |
| REQ-16 | Optional | ARCHITECTURE: LSP | Inline task display using inlay hints | [ZK-038.1](BACKLOG.md#task-zk-038-1) | Deferred beyond MVP; protocol and visual check if selected |
| REQ-17 | Required | ARCHITECTURE: ext; OQ-02,06; DEVELOPMENT; extension README | WASM extension registration, grammar strategy, server binary discovery, developer installation and troubleshooting | [ZK-003.1](BACKLOG.md#task-zk-003-1) [ZK-003.2](BACKLOG.md#task-zk-003-2) [ZK-015.1](BACKLOG.md#task-zk-015-1) [ZK-015.2](BACKLOG.md#task-zk-015-2) [ZK-015.3](BACKLOG.md#task-zk-015-3) | WASM build plus manual load and startup |
| REQ-18 | Required | ARCHITECTURE: TUI; BACKLOG: Phase 2; OQ-08 | Lesson/task/feedback/progress panes, vim navigation, hint display, terminal cleanup/resize, supported terminal route | [ZK-017.1](BACKLOG.md#task-zk-017-1) [ZK-017.2](BACKLOG.md#task-zk-017-2) [ZK-023.2](BACKLOG.md#task-zk-023-2) | Manual terminal operation and restoration |
| REQ-19 | Required | ARCHITECTURE: TUI; OQ-10; build plan | Launch fixture at agreed position through Zed CLI; paths with spaces, writable exercise files, missing CLI recovery | [ZK-018.1](BACKLOG.md#task-zk-018-1) [ZK-018.2](BACKLOG.md#task-zk-018-2) | CLI source evidence and manual launch |
| REQ-20 | Required; health handling inferred | ARCHITECTURE: TUI; build plan; BACKLOG ZK-019–021 | Polling, stale/missing state feedback, retry/next, progress unlocking and resume without hand-editing state | [ZK-019.1](BACKLOG.md#task-zk-019-1) [ZK-019.2](BACKLOG.md#task-zk-019-2) [ZK-020.1](BACKLOG.md#task-zk-020-1) [ZK-020.2](BACKLOG.md#task-zk-020-2) [ZK-021.1](BACKLOG.md#task-zk-021-1) [ZK-021.2](BACKLOG.md#task-zk-021-2) | Simulated updates and manual restart |
| REQ-21 | Required | Build plan: First useful release; BACKLOG ZK-022–023 | Three calibrated exercises form first local MVP; mistake, hint, retry, advance and resume demonstration | [ZK-022.1](BACKLOG.md#task-zk-022-1) [ZK-022.2](BACKLOG.md#task-zk-022-2) [ZK-022.3](BACKLOG.md#task-zk-022-3) [ZK-023.1](BACKLOG.md#task-zk-023-1) | Recorded three-exercise user journey |
| REQ-22 | Required | curriculum-author: Audience, Exercise design, Calibration | Existing modal-editing audience; one concept per exercise; goal-only tasks, category hints, short prose, manual calibration and no padding | [ZK-022.1](BACKLOG.md#task-zk-022-1) [ZK-024.3](BACKLOG.md#task-zk-024-3) [ZK-025.1](BACKLOG.md#task-zk-025-1) [ZK-026.1](BACKLOG.md#task-zk-026-1) [ZK-028.2](BACKLOG.md#task-zk-028-2) | Per-lesson authoring checklist and observed runs |
| REQ-23 | Required | Draft lesson; OQ-12; BACKLOG Phase 3 | Resolve module ordering, missing prerequisites and six fixtures, visual_line_select and Mini/Any walkthrough; audit capabilities before authoring | [ZK-024.1](BACKLOG.md#task-zk-024-1) [ZK-024.2](BACKLOG.md#task-zk-024-2) [ZK-024.3](BACKLOG.md#task-zk-024-3) [ZK-025.3](BACKLOG.md#task-zk-025-3) [ZK-033.1](BACKLOG.md#task-zk-033-1) | Reconciled module map and calibrated text-object draft |
| REQ-24 | Required; individual capabilities conditional | BACKLOG Phase 3; curriculum-author | Baseline, word/find/text-object, native operator and keymap curriculum, with honest dependencies | [ZK-025.1](BACKLOG.md#task-zk-025-1) [ZK-025.2](BACKLOG.md#task-zk-025-2) [ZK-025.3](BACKLOG.md#task-zk-025-3) [ZK-026.1](BACKLOG.md#task-zk-026-1) [ZK-026.2](BACKLOG.md#task-zk-026-2) [ZK-026.3](BACKLOG.md#task-zk-026-3) | Verified sources, complete fixtures, per-lesson calibration |
| REQ-25 | Required; drillability conditional | README; ARCHITECTURE: agent; BACKLOG Phase 3 | LSP actions, project panel, multibuffer, tasks, debugger, agent panel and ACP; non-observable work is a walkthrough | [ZK-027.1](BACKLOG.md#task-zk-027-1) [ZK-027.2](BACKLOG.md#task-zk-027-2) [ZK-027.3](BACKLOG.md#task-zk-027-3) [ZK-027.4](BACKLOG.md#task-zk-027-4) [ZK-027.5](BACKLOG.md#task-zk-027-5) [ZK-033.1](BACKLOG.md#task-zk-033-1) | Outcome classification and manual walkthrough records |
| REQ-26 | Required | ARCHITECTURE: agent; OQ-07,09; BACKLOG ZK-029–030 | ACP agent crate, compatible SDK/settings, model/provider and credential configuration | [ZK-029.1](BACKLOG.md#task-zk-029-1) [ZK-029.2](BACKLOG.md#task-zk-029-2) [ZK-030.1](BACKLOG.md#task-zk-030-1) [ZK-030.2](BACKLOG.md#task-zk-030-2) | Provider ADR, protocol test, manual agent-panel connection |
| REQ-27 | Required; limits/failure handling inferred | ARCHITECTURE: agent; build plan; BACKLOG ZK-031 | Explain last attempt using session/edit context, distinguish observation from inference, bound context, handle model errors/cancellation | [ZK-031.1](BACKLOG.md#task-zk-031-1) [ZK-031.2](BACKLOG.md#task-zk-031-2) [ZK-031.3](BACKLOG.md#task-zk-031-3) | Fake-provider cases and manual grounded explanation |
| REQ-28 | Required | ARCHITECTURE: agent; build plan: generated data; BACKLOG ZK-032 | Generate practice targeting weaknesses; validate lesson plus fixtures, store in writable data path and include in catalog | [ZK-006.5](BACKLOG.md#task-zk-006-5) [ZK-032.1](BACKLOG.md#task-zk-032-1) [ZK-032.2](BACKLOG.md#task-zk-032-2) [ZK-032.3](BACKLOG.md#task-zk-032-3) | Malformed-output rejection and generated-exercise completion |
| REQ-29 | Required | ARCHITECTURE: agent; draft walkthrough; curriculum-author | Guided walkthroughs use verified instructions and self-reported completion, not scored drill claims | [ZK-033.1](BACKLOG.md#task-zk-033-1) [ZK-033.2](BACKLOG.md#task-zk-033-2) | Manual walkthrough with explicit completion semantics |
| REQ-30 | Required; workflow mechanics inferred | rust-console: Testing; build plan: Validation; BACKLOG ZK-008,028,035 | Meaningful core tests, captured LSP regressions, local fmt/clippy checks, native/MSRV CI, separate WASM build, catalog checks | [ZK-008.1](BACKLOG.md#task-zk-008-1) [ZK-008.2](BACKLOG.md#task-zk-008-2) [ZK-028.1](BACKLOG.md#task-zk-028-1) [ZK-028.2](BACKLOG.md#task-zk-028-2) [ZK-035.1](BACKLOG.md#task-zk-035-1) [ZK-035.2](BACKLOG.md#task-zk-035-2) | Local commands and hosted results when available |
| REQ-31 | Required | CLAUDE: external lessons; build plan: layout; BACKLOG Phase 5 | Package native artifacts and external lessons; use writable user state/generated paths outside source checkout | [ZK-034.1](BACKLOG.md#task-zk-034-1) [ZK-034.2](BACKLOG.md#task-zk-034-2) | Staged installed-tree test |
| REQ-32 | Required | ADR 0002: cross compilation; BACKLOG Phase 5 | Platform build and actual smoke-test matrix; supported native/extension version combinations | [ZK-035.1](BACKLOG.md#task-zk-035-1) [ZK-035.2](BACKLOG.md#task-zk-035-2) [ZK-035.3](BACKLOG.md#task-zk-035-3) | Platform artifacts and dated manual evidence |
| REQ-33 | Required; recovery/update details inferred | BACKLOG Phase 5; build plan: Distribution | Clean install to first pass, optional agent setup, troubleshooting, update/uninstall and data handling | [ZK-036.1](BACKLOG.md#task-zk-036-1) [ZK-036.2](BACKLOG.md#task-zk-036-2) [ZK-036.3](BACKLOG.md#task-zk-036-3) | Clean-machine walkthrough and lifecycle checklist |
| REQ-34 | Required; release mechanics inferred | BACKLOG ZK-037; ADR 0001 | License/version/release notes, extension submission metadata, reviewable artifacts; external publication separate | [ZK-037.1](BACKLOG.md#task-zk-037-1) [ZK-037.2](BACKLOG.md#task-zk-037-2) [ZK-037.3](BACKLOG.md#task-zk-037-3) | Release review with validation evidence |
| REQ-35 | Excluded | CLAUDE: Out of scope and Hard rules; BACKLOG: Deliberately not doing | No other editors, beginner vim course, accounts, multiplayer, telemetry, leaderboards, web/Electron/local HTML UI | — | Scope review on every task; no implementation tasks |

## Open-question coverage

These owners recheck the relevant claim before dependent implementation. Existing
VERIFIED labels are not strengthened by this matrix; documented and runtime evidence
remain distinct.

| Question | Owner tasks | Required evidence |
|----------|-------------|-------------------|
| OQ-01 granularity | ZK-004.2, ZK-005.1 | Repeated raw change captures and supported-rule verdict |
| OQ-02 registration | ZK-001, ZK-003.2, ZK-003.3 | Documentation plus real installation and lifecycle capture |
| OQ-03 latency | ZK-004.3, ZK-016.1 | Visible timing, sample count and method/overhead limits |
| OQ-04 observation limits | ZK-005.1, ZK-006.1 | Current cited boundary and no richer API dependency |
| OQ-05 observer precedent | ZK-003.3, ZK-005.1 | Local evidence establishes only the behavior actually observed |
| OQ-06 WASM extension route | ZK-003.1, ZK-015.1, ZK-015.3 | Build and real compatibility/registration evidence |
| OQ-07 custom ACP setup | ZK-029.1, ZK-030.2 | Dated config source and manual connection |
| OQ-08 terminal hosting | ZK-023.2 | Fullscreen/resize/cleanup observations on each claimed route |
| OQ-09 SDK maturity | ZK-029.1, ZK-030.1 | Current SDK/toolchain evidence and protocol check |
| OQ-10 cursor placement | ZK-018.1 | CLI source/help plus real path/position test or fallback |
| OQ-11 undo | ZK-004.2, ZK-006.4, ZK-011.1 | Raw undo/redo samples and explicit counting decision |
| OQ-12 curriculum support | ZK-024.1, ZK-025–027 | Per-feature primary source and manual content validation |

## Unresolved decisions

| Conflict or missing contract | Decision owner | Implementation blocked until resolved |
|------------------------------|----------------|----------------------------------------|
| Overlapping current-state writers and last-writer-wins limits | ZK-006.1 | Persistence and LSP/TUI integration |
| Process count versus core library/WASM terminology | ZK-006.1 | Native component boundary documentation |
| LSP active lesson discovery and TUI reset propagation | ZK-006.1, ZK-006.2 | ZK-014.1 and retry/navigation |
| Failure finalization, attempt IDs and dirty buffers | ZK-006.2 | Verification, retry and resume |
| Schema, cursor start, newline and path semantics | ZK-006.3 | Parser, replay and fixture launching |
| Score/weakness formulas and unsupported forbid rules | ZK-006.4 | Scoring and progress computation |
| Generated directory versus writable installed data (resolved: ADR 0005) | ZK-006.5, ZK-029.2 | Resolved by ZK-006.5 and ZK-029.2; ADR 0005 |
| Native Rust 1.75 versus modern extension/ACP dependencies (resolved: MSRV 1.85) | ZK-007.1, ZK-029.1 | Resolved by ZK-007.1 (native) and ZK-029.1 (ACP SDK); ADR 0004 |
| Module numbering and missing draft prerequisites | ZK-024.1 | Full curriculum admission |
| Provider choice and credentials (resolved: LM Studio + Big 3 + ChatGPT auth) | ZK-029.2 | Resolved by ZK-029.2; ADR 0005 |

These are decision tasks, not architecture changes made by this matrix. Inlay hints
are explicitly deferred under REQ-16. REQ-35 exclusions have no implementation tasks.

## User-journey coverage

| Journey | Task checkpoints |
|---------|------------------|
| Setup, observe and decide feasibility | ZK-001–005 |
| Build one exercise and see correct/incorrect feedback | ZK-006–016 |
| Select, hint, retry, advance, quit and resume | ZK-017–023 |
| Recover from missing tool, corrupt data, stale LSP or interruption | ZK-012.1–012.4, ZK-015.2, ZK-018.2, ZK-019.2, ZK-021.2 |
| Explain, generate practice and complete a walkthrough | ZK-029–033 |
| Install outside checkout, update and uninstall | ZK-034–037 |

Scope coverage is complete for the reviewed documents. Feasibility and future evidence
may legitimately revise tasks through recorded decisions; no fixed task count or
deadline is a product requirement.
