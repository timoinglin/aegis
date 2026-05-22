use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;
use tauri::{AppHandle, Manager};

fn daily_file() -> String {
    format!("aegis-{}.log", Local::now().format("%Y-%m-%d"))
}

/// Where the daily op-log lives: %APPDATA%\Aegis\logs\aegis-YYYY-MM-DD.log
fn log_path(app: &AppHandle) -> Option<PathBuf> {
    // %APPDATA%\Aegis\logs — see the note in settings_store.rs on why not app_data_dir().
    let dir = app.path().data_dir().ok()?.join("Aegis").join("logs");
    create_dir_all(&dir).ok()?;
    Some(dir.join(daily_file()))
}

/// Replace every secret occurrence with "***" so passwords never hit the log file.
fn redact(text: &str, secrets: &[String]) -> String {
    let mut out = text.to_string();
    for s in secrets {
        if !s.is_empty() {
            out = out.replace(s, "***");
        }
    }
    out
}

/// Write one operation line to the op-log. `detail` may contain the raw
/// underlying error — it is redacted here before it touches disk.
///
/// This is the second half of the golden rule: friendly text goes to the UI,
/// the raw (redacted) detail goes here so users can attach the log to a GitHub issue.
pub fn log_op(app: &AppHandle, level: &str, context: &str, detail: &str, secrets: &[String]) {
    if let Some(path) = log_path(app) {
        write_line(&path, level, context, detail, secrets);
    }
}

/// Headless logger (no AppHandle) for the --backup CLI path. Same file as the GUI.
pub fn log_headless(level: &str, context: &str, detail: &str, secrets: &[String]) {
    let Some(base) = std::env::var_os("APPDATA").map(PathBuf::from) else { return };
    let dir = base.join("Aegis").join("logs");
    if create_dir_all(&dir).is_ok() {
        write_line(&dir.join(daily_file()), level, context, detail, secrets);
    }
}

fn write_line(path: &PathBuf, level: &str, context: &str, detail: &str, secrets: &[String]) {
    let line = format!(
        "[{}] {:5} {} :: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level,
        context,
        redact(detail, secrets)
    );
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_every_secret_occurrence() {
        let raw = "auth as root:ascent then ra claude:rapass";
        let out = redact(raw, &["ascent".into(), "rapass".into()]);
        assert!(!out.contains("ascent") && !out.contains("rapass"));
        assert_eq!(out, "auth as root:*** then ra claude:***");
    }

    #[test]
    fn empty_secret_is_ignored() {
        // An empty secret must not turn into a "***" carpet over the whole string.
        let out = redact("hello", &[String::new()]);
        assert_eq!(out, "hello");
    }
}
