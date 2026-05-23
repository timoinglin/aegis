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
/// fall back to a bounded content-based drive scan that finds any folder with
/// mysql\bin\mysqldump.exe inside, no matter the folder name.
pub fn autodetect_server_path(repack_path: Option<&str>) -> Option<String> {
    // ...\Repack -> ...\Database\_Server (sibling under the install root).
    if let Some(root) = repack_path.and_then(|rp| Path::new(rp).parent()) {
        let server = root.join("Database").join("_Server");
        if crate::services::mysql::has_bins(&server) {
            return Some(server.to_string_lossy().into_owned());
        }
    }
    scan_drives(|p| crate::services::mysql::has_bins(p))
}

/// Find the "Repack" folder (authserver.exe / worldserver.exe). Prefer deriving
/// it from the known _Server path (sibling under the install root); fall back
/// to a content-based scan for any folder containing worldserver.exe.
pub fn autodetect_repack_path(server_path: Option<&str>) -> Option<String> {
    // ...\Database\_Server -> ...\<install root>\Repack
    if let Some(root) = server_path.and_then(|sp| Path::new(sp).parent().and_then(Path::parent)) {
        let repack = root.join("Repack");
        if repack.join("worldserver.exe").is_file() {
            return Some(repack.to_string_lossy().into_owned());
        }
    }
    scan_drives(|p| p.join("worldserver.exe").is_file())
}

/// Bounded depth-first scan of each drive looking for the first folder where
/// `matches` returns true. Goes 4 levels deep (enough for `C:\X\Y\Z\_Server`
/// without paying for an unbounded recursion); skips obvious system folders so
/// it doesn't waste time walking C:\Windows or C:\Users.
///
/// Content-based by design — we look at the FILES in a folder, not its name —
/// so a user-renamed `Database_c\_Server` is found just as easily as the stock
/// `Database\_Server`.
fn scan_drives(matches: impl Fn(&Path) -> bool) -> Option<String> {
    const MAX_DEPTH: u32 = 4;
    let is_noise = |name: &str| {
        let n = name.to_lowercase();
        matches!(
            n.as_str(),
            "windows"
            | "program files"
            | "program files (x86)"
            | "programdata"
            | "users"
            | "$recycle.bin"
            | "system volume information"
            | "perflogs"
            | "recovery"
            | "intel"
            | "appdata"
            | "boot"
            | "msocache"
            | "config.msi"
        ) || n.starts_with('$')
    };

    for drive in ["C:", "D:", "E:", "F:", "G:", "H:"] {
        let root = PathBuf::from(format!("{drive}\\"));
        if fs::read_dir(&root).is_err() {
            continue;
        }
        // DFS (LIFO via stack); first hit wins.
        let mut stack: Vec<(PathBuf, u32)> = vec![(root, 0)];
        while let Some((dir, depth)) = stack.pop() {
            // Don't run `matches` on the drive root itself — every drive would visit it.
            if depth > 0 && matches(&dir) {
                return Some(dir.to_string_lossy().into_owned());
            }
            if depth >= MAX_DEPTH {
                continue;
            }
            let Ok(entries) = fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    if is_noise(name) {
                        continue;
                    }
                }
                stack.push((p, depth + 1));
            }
        }
    }
    None
}

/// Best-effort guess for the WoW client folder. Two-stage so we don't pick a
/// model-dump or side-install when the user has a primary client folder:
///   1) Prefer a folder whose NAME hints at the right client (pandaria / mists /
///      5.4.8 / wow / warcraft) AND has WoW install signals — that's the typical
///      "Mists of Pandaria 5.4.8" folder.
///   2) Fall back to any folder with WoW install signals if no named match found.
/// Renamed clients are best picked via the Browse button.
pub fn autodetect_client_path() -> Option<String> {
    use crate::services::paths;
    let has_strong_signals = |p: &Path| -> bool {
        if !paths::has_wow_exe_in(p) {
            return false;
        }
        p.join("Data").is_dir() || p.join("Interface").join("AddOns").is_dir() || p.join("WTF").is_dir()
    };
    let name_hints_wow_mop = |p: &Path| -> bool {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase())
            .is_some_and(|n| {
                n.contains("pandaria")
                    || n.contains("mists")
                    || n.contains("5.4.8")
                    || n.contains("548")
                    || n.contains("warcraft")
                    || (n.starts_with("wow") && !n.contains("model"))
            })
    };
    // Pass 1: name hint + content.
    if let Some(hit) = scan_drives(|p| name_hints_wow_mop(p) && has_strong_signals(p)) {
        return Some(hit);
    }
    // Pass 2: content only — for clients with totally renamed folders.
    scan_drives(has_strong_signals)
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
