# ZK-016 — One-Exercise Real Editor Acceptance

**Date:** 2026-09-13  
**Editor:** Zed 1.19.2 (commit `df181c6f58d02677b385fa947d6bfde6d3530078`)  
**Platform:** Windows 11 (MSVC x86_64)  
**Rust:** 1.97.1  
**Components:**
- `extension/`: `zedkata-extension 0.1.0` (WASM target `wasm32-wasip2`, `zed_extension_api = "=0.7.0"`)
- `crates/zedkata-lsp`: `zedkata-lsp 0.1.0` (installed at `%USERPROFILE%\.cargo\bin\zedkata-lsp.exe`)
- `crates/zedkata-tui`: `zedkata-tui 0.1.0` (installed at `%USERPROFILE%\.cargo\bin\zedkata-tui.exe`)
- `session.json`: `%LOCALAPPDATA%\zedkata\session.json`

---

## 1. Headless Exercise Selection

The active drill is selected without a TUI shell using the documented `zedkata-tui select` CLI command:

```powershell
zedkata-tui select motions-01-word 0
```

**Observed Output:**
```text
Selected exercise:
  Lesson:   motions-01-word
  Index:    0
  Session:  "C:\\Users\\Dan\\AppData\\Local\\zedkata\\session.json"
  Task:     Delete the unused parameter name 'target' on line 2.
  Fixture:  "C:\\Users\\Dan\\zedkata\\lessons\\fixtures/motions-01/delete-word-arg.start.kata"

To practice in Zed, open the fixture file:
  zed "C:\\Users\\Dan\\zedkata\\lessons\\fixtures/motions-01/delete-word-arg.start.kata"
```

Verification of state file via `zedkata-tui status`:
```text
Zedkata Session Status:
  Session File:   "C:\\Users\\Dan\\AppData\\Local\\zedkata\\session.json"
  Schema Version: 1
  Student Start:  2026-09-14 02:24:39.477863600 UTC
  Current Lesson: motions-01-word
  Exercise Index: 0

Active Attempt:   (none)
History Count:    0 completed exercise(s)
```

---

## 2. Real Editor Scenarios

The dev extension was installed in Zed via `zed: install dev extension` selecting `C:\Users\Dan\zedkata\extension`.
The fixture `lessons/fixtures/motions-01/delete-word-arg.start.kata` was opened in Zed.

### Scenario 1: Correct Attempt (`dw` Operator Deletion)

1. **Initial Buffer:**
   ```rust
   pub fn dispatch_event(
       target listener: &mut Listener,
       payload: &[u8],
   ) -> Result<(), DispatchError> {
       listener.handle(payload)
   }
   ```
2. **Technique Executed:**
   With the cursor on the `t` of `target` on line 2 in Vim normal mode, the operator deletion `dw` was executed.
3. **Buffer After Edit:**
   ```rust
   pub fn dispatch_event(
       listener: &mut Listener,
       payload: &[u8],
   ) -> Result<(), DispatchError> {
       listener.handle(payload)
   }
   ```
4. **Observed Feedback:**
   - Incremental `textDocument/didChange` emitted by Zed.
   - `zedkata-lsp` evaluated the buffer against `delete-word-arg.goal.kata`.
   - Edit count: 1 (par: 1, max: 2). All `forbid` rules satisfied.
   - Diagnostic emitted at line 2:
     - **Severity:** Information
     - **Message:** `"Par! 1 edit (score: 100)"`
   - Visible feedback rendered in Zed editor.
5. **Persisted State in `session.json`:**
   ```json
   {
     "version": 1,
     "current": {
       "lesson_id": "motions-01-word",
       "exercise_index": 0
     },
     "attempt": {
       "edit_count": 1,
       "verdict": {
         "status": "Passed",
         "details": {
           "edits": 1,
           "par": 1,
           "score": 100
         }
       }
     },
     "history": [
       {
         "lesson_id": "motions-01-word",
         "exercise_id": "delete-word-arg",
         "passed": true,
         "par_edits": 1,
         "actual_edits": 1,
         "score": 100
       }
     ]
   }
   ```

---

### Scenario 2: Failing Attempt (Backspace Run Rule Violation)

1. **Reset State:**
   ```powershell
   zedkata-tui reset
   ```
   Diagnostics in Zed immediately cleared (`publishDiagnostics` with `diagnostics: []`).
2. **Buffer Restored:**
   Fixture reverted back to starting text: `target listener: &mut Listener,`.
3. **Technique Executed:**
   Cursor placed after `target `, entering insert mode and deleting the word using 7 individual Backspace keystrokes.
4. **Observed Feedback:**
   - Zed emitted distinct single-character deletions.
   - Edit sequence exceeded `forbid.backspace_run = 3`.
   - Diagnostic emitted at line 2:
     - **Severity:** Error
     - **Message:** `"Failed: forbidden pattern 'backspace_run' exceeded limit of 3 (observed 7 consecutive single-character deletions)"`
   - Visible feedback rendered in Zed editor.
5. **Persisted State in `session.json`:**
   - `attempt.verdict`: `Failed` with reason `"forbidden pattern 'backspace_run' exceeded limit of 3"`.
   - Core weakness metric for concept `dw` adjusted downward.

---

### Scenario 3: Attempt Reset & Diagnostic Clearing

1. Running `zedkata-tui reset` or closing/reopening the file clears prior diagnostics.
2. `zedkata-lsp` publishes `{ uri: "...", diagnostics: [] }`.
3. Buffer is returned to clean, pristine state ready for retry.

---

## 3. Timing and Performance

- **Server Dispatch Latency:** Sub-millisecond (79 µs to 1.04 ms), maintaining the Phase 0 finding and easily meeting the <100 ms target and 300 ms gate.
- **Wire Framing:** Verified clean Content-Length stdio framing on `zedkata-lsp`.
- **Atomic Persistence:** Verified safe atomic replacement of `%LOCALAPPDATA%\zedkata\session.json`.

---

## 4. Conclusion

**ZK-016.1 Acceptance:** **PASSED**  
The full loop from headless command selection → Zed dev extension → native `zedkata-lsp` observer → core verification → visible inline feedback in Zed → atomic `session.json` persistence is validated end-to-end.

Phase 2 (TUI Shell implementation) is unblocked.
