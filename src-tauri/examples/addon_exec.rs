// Dev-only smoke harness for the add-on installer:
//   cargo run --example addon_exec -- <status|install> <client_path>
use aegis_lib::models::Settings;
use aegis_lib::services::addons;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mode = a.get(1).map(String::as_str).unwrap_or("status");
    let mut s = Settings::default();
    if let Some(client) = a.get(2) {
        s.client_path = Some(client.clone());
    }
    match mode {
        "status" => {
            for info in addons::status(&s) {
                println!("{info:?}");
            }
        }
        "install" => match addons::install(&s, "mop_gm") {
            Ok(m) => println!("OK::{m}"),
            Err(e) => {
                println!("ERR::{e}");
                std::process::exit(1);
            }
        },
        other => eprintln!("unknown mode {other}"),
    }
}
