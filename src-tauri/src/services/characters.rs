use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use tauri::AppHandle;

use crate::models::{BackupFile, CharacterInfo, Settings};
use crate::services::{backup, logging, mysql, ra, repack_conf};

fn ra_config(s: &Settings) -> ra::RaConfig {
    ra::RaConfig {
        host: s.ra_host.clone(),
        port: s.ra_port,
        user: s.ra_user.clone(),
        password: s.ra_password.clone(),
    }
}

/// The characters DB name from worldserver.conf, else "characters".
fn character_db(settings: &Settings) -> String {
    repack_conf::read(settings.server_path.as_deref(), settings.repack_path.as_deref())
        .and_then(|i| i.databases.into_iter().nth(1))
        .unwrap_or_else(|| "characters".into())
}

fn class_name(id: u32) -> &'static str {
    match id {
        1 => "Warrior",
        2 => "Paladin",
        3 => "Hunter",
        4 => "Rogue",
        5 => "Priest",
        6 => "Death Knight",
        7 => "Shaman",
        8 => "Mage",
        9 => "Warlock",
        10 => "Monk",
        11 => "Druid",
        _ => "Unknown",
    }
}

/// Where character dumps are stored: <backup folder>\characters
fn characters_dir(app: &AppHandle, settings: &Settings) -> Result<PathBuf, String> {
    let dir = backup::resolve_dir(app, settings)?.join("characters");
    fs::create_dir_all(&dir).map_err(|e| format!("Couldn't create the characters folder: {e}"))?;
    Ok(dir)
}

/// The Repack folder = the worldserver's working dir, where `.pdump write` puts files.
fn repack_dir(settings: &Settings) -> Result<PathBuf, String> {
    settings
        .repack_path
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "Set your Repack folder in Settings first — character backups need it.".into())
}

pub fn list(settings: &Settings) -> Vec<CharacterInfo> {
    let db = character_db(settings);
    let sql = format!("SELECT guid, name, level, class, account FROM `{db}`.characters ORDER BY name");
    mysql::query(settings, &sql)
        .map(|out| {
            out.lines()
                .filter_map(|line| {
                    let f: Vec<&str> = line.split('\t').collect();
                    if f.len() < 5 {
                        return None;
                    }
                    Some(CharacterInfo {
                        guid: f[0].trim().parse().ok()?,
                        name: f[1].trim().to_string(),
                        level: f[2].trim().parse().unwrap_or(0),
                        class_name: class_name(f[3].trim().parse().unwrap_or(0)).to_string(),
                        account: f[4].trim().parse().unwrap_or(0),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Back up a single character via `.pdump write`, moving the dump into the backup folder.
pub fn backup_one(app: &AppHandle, settings: &Settings, name_or_guid: &str) -> Result<String, String> {
    let repack = repack_dir(settings)?;
    let dir = characters_dir(app, settings)?;
    let bare = format!("aegis_char_{}.sql", sanitize(name_or_guid));

    let resp = ra::run_command(&ra_config(settings), &format!(".pdump write {bare} {name_or_guid}"))
        .map_err(|e| friendly_ra(&e))?;
    if !resp.to_lowercase().contains("success") {
        return Err(format!("The server couldn't dump that character: {}", resp.trim()));
    }

    let src = repack.join(&bare);
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let dest = dir.join(format!("{}-{stamp}.sql", sanitize(name_or_guid)));
    move_file(&src, &dest)?;

    logging::log_op(app, "INFO", "character/backup", &format!("{name_or_guid} -> {}", dest.display()), &settings.secrets());
    Ok(format!("Backed up {name_or_guid}."))
}

/// Back up every character in one RA session, into a timestamped subfolder.
pub fn backup_all(app: &AppHandle, settings: &Settings) -> Result<String, String> {
    let repack = repack_dir(settings)?;
    let chars = list(settings);
    if chars.is_empty() {
        return Err("No characters found to back up.".into());
    }

    let commands: Vec<String> = chars
        .iter()
        .map(|c| format!(".pdump write aegis_char_{}.sql {}", c.guid, c.guid))
        .collect();
    let responses = ra::run_session(&ra_config(settings), &commands).map_err(|e| friendly_ra(&e))?;

    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let dest_dir = characters_dir(app, settings)?.join(format!("all-{stamp}"));
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let mut ok = 0usize;
    for (c, resp) in chars.iter().zip(responses.iter()) {
        if resp.to_lowercase().contains("success") {
            let src = repack.join(format!("aegis_char_{}.sql", c.guid));
            let dest = dest_dir.join(format!("{}-{}.sql", sanitize(&c.name), c.guid));
            if move_file(&src, &dest).is_ok() {
                ok += 1;
            }
        }
    }

    logging::log_op(app, "INFO", "character/backup_all", &format!("{ok}/{} -> {}", chars.len(), dest_dir.display()), &settings.secrets());
    Ok(format!("Backed up {ok} of {} characters to {}", chars.len(), dest_dir.display()))
}

/// Find a character GUID that's free across EVERY character_* table, not just
/// `characters`. The core's own `.pdump load` picker only checks `characters`,
/// so orphan rows in tables like `character_cuf_profiles` (from old, deleted
/// chars) cause "Transaction failed" duplicate-key errors. Passing this guid
/// explicitly avoids that.
fn safe_new_guid(settings: &Settings) -> u64 {
    let db = character_db(settings);
    let list_sql = format!(
        "SELECT table_name FROM information_schema.columns \
         WHERE table_schema='{db}' AND column_name='guid' AND \
         (table_name='characters' OR table_name LIKE 'character\\_%')"
    );
    let mut max = 0u64;
    if let Ok(out) = mysql::query(settings, &list_sql) {
        for table in out.lines().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let q = format!("SELECT IFNULL(MAX(guid), 0) FROM `{db}`.`{table}`");
            if let Ok(r) = mysql::query(settings, &q) {
                if let Ok(n) = r.trim().parse::<u64>() {
                    if n > max {
                        max = n;
                    }
                }
            }
        }
    }
    max + 1
}

/// Read the source character's name from a `.pdump` dump file. The third
/// single-quoted value of the `INSERT INTO \`characters\`` line is the name.
fn original_name_from_dump(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let f = std::fs::File::open(path).ok()?;
    for line in BufReader::new(f).lines().map_while(Result::ok).take(100) {
        if line.contains("INSERT INTO `characters` VALUES") {
            let after = line.splitn(2, "VALUES (").nth(1)?;
            // Split on `'`: [pre, guid, ", ", account, ", ", name, ...]
            let parts: Vec<&str> = after.splitn(7, '\'').collect();
            if parts.len() >= 6 {
                return Some(parts[5].to_string());
            }
        }
    }
    None
}

/// Import a character dump into an account via `.pdump load`. Aegis computes a
/// safe new GUID and (if not given) the original name from the dump, then passes
/// both explicitly so the import works even when orphan child rows are lurking.
pub fn import(app: &AppHandle, settings: &Settings, file_path: &str, account: &str, new_name: &str) -> Result<String, String> {
    let repack = repack_dir(settings)?;
    let src = Path::new(file_path);
    if !src.is_file() {
        return Err("That character file doesn't exist.".into());
    }
    if account.trim().is_empty() {
        return Err("Enter the account to import the character into.".into());
    }

    // pdump load needs a bare filename in the worldserver's folder.
    let bare = "aegis_import.sql";
    let staged = repack.join(bare);
    fs::copy(src, &staged).map_err(|e| format!("Couldn't stage the file: {e}"))?;

    let final_name = if !new_name.trim().is_empty() {
        new_name.trim().to_string()
    } else {
        original_name_from_dump(src).unwrap_or_else(|| "ImportedChar".into())
    };
    let new_guid = safe_new_guid(settings);
    let cmd = format!(".pdump load {bare} {} {final_name} {new_guid}", account.trim());

    let result = ra::run_command(&ra_config(settings), &cmd).map_err(|e| friendly_ra(&e));
    let _ = fs::remove_file(&staged); // tidy up regardless

    let resp = result?;
    let l = resp.to_lowercase();
    logging::log_op(
        app,
        "INFO",
        "character/import",
        &format!("into {account} as {final_name} (guid {new_guid}): {}", resp.trim()),
        &settings.secrets(),
    );
    if l.contains("success") || l.contains("loaded") {
        Ok(format!("Character imported into account {account} as {final_name}. {}", resp.trim()))
    } else if l.contains("name") && (l.contains("used") || l.contains("taken") || l.contains("exist")) {
        Err(format!("Import failed — that name is already taken. Pick a different new name. ({})", resp.trim()))
    } else if l.contains("fail") || l.contains("error") || l.contains("syntax") || l.contains("incorrect") || l.contains("not exist") {
        Err(format!("Import failed — {}. Try a different account or new name.", resp.trim()))
    } else {
        // pdump load is often terse on success; surface the server's words.
        Ok(format!("Done: {}", resp.trim()))
    }
}

/// Character dump files available to import (under <backup>\characters, recursively).
pub fn list_backups(app: &AppHandle, settings: &Settings) -> Vec<BackupFile> {
    let Ok(root) = characters_dir(app, settings) else { return Vec::new() };
    let mut files = Vec::new();
    collect_sql(&root, &mut files);
    files.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    files
}

fn collect_sql(dir: &Path, out: &mut Vec<BackupFile>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_sql(&p, out);
        } else if p.extension().is_some_and(|x| x == "sql") {
            if let Ok(meta) = entry.metadata() {
                let modified_ms = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                out.push(BackupFile {
                    name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
                    path: p.to_string_lossy().into_owned(),
                    size_bytes: meta.len(),
                    modified_ms,
                });
            }
        }
    }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

fn move_file(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.is_file() {
        return Err(format!("Expected dump file wasn't created: {}", src.display()));
    }
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    // Cross-drive fallback.
    fs::copy(src, dest).map_err(|e| format!("Couldn't move the dump: {e}"))?;
    let _ = fs::remove_file(src);
    Ok(())
}

fn friendly_ra(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.contains("connect") || l.contains("closed") || l.contains("timed out") {
        "Couldn't reach Remote Access. Make sure the worldserver is running.".into()
    } else if l.contains("login") || l.contains("rejected") {
        "Remote Access refused the login. Check the RA user and password in Settings.".into()
    } else {
        format!("Remote Access error: {raw}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_unsafe_chars() {
        assert_eq!(sanitize("Bob"), "Bob");
        assert_eq!(sanitize("a b/c"), "a_b_c");
    }

    #[test]
    fn class_names() {
        assert_eq!(class_name(8), "Mage");
        assert_eq!(class_name(99), "Unknown");
    }
}
