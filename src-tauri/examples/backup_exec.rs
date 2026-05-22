// Dev-only smoke harness: runs Aegis's real backup engine against a live server.
// Usage: cargo run --example backup_exec -- <server_path> <out_dir>
use aegis_lib::models::Settings;
use aegis_lib::services::backup::create_backup_to;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: backup_exec <server_path> <out_dir>");
        std::process::exit(2);
    }
    let mut s = Settings::default(); // db creds default to root/ascent
    s.server_path = Some(a[1].clone());

    match create_backup_to(&s, std::path::Path::new(&a[2]), "test") {
        Ok(r) => {
            println!("path={}", r.path);
            println!("size_bytes={}", r.size_bytes);
            println!("completed={}", r.completed);
            println!("duration_secs={}", r.duration_secs);
            for d in r.databases {
                println!("db {}: {} tables, ~{} rows", d.name, d.tables, d.approx_rows);
            }
        }
        Err(e) => {
            println!("ERR::{e}");
            std::process::exit(1);
        }
    }
}
