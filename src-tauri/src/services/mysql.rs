use std::path::{Path, PathBuf};
use std::process::Command;

use crate::models::Settings;

/// Resolve the path to a bundled mysql tool, e.g. `bin("mysqldump")`.
pub fn bin(server_path: &str, exe: &str) -> PathBuf {
    Path::new(server_path)
        .join("mysql")
        .join("bin")
        .join(format!("{exe}.exe"))
}

/// True if the given "_Server" folder actually contains the mysql client tools.
pub fn has_bins(server_path: &Path) -> bool {
    server_path
        .join("mysql")
        .join("bin")
        .join("mysqldump.exe")
        .is_file()
}

/// Build a Command for a bundled tool with the connection flags applied. The
/// password goes through MYSQL_PWD (env), not the command line — so it never
/// shows up in the process list or in mysql's "insecure password" warning.
pub fn base_cmd(settings: &Settings, exe: &str) -> Result<Command, String> {
    let server_path = settings
        .server_path
        .as_deref()
        .ok_or("The repack folder isn't set yet.")?;
    let tool = bin(server_path, exe);
    if !tool.is_file() {
        return Err(format!("{exe}.exe not found at {}", tool.display()));
    }
    let mut cmd = Command::new(tool);
    cmd.arg(format!("-h{}", settings.db_host))
        .arg(format!("-P{}", settings.db_port))
        .arg(format!("-u{}", settings.db_user))
        .env("MYSQL_PWD", &settings.db_password);
    Ok(cmd)
}

/// Run a `SELECT 1` to confirm the connection. Err carries the raw stderr.
pub fn ping(settings: &Settings) -> Result<(), String> {
    let output = base_cmd(settings, "mysql")?
        .arg("-N")
        .arg("-e")
        .arg("SELECT 1")
        .output()
        .map_err(|e| format!("failed to launch mysql.exe: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// Run a query and return stdout as tab-separated, newline-delimited rows (-N -B).
pub fn query(settings: &Settings, sql: &str) -> Result<String, String> {
    let output = base_cmd(settings, "mysql")?
        .arg("-N")
        .arg("-B")
        .arg("-e")
        .arg(sql)
        .output()
        .map_err(|e| format!("failed to launch mysql.exe: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}
