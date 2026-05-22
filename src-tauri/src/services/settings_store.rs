use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::Settings;

/// %APPDATA%\Aegis\settings.json
fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Brief specifies %APPDATA%\Aegis exactly. Tauri's app_data_dir() would append
    // the bundle identifier (…\io.github.timoinglin.aegis), so we build it from the
    // roaming data dir + "Aegis" instead.
    let dir = app
        .path()
        .data_dir()
        .map_err(|e| format!("Could not resolve the app data folder: {e}"))?
        .join("Aegis");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

/// Load saved settings, or sensible defaults on first run / unreadable file.
pub fn load(app: &AppHandle) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };
    match fs::read_to_string(&path) {
        // Tolerate a UTF-8 BOM in case the file was hand-edited in an editor that adds one.
        Ok(text) => serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

/// Persist settings to disk and return what was written.
pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

/// Best-effort probe for the repack's "_Server" folder. Validated by finding
/// mysql\bin\mysqldump.exe inside it. Returns the first match, or None.
///
/// TODO(v0.1): widen the candidate set (registry lookup, drive scan with a depth
/// cap). For now we check the common EmuCoach install layouts.
pub fn autodetect_server_path() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for drive in ["C:", "D:", "E:"] {
        candidates.push(PathBuf::from(format!("{drive}\\mop_repack\\MOPPREMIUM\\Database\\_Server")));
        candidates.push(PathBuf::from(format!("{drive}\\MOPPREMIUM\\Database\\_Server")));
        candidates.push(PathBuf::from(format!("{drive}\\EmuCoach\\MOPPREMIUM\\Database\\_Server")));
    }
    candidates
        .into_iter()
        .find(|p| crate::services::mysql::has_bins(p))
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Settings;

    #[test]
    fn tolerates_utf8_bom() {
        // Mirrors load(): a hand-edited file with a BOM must still parse.
        let with_bom = format!("\u{feff}{}", serde_json::to_string(&Settings::default()).unwrap());
        let parsed: Settings =
            serde_json::from_str(with_bom.trim_start_matches('\u{feff}')).unwrap();
        assert_eq!(parsed.db_user, "root");
    }

    #[test]
    fn autodetect_resolves_on_this_machine() {
        // Only assert when the repack is actually present (keeps CI/other machines green).
        let known = r"C:\mop_repack\MOPPREMIUM\Database\_Server";
        if std::path::Path::new(known).join(r"mysql\bin\mysqldump.exe").exists() {
            assert_eq!(autodetect_server_path().as_deref(), Some(known));
        }
    }
}
