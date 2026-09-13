# Architecture

## Components

### `zedkata-core`

Pure library. Sync, no Tokio, no I/O beyond reading and writing the files it owns.

Responsibilities:
- Parse lesson TOML into a typed `Lesson` / `Exercise` model.
- Own the `SessionState` type and its serialization.
- Implement verification: given a starting buffer, a goal buffer, and an ordered list of
  observed edits, decide pass/fail and compute a technique score.
- Scoring rules. No other crate decides whether an attempt passed.

Everything else is a shell around this.

### `zedkata-lsp`

A language server registered for the `zedkata` language (file extension `.kata`). Zed
spawns it and speaks LSP over stdio. It is not a real language server and makes no
attempt to be one.

What it does:
- On `textDocument/didOpen`, records the baseline buffer.
- On `textDocument/didChange`, appends the delta to an in-memory edit log for that
  document and writes a rolling summary into `session.json`.
- On a passing or failing attempt, pushes `textDocument/publishDiagnostics` back into
  the buffer as inline feedback — this is the primary feedback channel, because it puts
  the response where the student's eyes already are.
- Optionally serves `textDocument/inlayHint` to render the current task inline above the
  target line.

What it does not do: completions, hover, formatting, symbols. Return empty.

**Precedent:** a Zed Discord-presence extension was built by shipping a fake language
server that reported which files were open. The seam is known to work; the open question
is only how granular the data is.

### `zedkata-tui`

ratatui + crossterm. Runs as a Zed Terminal Thread (`zed: new terminal thread`) or in a
terminal pane.

Panes: lesson list, current task text, live attempt feedback, progress/score.

Its only inputs are the lesson files and `session.json`. It polls the state file; it
does not talk to the LSP.

Navigation is vim-keyed, obviously.

### `zedkata-agent`

An ACP agent registered in Zed's `settings.json`:

```json
{
  "agent_servers": {
    "zedkata": {
      "type": "custom",
      "command": "zedkata-agent",
      "args": ["--acp"],
      "env": {}
    }
  }
}
```

Handles what drills cannot:
- "Why did `cib` include the braces?" — explanation grounded in the actual edit log.
- Generating a fresh exercise targeting a motion the student keeps failing.
- Guided walkthroughs of Zed features that have no pass/fail state: the agent panel,
  multibuffer editing, tasks, the debugger, ACP itself.

It reads `session.json` for context. It writes lesson-generation output to
`lessons/generated/`.

### `zedkata-ext`

Minimal Zed extension, Rust → WASM. Registers the `zedkata` language and points Zed at
the `zedkata-lsp` binary. Expected to be ~100 lines and then never touched.

---

## State contract

Single file. Everything else is derived.

Path: `$XDG_STATE_HOME/zedkata/session.json`, falling back to platform equivalents.

```jsonc
{
  "version": 1,
  "student": { "started_at": "2026-09-13T14:02:11Z" },
  "current": {
    "lesson_id": "motions-03-textobjects",
    "exercise_index": 2,
    "buffer_path": "/tmp/zedkata/motions-03-2.kata",
    "opened_at": "2026-09-13T14:31:00Z"
  },
  "attempt": {
    "edit_count": 7,
    "edits": [
      { "at": "2026-09-13T14:31:04.221Z",
        "range": { "start": [4, 12], "end": [4, 19] },
        "inserted": "", "deleted_len": 7 }
    ],
    "verdict": null
  },
  "history": [
    { "lesson_id": "motions-01-word", "passed": true, "par_edits": 3,
      "actual_edits": 3, "score": 100, "completed_at": "..." }
  ],
  "weaknesses": { "text-objects": 0.4, "registers": 0.9 }
}
```

Writers: `zedkata-lsp` owns `current` and `attempt`. `zedkata-tui` owns `current.lesson_id`
when the student navigates. `zedkata-core` computes `history` and `weaknesses` on verdict.

Concurrent-write handling: write to a temp file and rename. Last writer wins. This is a
single-user local tool; do not build a lock protocol.

---

## Lesson format

```toml
[lesson]
id = "motions-03-textobjects"
title = "Text objects"
requires = ["motions-01-word"]
concepts = ["ci", "ca", "di", "AnyQuotes", "AnyBrackets"]

prose = """
Zed ships text objects vim doesn't have. `AnyQuotes` and `AnyBrackets` select
quoted or bracketed text without you naming which delimiter. `MiniQuotes` and
`MiniBrackets` follow mini.ai's behavior instead.
"""

[[exercise]]
id = "swap-string-contents"
task = "Replace the contents of the string on line 4. Do not touch the quotes."
start = "fixtures/motions-03/swap-string.start.kata"
goal  = "fixtures/motions-03/swap-string.goal.kata"
par_edits = 1                       # `ci"` then type — one contiguous edit
max_edits = 3
hint = "You do not need to know which kind of quote it is."

[exercise.forbid]
backspace_run = 3                   # 3+ consecutive single-char deletions fails
```

`par_edits` is the technique signal. Matching the buffer is necessary but not
sufficient; the attempt must reach the goal within the edit budget.

**This entire mechanism is contingent on Phase 0.** If Zed coalesces `didChange`,
`par_edits` and `forbid` are meaningless and the format degrades to goal-matching only.

---

## Feedback loop

```
student types in Zed
  → Zed sends didChange to zedkata-lsp
  → lsp appends to edit log, writes session.json
  → lsp asks core: does buffer match goal?
      no  → continue
      yes → core scores the edit log against par_edits and forbid rules
  → lsp publishes diagnostic into the buffer ("Passed — 1 edit, par 1")
  → tui polls session.json, updates progress, unlocks next exercise
```

Target round-trip: under 100ms from keystroke to diagnostic. If it lands above ~300ms
the inline feedback feels broken and the TUI becomes the primary channel instead.
