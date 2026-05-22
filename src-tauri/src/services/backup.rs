use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Instant, SystemTime};

use chrono::Local;
use tauri::{AppHandle, Manager};

use crate::models::{BackupResult, DbBackupInfo, Settings};
use crate::services::{logging, mysql, settings_store};

/// Default backup location: %APPDATA%\Aegis\backups
pub fn default_dir(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().data_dir().ok()?.join("Aegis").join("backups"))
}

/// The folder backups are written to (user setting, or the default).
pub fn resolve_dir(app: &AppHandle, settings: &Settings) -> Result<PathBuf, String> {
    let dir = match settings.backup_dir.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(custom) => PathBuf::from(custom),
        None => default_dir(app).ok_or("Could not resolve the default backup folder.")?,
    };
    fs::create_dir_all(&dir).map_err(|e| format!("Couldn't create the backup folder: {e}"))?;
    Ok(dir)
}

/// Which databases to back up. Authoritative source = the DB names the worldserver
/// actually uses (worldserver.conf), so this adapts to Free (mop_*) vs Premium and
/// any custom names. Falls back to probing for the standard game DBs.
pub fn databases_to_backup(settings: &Settings) -> Vec<String> {
    // Authoritative: the DB names worldserver.conf actually uses (handles renamed DBs).
    if let Some(info) =
        crate::services::repack_conf::read(settings.server_path.as_deref(), settings.repack_path.as_deref())
    {
        if !info.databases.is_empty() {
            return info.databases;
        }
    }
    // Fallback: keep only the standard game DBs that actually exist.
    let existing = mysql::query(settings, "SHOW DATABASES").unwrap_or_default();
    let present: Vec<&str> = existing.lines().map(str::trim).collect();
    ["auth", "characters", "world", "mop_auth", "mop_characters", "mop_world"]
        .into_iter()
        .filter(|db| present.contains(db))
        .map(String::from)
        .collect()
}

pub fn db_stats(settings: &Settings, db: &str) -> DbBackupInfo {
    let sql = format!(
        "SELECT COUNT(*), IFNULL(SUM(table_rows),0) FROM information_schema.tables WHERE table_schema='{db}'"
    );
    let (tables, approx_rows) = mysql::query(settings, &sql)
        .ok()
        .and_then(|out| {
            let mut parts = out.split('\t');
            Some((
                parts.next()?.trim().parse().ok()?,
                parts.next()?.trim().parse().ok()?,
            ))
        })
        .unwrap_or((0, 0));
    DbBackupInfo {
        name: db.to_string(),
        tables,
        approx_rows,
    }
}

/// Create a full backup of the live game databases. Reused as the safety backup
/// before destructive operations. Resolves the folder, runs the dump, and logs.
pub fn create_backup(app: &AppHandle, settings: &Settings, label: &str) -> Result<BackupResult, String> {
    let dir = resolve_dir(app, settings)?;
    let secrets = settings.secrets();
    match create_backup_to(settings, &dir, label) {
        Ok(result) => {
            logging::log_op(
                app,
                "INFO",
                "backup/create",
                &format!(
                    "{} ({} DBs, {} bytes, footer={}, {}s)",
                    result.path, result.databases.len(), result.size_bytes, result.completed, result.duration_secs
                ),
                &secrets,
            );
            Ok(result)
        }
        Err(raw) => {
            logging::log_op(app, "ERROR", "backup/create", &format!("failed: {raw}"), &secrets);
            // Friendly for the UI; the raw cause is in the log above.
            Err(if raw.contains("mysqldump") || raw.contains("disk") {
                "The backup didn't complete. See the log for details.".into()
            } else {
                raw
            })
        }
    }
}

/// The pure dump: write a full backup of the live game DBs to `dir`. No AppHandle,
/// no logging — so it's unit/integration-testable. Returns size + per-DB sanity info.
pub fn create_backup_to(settings: &Settings, dir: &Path, label: &str) -> Result<BackupResult, String> {
    fs::create_dir_all(dir).map_err(|e| format!("Couldn't create the backup folder: {e}"))?;

    let dbs = databases_to_backup(settings);
    if dbs.is_empty() {
        return Err("Couldn't find any databases to back up. Check the connection in Settings.".into());
    }

    let stats: Vec<DbBackupInfo> = dbs.iter().map(|db| db_stats(settings, db)).collect();

    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("aegis-{label}-{stamp}.sql"));
    let file = File::create(&path).map_err(|e| format!("Couldn't create the backup file: {e}"))?;

    let started = Instant::now();
    let mut cmd = mysql::base_cmd(settings, "mysqldump")?;
    cmd.arg("--single-transaction") // consistent snapshot of InnoDB without locking
        .arg("--routines")
        .arg("--events")
        .arg("--default-character-set=utf8mb4")
        .arg("--databases")
        .args(&dbs)
        .stdout(Stdio::from(file))
        .stderr(Stdio::piped());

    let output = cmd
        .spawn()
        .map_err(|e| format!("Couldn't start mysqldump: {e}"))?
        .wait_with_output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let raw = String::from_utf8_lossy(&output.stderr).into_owned();
        let _ = fs::remove_file(&path); // don't leave a half-written file around
        return Err(format!("mysqldump failed: {raw}"));
    }

    let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    Ok(BackupResult {
        path: path.to_string_lossy().into_owned(),
        size_bytes,
        completed: has_completion_footer(&path),
        databases: stats,
        duration_secs: started.elapsed().as_secs(),
    })
}

/// mysqldump writes "-- Dump completed on ..." as its last line ONLY on success,
/// so its presence is a reliable "the file isn't truncated" signal.
fn has_completion_footer(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else { return false };
    let Ok(len) = f.metadata().map(|m| m.len()) else { return false };
    let tail = len.min(512);
    if f.seek(SeekFrom::End(-(tail as i64))).is_err() {
        return false;
    }
    let mut buf = String::new();
    if f.read_to_string(&mut buf).is_err() {
        // Binary-safe fallback isn't needed; mysqldump tails are ASCII.
        return false;
    }
    buf.contains("Dump completed")
}

/// All *.sql backups in the folder, newest first — for the Restore picker.
pub fn list_backups(dir: &Path) -> Vec<crate::models::BackupFile> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<crate::models::BackupFile> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "sql"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            Some(crate::models::BackupFile {
                name: e.file_name().to_string_lossy().into_owned(),
                path: e.path().to_string_lossy().into_owned(),
                size_bytes: meta.len(),
                modified_ms,
            })
        })
        .collect();
    files.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    files
}

/// The login/auth DB name (where web_* tables live) per worldserver.conf, else "auth".
fn login_db(settings: &Settings) -> String {
    crate::services::repack_conf::read(settings.server_path.as_deref(), settings.repack_path.as_deref())
        .and_then(|i| i.databases.into_iter().next())
        .unwrap_or_else(|| "auth".into())
}

/// Names of the registration-portal tables (web_*) in the auth DB.
fn web_tables(settings: &Settings, db: &str) -> Vec<String> {
    let sql = format!(
        "SELECT table_name FROM information_schema.tables WHERE table_schema='{db}' AND table_name LIKE 'web\\_%' ORDER BY table_name"
    );
    mysql::query(settings, &sql)
        .map(|out| out.lines().map(|l| l.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

/// Back up only the registration-portal (web_*) tables from the auth DB.
pub fn create_web_backup(app: &AppHandle, settings: &Settings) -> Result<BackupResult, String> {
    let dir = resolve_dir(app, settings)?;
    let db = login_db(settings);
    let tables = web_tables(settings, &db);
    if tables.is_empty() {
        return Err("No website tables found — this is only useful if you installed the registration portal (wow-mop-registration).".into());
    }

    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("aegis-web-{stamp}.sql"));
    let file = File::create(&path).map_err(|e| format!("Couldn't create the backup file: {e}"))?;

    let started = Instant::now();
    let mut cmd = mysql::base_cmd(settings, "mysqldump")?;
    cmd.arg("--single-transaction")
        .arg("--default-character-set=utf8mb4")
        .arg(&db)
        .args(&tables)
        .stdout(Stdio::from(file))
        .stderr(Stdio::piped());

    let output = cmd
        .spawn()
        .map_err(|e| format!("Couldn't start mysqldump: {e}"))?
        .wait_with_output()
        .map_err(|e| e.to_string())?;

    let secrets = settings.secrets();
    if !output.status.success() {
        let raw = String::from_utf8_lossy(&output.stderr).into_owned();
        let _ = fs::remove_file(&path);
        logging::log_op(app, "ERROR", "backup/web", &format!("mysqldump failed: {raw}"), &secrets);
        return Err("The website backup didn't complete. See the log for details.".into());
    }

    let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let result = BackupResult {
        path: path.to_string_lossy().into_owned(),
        size_bytes,
        completed: has_completion_footer(&path),
        databases: vec![DbBackupInfo { name: format!("{db} (web_* only)"), tables: tables.len() as u64, approx_rows: 0 }],
        duration_secs: started.elapsed().as_secs(),
    };
    logging::log_op(app, "INFO", "backup/web", &format!("{} ({} tables, {size_bytes} bytes)", result.path, tables.len()), &secrets);
    Ok(result)
}

/// Keep the `keep` newest backups, delete the rest. Returns how many were removed.
/// keep == 0 means keep everything.
pub fn prune_backups(dir: &Path, keep: u32) -> usize {
    if keep == 0 {
        return 0;
    }
    list_backups(dir) // newest first
        .into_iter()
        .skip(keep as usize)
        .filter(|f| fs::remove_file(&f.path).is_ok())
        .count()
}

/// Headless backup for the scheduled task (`aegis.exe --backup`). No AppHandle —
/// reads settings + resolves paths from %APPDATA%, backs up, prunes, logs.
pub fn run_scheduled() -> Result<String, String> {
    let settings = settings_store::load_headless();
    let dir = match settings.backup_dir.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(custom) => PathBuf::from(custom),
        None => settings_store::aegis_dir()
            .ok_or("Couldn't resolve the backup folder.")?
            .join("backups"),
    };
    fs::create_dir_all(&dir).map_err(|e| format!("Couldn't create the backup folder: {e}"))?;

    match create_backup_to(&settings, &dir, "scheduled") {
        Ok(r) => {
            let pruned = prune_backups(&dir, settings.backup_keep);
            logging::log_headless(
                "INFO",
                "backup/scheduled",
                &format!("{} ({} bytes, footer={}, pruned {pruned})", r.path, r.size_bytes, r.completed),
                &settings.secrets(),
            );
            Ok(format!("Saved {} (removed {pruned} old)", r.path))
        }
        Err(e) => {
            logging::log_headless("ERROR", "backup/scheduled", &format!("failed: {e}"), &settings.secrets());
            Err(e)
        }
    }
}

/// Newest *.sql backup in the folder (path, modified time, size) — for Health.
pub fn newest_backup(dir: &Path) -> Option<(PathBuf, SystemTime, u64)> {
    let entries = fs::read_dir(dir).ok()?;
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "sql"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some((e.path(), meta.modified().ok()?, meta.len()))
        })
        .max_by_key(|(_, t, _)| *t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_detection_on_temp_file() {
        let mut p = std::env::temp_dir();
        p.push("aegis_footer_test.sql");
        fs::write(&p, "CREATE TABLE x;\n-- Dump completed on 2026-05-22\n").unwrap();
        assert!(has_completion_footer(&p));
        fs::write(&p, "CREATE TABLE x; -- truncated").unwrap();
        assert!(!has_completion_footer(&p));
        let _ = fs::remove_file(&p);
    }
}
