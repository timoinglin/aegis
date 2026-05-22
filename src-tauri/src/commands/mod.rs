// The IPC surface: thin #[tauri::command] wrappers. All real work lives in
// services::*. Keep these functions boring — parse args, call a service, return.

use tauri::AppHandle;

use crate::models::{AddonInfo, BackupFile, BackupResult, DbConnInfo, HealthReport, RestoreResult, ServerStatus, Settings};
use crate::services::{accounts, addons, backup, health, logging, repack_conf, restore, server_control, settings_store};
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
pub fn autodetect_server_path() -> Option<String> {
    settings_store::autodetect_server_path()
}

#[tauri::command]
pub fn autodetect_repack_path(server_path: Option<String>) -> Option<String> {
    settings_store::autodetect_repack_path(server_path.as_deref())
}

#[tauri::command]
pub fn autodetect_client_path() -> Option<String> {
    settings_store::autodetect_client_path()
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
        addons::status(&settings)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn install_addon(app: AppHandle, id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = settings_store::load(&app);
        let result = addons::install(&settings, &id);
        match &result {
            Ok(msg) => logging::log_op(&app, "INFO", "addon/install", &format!("{id}: {msg}"), &settings.secrets()),
            Err(e) => logging::log_op(&app, "ERROR", "addon/install", &format!("{id}: {e}"), &settings.secrets()),
        }
        result
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
