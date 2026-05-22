// Dev-only smoke harness for server control:
//   cargo run --example server_exec -- <status|start|stop|restart> <mysql|auth|world> <repack_path>
use aegis_lib::models::Settings;
use aegis_lib::services::server_control::{self, Service};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let action = a.get(1).map(String::as_str).unwrap_or("status");
    let mut s = Settings::default();
    s.server_path = Some(r"C:\mop_repack\MOPPREMIUM\Database\_Server".into());
    if let Some(repack) = a.get(3) {
        s.repack_path = Some(repack.clone());
    }

    if action == "status" {
        let st = server_control::status(&s);
        println!("{st:?}");
        return;
    }

    let svc = Service::parse(a.get(2).map(String::as_str).unwrap_or("auth")).unwrap();
    let r = match action {
        "start" => server_control::start(&s, svc),
        "stop" => server_control::stop(&s, svc),
        "restart" => server_control::restart(&s, svc),
        other => Err(format!("unknown action {other}")),
    };
    match r {
        Ok(m) => println!("OK::{m}"),
        Err(e) => {
            println!("ERR::{e}");
            std::process::exit(1);
        }
    }
}
