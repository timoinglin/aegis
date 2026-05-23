// The IPC surface: thin #[tauri::command] wrappers. All real work lives in
// services::*. Keep these functions boring — parse args, call a service, return.

use tauri::AppHandle;

use crate::models::{AddonInfo, BackupFile, BackupResult, CharacterInfo, DbConnInfo, DbSize, HealthReport, MaintenanceResult, RestoreResult, ScheduleStatus, ServerStatus, Settings};
use crate::services::{accounts, addons, backup, characters, health, logging, maintenance, paths, ra, repack_conf, restore, scheduler, server_control, settings_store};
use crate::services::server_control::Service;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    settings_store::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    settings_store::save(&app, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn autodetect_server_path(repack_path: Option<String>) -> Option<String> {
    settings_store::autodetect_server_path(repack_path.as_deref())
}

#[tauri::command]
pub fn autodetect_repack_path(server_path: Option<String>) -> Option<String> {
    settings_store::autodetect_repack_path(server_path.as_deref())
}

#[tauri::command]
pub fn autodetect_client_path() -> Option<String> {
    settings_store::autodetect_client_path()
}

/// Verify a folder really is what we expect. kind: "server" | "repack" | "client".
#[tauri::command]
pub fn validate_path(kind: String, path: String) -> bool {
    paths::validate(&kind, &path)
}

/// Real Remote Access test: actually log in with the given credentials and run a
/// harmless read-only command. Distinguishes unreachable / rejected / connected,
/// so the wizard doesn't claim success when only the port is open.
#[tauri::command]
pub async fn test_remote_access(host: String, port: u16, user: String, password: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if user.trim().is_empty() || password.is_empty() {
            return Err("Enter a Remote Access user and password to test the login.".to_string());
        }
        let cfg = ra::RaConfig { host, port, user, password };
        match ra::run_command(&cfg, ".server info") {
            Ok(_) => Ok("Connected — login works!".to_string()),
            Err(e) => {
                let l = e.to_lowercase();
                Err(if l.contains("connect") || l.contains("resolve") {
                    "Couldn't reach Remote Access. Is the worldserver running and Ra.Enable set to 1?".into()
                } else if l.contains("rejected") || l.contains("login") {
                    "Reachable, but the login was rejected — check the user and password.".into()
                } else if l.contains("timed out") || l.contains("closed") {
                    "Reachable, but the server didn't finish the login in time.".into()
                } else {
                    format!("Couldn't log in: {e}")
                })
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Read the real DB host/port/user/password + DB names from worldserver.conf,
/// so changed defaults (renamed DBs, different password) are picked up automatically.
#[tauri::command]
pub fn read_db_config(server_path: Option<String>, repack_path: Option<String>) -> Option<DbConnInfo> {
    repack_conf::read(server_path.as_deref(), repack_path.as_deref())
}

#[tauri::command]
pub fn run_health_checks(app: AppHandle) -> HealthReport {
    health::report(&app)
}

#[tauri::command]
pub fn recheck(app: AppHandle, id: String) -> HealthReport {
    health::recheck(&app, &id)
}

// Account ops talk to the worldserver over RA (blocking I/O up to ~10s), so they
// run on the blocking pool to keep the UI responsive.

#[tauri::command]
pub async fn create_account(
    app: AppHandle,
    username: String,
    password: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        accounts::create_account(&app, &settings, &username, &password)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_gm_level(
    app: AppHandle,
    username: String,
    level: u8,
    realm_id: i32,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        accounts::set_gm_level(&app, &settings, &username, level, realm_id)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Delete an account (and its characters) via RA. Destructive — UI must confirm.
#[tauri::command]
pub async fn delete_account(app: AppHandle, username: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        accounts::delete_account(&app, &settings, &username)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Direct-DB GM level override (skips RA when it refuses with "low security").
#[tauri::command]
pub async fn set_gm_level_direct(
    app: AppHandle,
    username: String,
    level: u8,
    realm_id: i32,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        accounts::set_gm_level_direct(&app, &settings, &username, level, realm_id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_account_password(
    app: AppHandle,
    username: String,
    new_password: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        accounts::set_password(&app, &settings, &username, &new_password)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The resolved backup folder, shown in the UI so the user knows where files land.
#[tauri::command]
pub fn backup_dir(app: AppHandle) -> String {
    let settings = settings_store::load(&app);
    backup::resolve_dir(&app, &settings)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Full DB backup (mysqldump). Long-running, so it goes on the blocking pool.
#[tauri::command]
pub async fn create_backup(app: AppHandle) -> Result<BackupResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        backup::create_backup(&app, &settings, "backup")
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Current state of the automatic-backup scheduled task.
#[tauri::command]
pub fn schedule_status(app: AppHandle) -> ScheduleStatus {
    let settings = settings_store::load(&app);
    scheduler::status(&settings)
}

/// Save settings, then make the Windows scheduled task match them.
#[tauri::command]
pub fn apply_schedule(app: AppHandle, settings: Settings) -> Result<ScheduleStatus, String> {
    settings_store::save(&app, &settings)?;
    let status = scheduler::apply(&settings)?;
    logging::log_op(&app, "INFO", "schedule", &format!("enabled={} {}", settings.backup_schedule_enabled, status.summary), &settings.secrets());
    Ok(status)
}

/// Back up only the registration-portal (web_*) tables.
#[tauri::command]
pub async fn create_web_backup(app: AppHandle) -> Result<BackupResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        backup::create_web_backup(&app, &settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Backups available to restore, newest first.
#[tauri::command]
pub fn list_backups(app: AppHandle) -> Vec<BackupFile> {
    let settings = settings_store::load(&app);
    match backup::resolve_dir(&app, &settings) {
        Ok(dir) => backup::list_backups(&dir),
        Err(_) => Vec::new(),
    }
}

// --- Add-ons ---

#[tauri::command]
pub async fn list_addons(app: AppHandle) -> Vec<AddonInfo> {
    // Hits GitHub for latest versions, so off the main thread.
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        addons::status(&app, &settings)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub fn addon_thumbnail(app: AppHandle, id: String) -> Option<String> {
    addons::thumbnail(&app, &id)
}

#[tauri::command]
pub async fn uninstall_addon(app: AppHandle, id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        let result = addons::uninstall(&app, &settings, &id);
        match &result {
            Ok(msg) => logging::log_op(&app, "INFO", "addon/uninstall", &format!("{id}: {msg}"), &settings.secrets()),
            Err(e) => logging::log_op(&app, "ERROR", "addon/uninstall", &format!("{id}: {e}"), &settings.secrets()),
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_addon(app: AppHandle, id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        let result = addons::install(&app, &settings, &id);
        match &result {
            Ok(msg) => logging::log_op(&app, "INFO", "addon/install", &format!("{id}: {msg}"), &settings.secrets()),
            Err(e) => logging::log_op(&app, "ERROR", "addon/install", &format!("{id}: {e}"), &settings.secrets()),
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Database upkeep via mysqlcheck. mode: "analyze" | "optimize" | "repair".
#[tauri::command]
pub async fn db_maintenance(app: AppHandle, mode: String) -> Result<MaintenanceResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        maintenance::run(&app, &settings, &mode)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// On-disk size + table count for each game database. Used by the Maintenance summary.
#[tauri::command]
pub async fn db_sizes(app: AppHandle) -> Vec<DbSize> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        maintenance::db_sizes(&settings)
    })
    .await
    .unwrap_or_default()
}

/// Open a local file with the OS default app (e.g. Notepad for .conf). Done in
/// Rust because the opener plugin's `openPath` silently no-ops without a path
/// scope, and a Rust shell-exec is rock-solid on Windows.
#[tauri::command]
pub fn open_in_default_app(path: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("That file doesn't exist: {path}"));
    }
    // `cmd /c start "" "<path>"` is the canonical Windows "open with default" call.
    // The empty "" is the window title (start treats the first quoted arg as one).
    Command::new("cmd")
        .args(["/c", "start", "", &path])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Couldn't open the file: {e}"))
}

// --- Characters (.pdump via Remote Access) ---

#[tauri::command]
pub async fn list_characters(app: AppHandle) -> Vec<CharacterInfo> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        characters::list(&settings)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn backup_character(app: AppHandle, name_or_guid: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        characters::backup_one(&app, &settings, &name_or_guid)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn backup_all_characters(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        characters::backup_all(&app, &settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn list_character_backups(app: AppHandle) -> Vec<BackupFile> {
    let settings = settings_store::load(&app);
    characters::list_backups(&app, &settings)
}

#[tauri::command]
pub async fn import_character(app: AppHandle, path: String, account: String, new_name: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        characters::import(&app, &settings, &path, &account, &new_name)
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Server process control ---

#[tauri::command]
pub fn server_status(app: AppHandle) -> ServerStatus {
    let settings = settings_store::load(&app);
    server_control::status(&settings)
}

#[tauri::command]
pub async fn server_action(app: AppHandle, service: String, action: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        if action == "start_all" {
            return server_control::start_all(&settings);
        }
        let svc = Service::parse(&service)?;
        let result = match action.as_str() {
            "start" => server_control::start(&settings, svc),
            "stop" => server_control::stop(&settings, svc),
            "restart" => server_control::restart(&settings, svc),
            other => Err(format!("Unknown action: {other}")),
        };
        if let Ok(msg) = &result {
            crate::services::logging::log_op(&app, "INFO", "server", &format!("{action} {service}: {msg}"), &settings.secrets());
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Restore a backup file. Destructive — takes an automatic safety backup first,
/// requires the typed confirmation, and refuses if the server is running.
#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    path: String,
    confirmation: String,
) -> Result<RestoreResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        restore::restore(&app, &settings, &path, &confirmation)
    })
    .await
    .map_err(|e| e.to_string())?
}
