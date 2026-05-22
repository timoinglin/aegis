use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::process::Stdio;

use tauri::AppHandle;

use crate::models::{DbBackupInfo, RestoreResult, Settings};
use crate::services::{backup, logging, mysql, server_state};

/// The exact word the user must type to confirm a restore.
pub const CONFIRM_WORD: &str = "RESTORE";

/// Full restore with the complete safety net:
///   1. typed confirmation, 2. the file looks like a real backup,
///   3. worldserver/authserver are stopped, 4. an automatic safety backup,
///   then the import — and only then do we touch the live databases.
pub fn restore(
    app: &AppHandle,
    settings: &Settings,
    sql_path: &str,
    confirmation: &str,
) -> Result<RestoreResult, String> {
    let secrets = settings.secrets();

    if !is_confirmed(confirmation) {
        return Err(format!("Please type {CONFIRM_WORD} to confirm."));
    }

    let path = Path::new(sql_path);
    if !path.is_file() {
        return Err("That backup file no longer exists.".into());
    }
    if !looks_like_backup(path) {
        return Err("That file doesn't look like a database backup Aegis can restore.".into());
    }

    // Guard: restoring while the server is up corrupts the live database.
    preflight_servers_stopped()?;

    // Safety net: a fresh backup BEFORE we overwrite anything. Abort if it fails.
    logging::log_op(app, "INFO", "restore", "taking safety backup before restore", &secrets);
    let safety = backup::create_backup(app, settings, "safety").map_err(|e| {
        format!("Couldn't take a safety backup first, so the restore was cancelled. {e}")
    })?;

    // Do the import.
    logging::log_op(app, "INFO", "restore", &format!("restoring from {sql_path}"), &secrets);
    run_sql_file(settings, path).map_err(|raw| {
        logging::log_op(app, "ERROR", "restore", &format!("import failed: {raw}"), &secrets);
        format!(
            "The restore failed. Your data was NOT changed beyond what the import managed — your safety backup is at {}. See the log for details.",
            safety.path
        )
    })?;

    // Sanity readout for the databases the file targeted.
    let databases: Vec<DbBackupInfo> = db_names_in_file(path)
        .iter()
        .map(|db| backup::db_stats(settings, db))
        .collect();

    logging::log_op(
        app,
        "INFO",
        "restore",
        &format!("restore complete from {sql_path} (safety: {})", safety.path),
        &secrets,
    );

    Ok(RestoreResult {
        restored_from: sql_path.to_string(),
        safety_backup_path: safety.path,
        databases,
    })
}

/// Run a .sql file through the bundled mysql client (mysql reads it from stdin).
/// Pure mechanic — no guards — so it can be exercised against a scratch DB.
pub fn run_sql_file(settings: &Settings, path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Couldn't open the backup file: {e}"))?;
    let output = mysql::base_cmd(settings, "mysql")?
        .stdin(Stdio::from(file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Couldn't start mysql: {e}"))?
        .wait_with_output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// True only for the exact confirmation word.
pub fn is_confirmed(confirmation: &str) -> bool {
    confirmation.trim() == CONFIRM_WORD
}

/// Worldserver/authserver must be stopped (mysqld stays up — we need it).
pub fn preflight_servers_stopped() -> Result<(), String> {
    let mut running = Vec::new();
    if server_state::is_running("worldserver.exe") {
        running.push("worldserver");
    }
    if server_state::is_running("authserver.exe") {
        running.push("authserver");
    }
    if running.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Stop your {} before restoring — restoring while the server is running can corrupt your database.",
            running.join(" and ")
        ))
    }
}

/// Cheap heuristic that the file is a SQL dump: a CREATE/USE near the top, or our
/// mysqldump completion footer near the end.
fn looks_like_backup(path: &Path) -> bool {
    if let Ok(f) = File::open(path) {
        let mut head = String::new();
        let _ = BufReader::new(f).take(8192).read_to_string(&mut head);
        let h = head.to_uppercase();
        if h.contains("CREATE DATABASE") || h.contains("CREATE TABLE") || h.contains("USE `") {
            return true;
        }
    }
    // Fall back to the footer (tail).
    if let Ok(mut f) = File::open(path) {
        if let Ok(len) = f.metadata().map(|m| m.len()) {
            let tail = len.min(512);
            if f.seek(SeekFrom::End(-(tail as i64))).is_ok() {
                let mut buf = String::new();
                let _ = f.read_to_string(&mut buf);
                return buf.contains("Dump completed");
            }
        }
    }
    false
}

/// Database names named by `CREATE DATABASE ... \`name\`` lines in the dump.
fn db_names_in_file(path: &Path) -> Vec<String> {
    let Ok(f) = File::open(path) else { return Vec::new() };
    let mut names = Vec::new();
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        let t = line.trim_start();
        if t.starts_with("CREATE DATABASE") {
            if let Some(name) = line.split('`').nth(1) {
                let n = name.to_string();
                if !names.contains(&n) {
                    names.push(n);
                }
            }
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn confirmation_must_match_exactly() {
        assert!(is_confirmed("RESTORE"));
        assert!(is_confirmed("  RESTORE  ")); // trimmed
        assert!(!is_confirmed("restore"));
        assert!(!is_confirmed("RESTOR"));
        assert!(!is_confirmed(""));
    }

    #[test]
    fn detects_backup_and_extracts_db_names() {
        let mut p = std::env::temp_dir();
        p.push("aegis_restore_test.sql");
        fs::write(
            &p,
            "-- header\nCREATE DATABASE /*!32312 IF NOT EXISTS*/ `scratch` /*x*/;\nUSE `scratch`;\nCREATE TABLE t(id int);\n",
        )
        .unwrap();
        assert!(looks_like_backup(&p));
        assert_eq!(db_names_in_file(&p), vec!["scratch".to_string()]);
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn rejects_non_backup_file() {
        let mut p = std::env::temp_dir();
        p.push("aegis_not_backup.txt");
        fs::write(&p, "just some notes, nothing sql here").unwrap();
        assert!(!looks_like_backup(&p));
        let _ = fs::remove_file(&p);
    }
}
