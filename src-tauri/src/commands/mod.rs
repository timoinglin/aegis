// The IPC surface: thin #[tauri::command] wrappers. All real work lives in
// services::*. Keep these functions boring — parse args, call a service, return.

use tauri::AppHandle;

use crate::models::{BackupFile, BackupResult, HealthReport, RestoreResult, Settings};
use crate::services::{accounts, backup, health, restore, settings_store};

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
