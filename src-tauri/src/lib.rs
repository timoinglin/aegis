mod commands;
pub mod models;
pub mod services;

/// App entry point, called from main.rs.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::autodetect_server_path,
            commands::run_health_checks,
            commands::recheck,
            commands::create_account,
            commands::set_gm_level,
            commands::set_account_password,
            commands::backup_dir,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Aegis");
}
