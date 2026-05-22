use std::fs;
use std::path::{Path, PathBuf};

use crate::models::DbConnInfo;

/// Locate worldserver.conf: the explicit Repack folder, else derived next to _Server.
pub fn conf_path(server_path: Option<&str>, repack_path: Option<&str>) -> Option<PathBuf> {
    if let Some(repack) = repack_path {
        let p = Path::new(repack).join("worldserver.conf");
        if p.is_file() {
            return Some(p);
        }
    }
    let server = server_path?;
    let p = Path::new(server)
        .parent()? // ...\Database
        .parent()? // ...\<install root>
        .join("Repack")
        .join("worldserver.conf");
    p.is_file().then_some(p)
}

/// Extract the value inside the first quotes for a `Key = "..."` line.
fn conf_value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find(|l| l.trim_start().starts_with(key))
        .and_then(|l| l.split('"').nth(1))
        .map(str::to_string)
}

/// Read the real DB connection (host/port/user/pass) + DB names the server uses.
pub fn read(server_path: Option<&str>, repack_path: Option<&str>) -> Option<DbConnInfo> {
    let text = fs::read_to_string(conf_path(server_path, repack_path)?).ok()?;
    parse(&text)
}

/// Parse the connection out of worldserver.conf text.
/// *DatabaseInfo format: "host;port;user;pass;dbname".
fn parse(text: &str) -> Option<DbConnInfo> {
    let login = conf_value(text, "LoginDatabaseInfo")?;
    let parts: Vec<&str> = login.split(';').collect();
    if parts.len() < 5 {
        return None;
    }

    // DB names from all three connections (deduped, in order login/char/world).
    let mut databases = Vec::new();
    for key in ["LoginDatabaseInfo", "CharacterDatabaseInfo", "WorldDatabaseInfo"] {
        if let Some(name) = conf_value(text, key)
            .and_then(|v| v.split(';').nth(4).map(str::to_string))
            .filter(|s| !s.trim().is_empty())
        {
            if !databases.contains(&name) {
                databases.push(name);
            }
        }
    }

    Some(DbConnInfo {
        host: parts[0].trim().to_string(),
        port: parts[1].trim().parse().unwrap_or(3306),
        user: parts[2].trim().to_string(),
        password: parts[3].trim().to_string(),
        databases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_names_and_creds() {
        // A server where someone changed the defaults — Aegis must read the real values.
        let text = "LoginDatabaseInfo     = \"10.0.0.5;3307;wow;s3cret;myauth\"\nCharacterDatabaseInfo = \"10.0.0.5;3307;wow;s3cret;mychars\"\nWorldDatabaseInfo     = \"10.0.0.5;3307;wow;s3cret;myworld\"\n";
        let info = parse(text).expect("parsed");
        assert_eq!(info.host, "10.0.0.5");
        assert_eq!(info.port, 3307);
        assert_eq!(info.user, "wow");
        assert_eq!(info.password, "s3cret");
        assert_eq!(info.databases, vec!["myauth", "mychars", "myworld"]);
    }

    #[test]
    fn none_when_no_login_line() {
        assert!(parse("# nothing useful here\n").is_none());
    }
}
