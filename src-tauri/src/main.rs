// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Headless mode for the scheduled task: run a backup and exit, no window.
    if std::env::args().any(|a| a == "--backup") {
        match aegis_lib::services::backup::run_scheduled() {
            Ok(msg) => {
                println!("{msg}");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    aegis_lib::run();
}
