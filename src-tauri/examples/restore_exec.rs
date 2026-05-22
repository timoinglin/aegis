// Dev-only smoke harness for restore internals (no AppHandle needed):
//   cargo run --example restore_exec -- preflight
//   cargo run --example restore_exec -- run <server_path> <sql_file>
use aegis_lib::models::Settings;
use aegis_lib::services::restore::{preflight_servers_stopped, run_sql_file};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("preflight") => match preflight_servers_stopped() {
            Ok(()) => println!("PREFLIGHT_OK (servers stopped — restore would be allowed)"),
            Err(e) => println!("PREFLIGHT_BLOCKED::{e}"),
        },
        Some("run") if a.len() >= 4 => {
            let mut s = Settings::default();
            s.server_path = Some(a[2].clone());
            match run_sql_file(&s, std::path::Path::new(&a[3])) {
                Ok(()) => println!("IMPORT_OK"),
                Err(e) => {
                    println!("IMPORT_ERR::{e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage: restore_exec preflight | run <server_path> <sql_file>");
            std::process::exit(2);
        }
    }
}
