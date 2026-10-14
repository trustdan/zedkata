//! Optional raw-traffic capture for evidence gathering.
//!
//! Enabled only when `ZEDKATA_CAPTURE` names an output path. Captures exist to answer
//! open questions in `docs/OPEN-QUESTIONS.md` with reproducible wire evidence; nothing
//! in the server's behavior may depend on whether capture is active.
//!
//! The record shape matches the Phase 0 probe captures in `tests/fixtures/lsp/`, so new
//! captures replay through the existing golden-fixture tests.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Appends raw JSON-RPC payloads to a `.jsonl` file.
///
/// Write failures disable the capture and warn on stderr rather than surfacing as
/// protocol errors: losing evidence is recoverable, dropping a student's session is not.
pub struct Capture {
    file: Option<File>,
    sequence: u64,
}

impl Capture {
    /// Builds a capture from the `ZEDKATA_CAPTURE` environment variable.
    ///
    /// Returns a disabled capture when the variable is unset or the file cannot be
    /// opened, so the caller never needs to branch on whether capture is configured.
    pub fn from_env() -> Self {
        match std::env::var("ZEDKATA_CAPTURE") {
            Ok(path) if !path.trim().is_empty() => Self::to_path(Path::new(&path)),
            _ => Self::disabled(),
        }
    }

    fn disabled() -> Self {
        Self {
            file: None,
            sequence: 0,
        }
    }

    fn to_path(path: &Path) -> Self {
        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => {
                eprintln!("[zedkata-lsp] capturing traffic to {}", path.display());
                Self {
                    file: Some(file),
                    sequence: 0,
                }
            }
            Err(e) => {
                eprintln!(
                    "[zedkata-lsp] capture disabled: cannot open {}: {}",
                    path.display(),
                    e
                );
                Self::disabled()
            }
        }
    }

    /// Records one raw payload with its direction (`"in"` or `"out"`).
    ///
    /// Payloads are written as parsed JSON when possible and as an escaped string
    /// otherwise, so a malformed message is still evidence rather than a gap.
    pub fn record(&mut self, direction: &str, payload: &[u8]) {
        let Some(file) = self.file.as_mut() else {
            return;
        };

        self.sequence += 1;

        let message: serde_json::Value = serde_json::from_slice(payload)
            .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(payload).into()));

        let record = serde_json::json!({
            "sequence": self.sequence,
            "at": now_iso8601(),
            "direction": direction,
            "message": message,
        });

        let mut line = match serde_json::to_vec(&record) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("[zedkata-lsp] capture disabled: cannot serialize record: {e}");
                self.file = None;
                return;
            }
        };
        line.push(b'\n');

        if let Err(e) = file.write_all(&line).and_then(|()| file.flush()) {
            eprintln!("[zedkata-lsp] capture disabled: write failed: {e}");
            self.file = None;
        }
    }

    /// Whether records are being written.
    pub fn is_active(&self) -> bool {
        self.file.is_some()
    }
}

/// UTC timestamp in the `2026-09-13T14:33:58.619Z` form used by the Phase 0 captures.
fn now_iso8601() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    let days = secs / 86_400;
    let time_of_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year,
        month,
        day,
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60,
        millis
    )
}

/// Howard Hinnant's days-from-civil inverse, avoiding a date dependency for one timestamp.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_capture_records_nothing() {
        let mut capture = Capture::disabled();
        capture.record("in", b"{\"method\":\"initialize\"}");
        assert!(!capture.is_active());
        assert_eq!(capture.sequence, 0);
    }

    #[test]
    fn records_match_golden_fixture_shape() {
        let dir = std::env::temp_dir().join(format!("zedkata-capture-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("shape.jsonl");

        let mut capture = Capture::to_path(&path);
        assert!(capture.is_active());
        capture.record(
            "in",
            br#"{"jsonrpc":"2.0","method":"textDocument/didChange"}"#,
        );
        capture.record("out", br#"{"jsonrpc":"2.0","method":"publishDiagnostics"}"#);
        drop(capture);

        let contents = std::fs::read_to_string(&path).expect("capture file");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);

        let first: serde_json::Value = serde_json::from_str(lines[0]).expect("valid json line");
        assert_eq!(first["sequence"], 1);
        assert_eq!(first["direction"], "in");
        assert_eq!(first["message"]["method"], "textDocument/didChange");
        assert!(first["at"].as_str().is_some_and(|s| s.ends_with('Z')));

        let second: serde_json::Value = serde_json::from_str(lines[1]).expect("valid json line");
        assert_eq!(second["sequence"], 2);
        assert_eq!(second["direction"], "out");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn malformed_payload_is_recorded_as_text() {
        let dir = std::env::temp_dir().join(format!("zedkata-capture-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("malformed.jsonl");

        let mut capture = Capture::to_path(&path);
        capture.record("in", b"not json at all");
        drop(capture);

        let contents = std::fs::read_to_string(&path).expect("capture file");
        let record: serde_json::Value =
            serde_json::from_str(contents.lines().next().expect("one line")).expect("valid json");
        assert_eq!(record["message"], "not json at all");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unopenable_path_disables_capture() {
        let path = std::env::temp_dir()
            .join("zedkata-missing-dir")
            .join("nested")
            .join("capture.jsonl");
        let capture = Capture::to_path(&path);
        assert!(!capture.is_active());
    }

    #[test]
    fn timestamp_matches_known_epoch_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_709), (2026, 9, 13));
    }
}
