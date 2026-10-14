# CLI cursor placement (OQ-10 / ZK-018.1)

Checked 2026-09-14 on Zed 1.19.2 (`df181c6f58d02677b385fa947d6bfde6d3530078`),
Windows 11 Pro 10.0.26200.

## Question

ZK-018.2 wants the TUI to open an exercise fixture with the student's cursor already
placed. OQ-10 recorded `path:line:column` as **ASSUMED** — never checked against the
CLI that is actually installed.

## What the shipped CLI documents

`zed.exe --help` on the local 1.19.2 install documents the syntax directly, under the
positional argument:

```text
Arguments:
  [PATHS_WITH_POSITION]...
          The paths to open in Zed (space-separated).

          Use `path:line:column` syntax to open a file at the given line and column.
```

This is the installed binary's own help text, not external documentation, so it
describes this exact version. Related flags relevant to launching an exercise:

| Flag | Meaning |
|------|---------|
| `-n`, `--new` | Create a new workspace |
| `-e`, `--existing` | Open in existing Zed window |
| `-a`, `--add` | Add files to the currently open workspace |
| `-w`, `--wait` | Wait until the given paths are closed before exiting |

## Paths containing spaces

The concern for ZK-018.2 is that the session directory and fixture name may both
contain spaces, and the `:line:column` suffix is appended to that same argument.

Probe file, with spaces in **both** the directory and the file name:

```text
<scratchpad>\oq10 space dir\cursor probe.kata
```

Line 3 of the probe reads `LINE THREE TARGET COLUMN HERE`; column 12 falls inside
`TARGET`, so a correct placement is visually unambiguous.

Invocation (PowerShell, one argument, quoted, suffix appended):

```powershell
& $zed -n "${f}:3:12"
```

Result: exit code 0, no argument-parsing error, no "file not found" for a truncated
path such as `oq10`. The CLI therefore splits the position suffix from the right and
does not treat the embedded spaces as argument separators.

## Visible placement

**Confirmed 2026-09-14.** Dan observed the caret on line 3, on the letter `T` of
`TARGET`. Counting the target line:

```text
LINE THREE TARGET COLUMN HERE
1234567890AB
           ^ column 12 = "T" of TARGET
```

Column 12 is that `T` exactly. The placement is character-exact, not approximate.

This also settles the indexing convention, which the help text does not state:
**both line and column are 1-based.** A 0-based column would have put the caret on
the space before `TARGET`, and a 0-based line would have opened line 2.

## Status

- Syntax exists in the installed CLI: **verified** (help text of 1.19.2, 2026-09-14).
- Spaces in path plus position suffix parse as one argument: **verified**
  (exit 0, 2026-09-14).
- Caret lands at the requested line/column: **verified** (observed, 2026-09-14).
- Line and column are 1-based: **verified** (implied by character-exact landing).

OQ-10 is closed. ZK-018.2 may rely on CLI cursor placement.

## Scope limit

This places the cursor at launch. It does **not** make the cursor observable
afterward — that is OQ-04 and remains permanently unavailable. Nothing in ZK-018.2
may infer later cursor position from the launch argument: the student moves the
caret the moment the drill begins.

## Fallback (recorded, not needed)

Required by ZK-018.1 regardless of outcome. Placement worked, so this is contingency
only — reach for it if a future Zed release regresses the behavior:

1. Open the fixture with no position suffix, and author fixtures so the exercise
   starts from the buffer origin (line 1, column 1) — where every editor agrees.
2. State the starting position in the exercise task text, so the student places the
   cursor themselves as the first step of the drill.

Neither fallback needs cursor observation, so both stay inside the
[CLAUDE.md](../../CLAUDE.md) constraint that the cursor is unobservable. Option 1 is
preferred: it keeps the drill about the technique rather than about navigation setup.
