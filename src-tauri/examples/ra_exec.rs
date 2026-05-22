// Dev-only smoke harness: runs ONE command through Aegis's real RA client against
// a live server. Not shipped. Usage:
//   cargo run --example ra_exec -- <host> <port> <user> <pass> "<command>"
use aegis_lib::services::ra::{run_command, RaConfig};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 6 {
        eprintln!("usage: ra_exec <host> <port> <user> <pass> \"<command>\"");
        std::process::exit(2);
    }
    let cfg = RaConfig {
        host: a[1].clone(),
        port: a[2].parse().expect("port"),
        user: a[3].clone(),
        password: a[4].clone(),
    };
    match run_command(&cfg, &a[5]) {
        Ok(resp) => println!("OK::{resp}"),
        Err(e) => {
            println!("ERR::{e}");
            std::process::exit(1);
        }
    }
}
