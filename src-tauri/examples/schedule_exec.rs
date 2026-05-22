// Dev-only smoke harness for the Task Scheduler integration:
//   cargo run --example schedule_exec -- <on|off|status>
use aegis_lib::models::Settings;
use aegis_lib::services::scheduler;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "status".into());
    let mut s = Settings::default();
    s.backup_time = "03:00".into();
    s.backup_schedule_enabled = mode == "on";

    let result = match mode.as_str() {
        "status" => Ok(scheduler::status(&s)),
        "on" | "off" => scheduler::apply(&s),
        other => Err(format!("unknown mode {other}")),
    };
    match result {
        Ok(st) => println!("OK enabled={} summary={:?} next={:?}", st.enabled, st.summary, st.next_run),
        Err(e) => {
            println!("ERR::{e}");
            std::process::exit(1);
        }
    }
}
