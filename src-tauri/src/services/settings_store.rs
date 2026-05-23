use std::fs;
use std::path::{Path, PathBuf};

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

/// %APPDATA%\Aegis resolved without a Tauri AppHandle (for headless --backup mode).
/// Matches what app.path().data_dir() returns on Windows (roaming AppData).
pub fn aegis_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Aegis"))
}

/// Load settings directly from disk, no AppHandle — for the --backup CLI path.
pub fn load_headless() -> Settings {
    let Some(dir) = aegis_dir() else {
        return Settings::default();
    };
    match fs::read_to_string(dir.join("settings.json")) {
        Ok(text) => serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
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

/// Best-effort probe for the repack's "_Server" folder. Prefer deriving from a
/// known Repack path (sibling layout: `<root>\Repack` ⇔ `<root>\Database\_Server`),
/// fall back to a bounded drive scan.
pub fn autodetect_server_path(repack_path: Option<&str>) -> Option<String> {
    // ...\Repack -> ...\Database\_Server (sibling under the install root).
    if let Some(root) = repack_path.and_then(|rp| Path::new(rp).parent()) {
        let server = root.join("Database").join("_Server");
        if crate::services::mysql::has_bins(&server) {
            return Some(server.to_string_lossy().into_owned());
        }
    }
    let rels = ["Database\\_Server", "_Server"];
    scan_drives(&rels, |p| crate::services::mysql::has_bins(p))
}

/// Find the "Repack" folder (authserver.exe / worldserver.exe). Prefer deriving
/// it from the known _Server path (it's a sibling under the install root); fall
/// back to a generic drive scan.
pub fn autodetect_repack_path(server_path: Option<&str>) -> Option<String> {
    // ...\Database\_Server -> ...\<install root>\Repack
    if let Some(root) = server_path.and_then(|sp| Path::new(sp).parent().and_then(Path::parent)) {
        let repack = root.join("Repack");
        if repack.join("worldserver.exe").is_file() {
            return Some(repack.to_string_lossy().into_owned());
        }
    }
    let rels = ["Repack"];
    scan_drives(&rels, |p| p.join("worldserver.exe").is_file())
}

/// Probe `<drive>\<a>\<rel>` AND `<drive>\<a>\<b>\<rel>` against `matches` —
/// covers `C:\Repack` and `C:\Games\MyMoP\Repack` alike without paying the
/// cost of unbounded recursion. Returns the first hit.
fn scan_drives(rels: &[&str], matches: impl Fn(&Path) -> bool) -> Option<String> {
    let try_rels = |base: &Path, matches: &dyn Fn(&Path) -> bool| -> Option<String> {
        for rel in rels {
            let candidate = base.join(rel);
            if matches(&candidate) {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
        None
    };
    for drive in ["C:", "D:", "E:", "F:"] {
        let Ok(level1) = fs::read_dir(format!("{drive}\\")) else { continue };
        for a in level1.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
            if let Some(hit) = try_rels(&a, &matches) {
                return Some(hit);
            }
            // One folder deeper, e.g. C:\Games\MyMoP\Database\_Server.
            let Ok(level2) = fs::read_dir(&a) else { continue };
            for b in level2.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
                if let Some(hit) = try_rels(&b, &matches) {
                    return Some(hit);
                }
            }
        }
    }
    None
}

/// Best-effort guess for the WoW client folder. Scans drive roots for a folder
/// whose name looks like a MoP client and that contains either an Interface\AddOns
/// folder or any `wow*.exe` (different builds rename the exe).
pub fn autodetect_client_path() -> Option<String> {
    use crate::services::paths;
    let looks_like_client = |name: &str| {
        let n = name.to_lowercase();
        n.contains("pandaria") || n.contains("mists") || n.contains("5.4.8") || n.contains("548")
    };
    for drive in ["C:", "D:", "E:", "F:"] {
        let Ok(entries) = fs::read_dir(format!("{drive}\\")) else { continue };
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else { continue };
            if !looks_like_client(name) {
                continue;
            }
            if let Some(s) = p.to_str() {
                if paths::is_client_dir(s) {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
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
            assert_eq!(autodetect_server_path(None).as_deref(), Some(known));
        }
    }

    #[test]
    fn server_derives_from_repack_sibling() {
        // If Repack is at <root>\Repack, _Server is at <root>\Database\_Server.
        let repack = r"C:\mop_repack\MOPPREMIUM\Repack";
        let expected = r"C:\mop_repack\MOPPREMIUM\Database\_Server";
        if std::path::Path::new(expected).join(r"mysql\bin\mysqldump.exe").exists()
            && std::path::Path::new(repack).is_dir()
        {
            assert_eq!(autodetect_server_path(Some(repack)).as_deref(), Some(expected));
        }
    }
}
