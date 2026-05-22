use std::os::windows::process::CommandExt;
use std::process::Command;

use crate::models::{ScheduleStatus, Settings};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TASK_NAME: &str = "Aegis Automatic Backup";

fn schtasks(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("schtasks")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

fn weekday_name(code: &str) -> &str {
    match code.to_uppercase().as_str() {
        "MON" => "Monday",
        "TUE" => "Tuesday",
        "WED" => "Wednesday",
        "THU" => "Thursday",
        "FRI" => "Friday",
        "SAT" => "Saturday",
        _ => "Sunday",
    }
}

/// Plain-language summary built from settings (locale-independent).
fn summary(s: &Settings) -> String {
    if s.backup_frequency == "weekly" {
        format!("Every {} at {}", weekday_name(&s.backup_weekday), s.backup_time)
    } else {
        format!("Every day at {}", s.backup_time)
    }
}

fn task_exists() -> bool {
    schtasks(&["/Query", "/TN", TASK_NAME])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse "Next Run Time" by CSV column POSITION (column 3), so it works regardless
/// of the system language or console encoding.
fn next_run() -> Option<String> {
    let out = schtasks(&["/Query", "/TN", TASK_NAME, "/FO", "CSV", "/V", "/NH"]).ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| !l.trim().is_empty())?;
    // Columns: "HostName","TaskName","Next Run Time","Status",...
    let fields: Vec<&str> = line.split("\",\"").collect();
    let v = fields.get(2)?.trim_matches('"').trim();
    if v.is_empty() || v.starts_with("N/A") {
        None
    } else {
        Some(v.to_string())
    }
}

pub fn status(settings: &Settings) -> ScheduleStatus {
    let enabled = task_exists();
    ScheduleStatus {
        enabled,
        summary: if enabled { summary(settings) } else { "Off".into() },
        next_run: if enabled { next_run() } else { None },
    }
}

/// Create/update or remove the scheduled task to match settings.
pub fn apply(settings: &Settings) -> Result<ScheduleStatus, String> {
    if settings.backup_schedule_enabled {
        create(settings)?;
    } else {
        remove();
    }
    Ok(status(settings))
}

fn create(settings: &Settings) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Couldn't find Aegis's own path: {e}"))?;
    let tr = format!("\"{}\" --backup", exe.display());
    let weekly = settings.backup_frequency == "weekly";

    let mut args: Vec<&str> = vec![
        "/Create", "/TN", TASK_NAME, "/TR", &tr,
        "/SC", if weekly { "WEEKLY" } else { "DAILY" },
        "/ST", &settings.backup_time,
        "/F", // overwrite if it already exists
    ];
    if weekly {
        args.push("/D");
        args.push(&settings.backup_weekday);
    }

    let out = schtasks(&args).map_err(|e| format!("Couldn't run Task Scheduler: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("Task Scheduler refused the schedule: {}", err.trim()))
    }
}

fn remove() {
    let _ = schtasks(&["/Delete", "/TN", TASK_NAME, "/F"]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_reads_settings() {
        let mut s = Settings::default();
        assert_eq!(summary(&s), "Every day at 03:00");
        s.backup_frequency = "weekly".into();
        s.backup_weekday = "SUN".into();
        s.backup_time = "04:30".into();
        assert_eq!(summary(&s), "Every Sunday at 04:30");
    }
}
