use serde::{Deserialize, Serialize};

/// Worst-of severity for any health check. Serializes to lowercase to match
/// the TS `HealthStatus` union in src/lib/types.ts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Warn,
    Error,
    Unknown,
}

/// The single shape every status/error surface uses.
///
/// Golden rule: `title` / `why` / `fix` are the *friendly* user-facing text.
/// The raw underlying error is never put here — it goes to the op-log
/// (redacted) via `services::logging`, so GitHub-issue debugging stays possible.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheck {
    pub id: String,
    pub category: String,
    pub status: HealthStatus,
    pub title: String,
    pub why: String,
    pub fix: Vec<String>,
    /// false = reserved/dormant slot, wired now and populated when its feature lands.
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub checks: Vec<HealthCheck>,
    pub overall: HealthStatus,
    pub checked_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Container-level default: any field missing from an older settings.json falls back
// to its default instead of failing the whole parse. Future-proofs adding fields.
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    /// The repack's "_Server" folder (contains mysql\bin + MySQL.bat).
    pub server_path: Option<String>,
    /// The "Repack" folder (authserver.exe / worldserver.exe / *.conf).
    pub repack_path: Option<String>,
    /// The WoW client folder (contains Wow.exe + Interface\AddOns).
    pub client_path: Option<String>,
    /// Where backups are written. None = the default (%APPDATA%\Aegis\backups).
    pub backup_dir: Option<String>,
    pub ra_host: String,
    pub ra_port: u16,
    pub ra_user: String,
    pub ra_password: String,
    /// False until the first-run setup wizard has been completed.
    pub setup_complete: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // Documented EmuCoach repack defaults. RA creds are instance-specific
        // (set in worldserver.conf), so we leave them blank for the user to fill.
        Self {
            db_host: "127.0.0.1".into(),
            db_port: 3306,
            db_user: "root".into(),
            db_password: "ascent".into(),
            server_path: None,
            repack_path: None,
            client_path: None,
            backup_dir: None,
            ra_host: "127.0.0.1".into(),
            ra_port: 3443,
            ra_user: String::new(),
            ra_password: String::new(),
            setup_complete: false,
        }
    }
}

impl Settings {
    /// Secrets that must be masked before anything reaches the op-log.
    pub fn secrets(&self) -> Vec<String> {
        [self.db_password.clone(), self.ra_password.clone()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Per-database summary captured at backup time (the row-count sanity check).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbBackupInfo {
    pub name: String,
    pub tables: u64,
    pub approx_rows: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
    pub size_bytes: u64,
    /// True when the dump file ends with mysqldump's completion footer.
    pub completed: bool,
    pub databases: Vec<DbBackupInfo>,
    pub duration_secs: u64,
}

/// Running state of one server process.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceState {
    pub running: bool,
    pub pid: Option<u32>,
    /// True if Aegis knows where to launch it (path configured + exe present).
    pub launchable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub mysql: ServiceState,
    pub authserver: ServiceState,
    pub worldserver: ServiceState,
}

/// A backup file Aegis can offer to restore.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    /// Unix millis of last modification (newest first in the UI).
    pub modified_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored_from: String,
    /// The automatic safety backup taken right before the restore.
    pub safety_backup_path: String,
    pub databases: Vec<DbBackupInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_with_camelcase_keys() {
        let json = serde_json::to_string(&Settings::default()).unwrap();
        // Frontend (src/lib/types.ts) expects camelCase.
        assert!(json.contains("dbHost") && json.contains("serverPath") && json.contains("raPort"));
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.db_host, "127.0.0.1");
        assert_eq!(back.db_port, 3306);
        assert!(back.server_path.is_none());
    }

    #[test]
    fn secrets_excludes_empty() {
        let mut s = Settings::default(); // ra_password is empty by default
        let sec = s.secrets();
        assert!(sec.contains(&"ascent".to_string()));
        assert!(!sec.iter().any(|x| x.is_empty()));
        s.ra_password = "rapass".into();
        assert!(s.secrets().contains(&"rapass".to_string()));
    }
}
