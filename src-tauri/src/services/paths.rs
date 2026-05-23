use std::fs;
use std::path::Path;

/// Does this look like the repack's "_Server" folder? (has the mysql tools)
pub fn is_server_dir(path: &str) -> bool {
    crate::services::mysql::has_bins(Path::new(path))
}

/// Does this look like the "Repack" folder? (has the worldserver program)
pub fn is_repack_dir(path: &str) -> bool {
    Path::new(path).join("worldserver.exe").is_file()
}

/// Does this look like a WoW client folder? Different builds rename the exe
/// (Wow.exe, Wow_64.exe, WowClassic.exe, Wow_wod.exe, …), so the reliable
/// signal is "an exe starting with 'wow' next to an Interface\AddOns folder".
/// Fall back to the AddOns folder alone — that's the bit Aegis actually uses.
pub fn is_client_dir(path: &str) -> bool {
    let p = Path::new(path);
    let has_addons = p.join("Interface").join("AddOns").is_dir();
    let has_wow_exe = has_wow_exe_in(p);
    // AddOns alone is enough — that's what Aegis installs into; the exe is just a sanity check.
    has_addons || has_wow_exe
}

/// True if the folder contains *any* file named `wow*.exe` (case-insensitive).
pub fn has_wow_exe_in(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else { return false };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        if let Some(name) = entry.file_name().to_str().map(str::to_lowercase) {
            if name.starts_with("wow") && name.ends_with(".exe") {
                return true;
            }
        }
    }
    false
}

/// Validate a path by kind: "server" | "repack" | "client".
pub fn validate(kind: &str, path: &str) -> bool {
    if path.trim().is_empty() {
        return false;
    }
    match kind {
        "server" => is_server_dir(path),
        "repack" => is_repack_dir(path),
        "client" => is_client_dir(path),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_unknown_are_invalid() {
        assert!(!validate("server", ""));
        assert!(!validate("server", "   "));
        assert!(!validate("nope", "C:\\anything"));
    }

    #[test]
    fn validates_on_this_machine_when_present() {
        let repack = r"C:\mop_repack\MOPPREMIUM\Repack";
        if Path::new(repack).join("worldserver.exe").is_file() {
            assert!(validate("repack", repack));
            assert!(!validate("repack", r"C:\Windows")); // wrong folder
        }
    }

    #[test]
    fn client_dir_accepts_any_wow_exe_variant() {
        // Build a fake client folder with a "Wow_64.exe" — should pass even though
        // there's no "Wow.exe" and no Interface\AddOns.
        let tmp = std::env::temp_dir().join("aegis_test_client_alt");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("Wow_64.exe"), b"").unwrap();
        assert!(is_client_dir(tmp.to_str().unwrap()));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn client_dir_accepts_addons_folder_only() {
        // No exe at all, but Interface\AddOns exists — still a usable client for
        // the only thing Aegis writes there.
        let tmp = std::env::temp_dir().join("aegis_test_client_addons");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("Interface").join("AddOns")).unwrap();
        assert!(is_client_dir(tmp.to_str().unwrap()));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn client_dir_rejects_random_folder() {
        let tmp = std::env::temp_dir().join("aegis_test_not_a_client");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        assert!(!is_client_dir(tmp.to_str().unwrap()));
        let _ = fs::remove_dir_all(&tmp);
    }
}
