# Phase 0 observation matrix

Status: **prepared, not run**. ZK-004.1 only. ZK-003's real editor connection must
pass before collecting ZK-004.2/ZK-004.3 evidence. No result below establishes Zed
behavior, technique distinguishability, or latency.

## Before the run

1. Complete the [dev-extension connection check](../../spikes/lsp-probe/extension/README.md).
2. Record date, Zed version/commit, OS, probe version/dependency lockfiles, vim-mode
   setting, keyboard layout, autosave mode/delay, formatting-on-save and edit prediction.
   Record fixture newline style and any personal keybindings relevant to the actions.
3. Use the repo root as Zed's project; confirm **Zedkata Probe** for every case file.
   The files below have identical UTF-8/LF baselines and distinct filenames/URIs.
4. Use autosave off for cases other than 16. Record initial settings so they can be
   restored. Keep formatting and edit prediction unchanged across comparisons; record
   unexpected automatic edits rather than hiding them. If changing a setting to remove
   interference, repeat the affected comparison and label both runs.
5. Retain the complete capture containing initialization, client capabilities and
   advertised sync/encoding. Each case's URI labels its document events even when
   several cases share one server process. Note capture filename and sequence bounds.

The actions are test inputs, not claims that every binding or input flow is supported.
If a requested operation does not work, record it as not performed and the observed
behavior. Do not substitute an unrecorded method. Manual editor input is needed;
filesystem edits or synthetic clients do not establish Zed's keystroke behavior.

## Baseline and reset

Every file contains:

```rust
fn main() {
    let words = "alpha bravo charlie";
    let message = "hello probe";
    let first = "repeat";
    let second = "repeat";
    let unicode = "café 🌍";
}
```

Open only the current case file. Start from the exact baseline, in normal mode unless
its action says otherwise. Locate the specified text visually; cursor positioning is
not part of the measured edit. Record any setup edits separately.

After the case, wait for feedback, close the tab, and discard unsaved changes. Save
cases intentionally modify their files: reset them only **after closing their tabs**
by copying the baseline above back into those files. Reopen and confirm a fresh
baseline before another run. Do not reset with undo inside the measured window;
undo history and reset changes would contaminate observations. Keep repeat runs in
separate files as provided, or record a new open/close interval and run label.

## Cases

All 20 cases are pending. Each numbered file identifies one test case, not one
expected notification. Compare notifications, individual contentChanges entries,
versions and final text separately.

| Case / fixture | Manual action | Question |
|----------------|---------------|----------|
| [01-dw](../../spikes/lsp-probe/fixtures/matrix/01-dw.kata) | Place normal-mode cursor on the `a` of `alpha` in `words`; perform `dw`. | Word-sized operator deletion. |
| [02-dw-repeat](../../spikes/lsp-probe/fixtures/matrix/02-dw-repeat.kata) | Repeat case 01 in this fresh file. | Repeatability of operator deletion. |
| [03-backspace](../../spikes/lsp-probe/fixtures/matrix/03-backspace.kata) | Enter insert mode immediately after `hello` in `message`; press Backspace five times, then Escape. | Five single-character deletions; record pace. |
| [04-backspace-repeat](../../spikes/lsp-probe/fixtures/matrix/04-backspace-repeat.kata) | Repeat case 03 in this fresh file. | Repeatability of backspace notifications. |
| [05-change-inner](../../spikes/lsp-probe/fixtures/matrix/05-change-inner.kata) | Place normal-mode cursor inside `hello probe`; perform `ci"`, type `fresh`, then Escape. | Delete/insert grouping; intended technique. |
| [06-change-inner-repeat](../../spikes/lsp-probe/fixtures/matrix/06-change-inner-repeat.kata) | Repeat case 05 in this fresh file. | Repeatability of change-inner grouping. |
| [07-visual-change](../../spikes/lsp-probe/fixtures/matrix/07-visual-change.kata) | Visually select exactly `hello probe` without quotes; press `c`, type `fresh`, then Escape. | Same final text as 05, using visual selection. |
| [08-mouse-change](../../spikes/lsp-probe/fixtures/matrix/08-mouse-change.kata) | Use the mouse to select exactly `hello probe`, then replace the selection with `fresh` using the editor's supported input flow. | Same final text as 05; record mode changes and actual steps. |
| [09-multicursor](../../spikes/lsp-probe/fixtures/matrix/09-multicursor.kata) | Using Zed's supported multi-cursor flow, select both occurrences of `repeat`; replace both with `fresh`. | Two changed ranges; record how cursors were created. |
| [10-paste](../../spikes/lsp-probe/fixtures/matrix/10-paste.kata) | In insert mode immediately before `hello probe`, paste `PASTED_TEXT_12345` once, then Escape. | One paste, not simulated typing. |
| [11-undo](../../spikes/lsp-probe/fixtures/matrix/11-undo.kata) | Perform case 01's `dw`, pause two seconds, then undo once. | Separate the setup deletion from its reversal. |
| [12-redo](../../spikes/lsp-probe/fixtures/matrix/12-redo.kata) | Perform `dw` on `alpha`, pause two seconds, undo, pause two seconds, then redo. | Distinguish all three labeled phases. |
| [13-rapid-typing](../../spikes/lsp-probe/fixtures/matrix/13-rapid-typing.kata) | In insert mode immediately before `hello probe`, type `abcdefghijklmno` quickly by hand, then Escape. | Typing burst; do not paste or automate. |
| [14-slow-typing](../../spikes/lsp-probe/fixtures/matrix/14-slow-typing.kata) | Insert the same `abcdefghijklmno` at the same place, about one character per second, then Escape. | Same final text as 13 at a different pace. |
| [15-save-control](../../spikes/lsp-probe/fixtures/matrix/15-save-control.kata) | With autosave off, insert `x` immediately before `hello probe`, pause two seconds, then save once. | Manual-save control; record relevant formatting settings. |
| [16-autosave](../../spikes/lsp-probe/fixtures/matrix/16-autosave.kata) | With autosave enabled, insert `x` at the same position as case 15 and wait for the configured autosave delay. | Same edit under autosave; record mode/delay and any formatting. |
| [17-unicode](../../spikes/lsp-probe/fixtures/matrix/17-unicode.kata) | Select exactly the globe emoji in `unicode`; replace it with `X` using the supported editor flow. | Non-ASCII/UTF-16 range evidence; preserve the accented character. |
| [18-multiline-paste](../../spikes/lsp-probe/fixtures/matrix/18-multiline-paste.kata) | In insert mode at the start of the `message` line, paste the two-line text shown below once, then Escape. | Multiline insertion and newline representation. |
| [19-dw-undo-repeat](../../spikes/lsp-probe/fixtures/matrix/19-dw-undo-repeat.kata) | Repeat case 11, retaining the two-second pause. | Repeatability of undo representation. |
| [20-mouse-change-repeat](../../spikes/lsp-probe/fixtures/matrix/20-mouse-change-repeat.kata) | Repeat case 08 with the same mode/input flow in this fresh file. | Repeatability of the alternative to change-inner. |

For case 18, copy these two lines with a trailing newline to the clipboard:

```text
    // pasted one
    // pasted two
```

Record clipboard newline style if known; preserve what actually arrives on the wire.
Cases 05/06/07/08/20 should all change the message value to `fresh` while preserving
quotes and the surrounding baseline. Cases 13/14 and 15/16 are also same-output
comparisons. If outputs differ, resolve the mismatch before comparing technique
signals; keep the failed run labeled rather than silently discarding it.

## Per-case results template

Copy this block into a dated results document for each case/run. Leave unknowns
explicit; never fill measurements from expectations.

```text
Case / repeat label:
Date / Zed version / settings snapshot:
Capture path / document URI / sequence interval:
Baseline verified:
Actual actions, modes and pace (including setup):
Incoming notification count / contentChanges entry count:
Ranges and rangeLength, or full-document replacement:
Document versions / negotiated encoding:
Raw payload references (retain complete originals):
UTC and monotonic timestamps / inter-notification intervals:
Expected and actual final text match:
Published diagnostic sequence / visible diagnostic observed:
Unexpected auto edits, errors, or unsupported actions:
Interpretation and limitations:
```

For undo/redo cases, label each action interval independently. At least three
representative raw payload examples must accompany the final Phase 0 finding;
retain the complete relevant captures under `tests/fixtures/lsp/` when recording
ZK-004.2. Synthetic captures under `test-output/` remain separate.

## Visible-latency check (ZK-004.3)

After the edit-shape run, repeat a simple single-character edit at least five times,
recording the corresponding diagnostic identifier and observation each time. Use a
measurement method that can correlate the publish event to visible rendering; if a
screen recording is used, record frame rate and clock alignment/error. A subjective
impression or time between two log entries is not a render-latency measurement.

Report sample count, individual results, range/median, method precision and synchronous
capture-write overhead. The probe logs before outbound transport write, so that time
includes subsequent transport work; it is not a render acknowledgement. If the method
cannot distinguish the 300 ms gate reliably, report latency unresolved and improve the
method rather than inventing a precise result. Under 100 ms is the target; the existing
300 ms criterion determines whether inline feedback is the primary channel.

## Handoff

ZK-004.2 owns actual captures and OQ-01/OQ-11 findings. ZK-004.3 owns visible-latency
findings for OQ-03. ZK-005 records the gate decision. This prepared matrix does not
complete any of those tasks. Restore personal editor settings after the run.
