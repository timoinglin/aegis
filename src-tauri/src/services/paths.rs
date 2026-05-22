use std::path::Path;

/// Does this look like the repack's "_Server" folder? (has the mysql tools)
pub fn is_server_dir(path: &str) -> bool {
    crate::services::mysql::has_bins(Path::new(path))
}

/// Does this look like the "Repack" folder? (has the worldserver program)
pub fn is_repack_dir(path: &str) -> bool {
    Path::new(path).join("worldserver.exe").is_file()
}

/// Does this look like a WoW client folder? (has Wow.exe)
pub fn is_client_dir(path: &str) -> bool {
    Path::new(path).join("Wow.exe").is_file()
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
}
