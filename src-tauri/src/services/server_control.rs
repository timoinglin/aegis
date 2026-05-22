use std::path::PathBuf;
use std::time::Duration;

use crate::models::{ServerStatus, ServiceState, Settings};
use crate::services::{ra, server_state};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Mysql,
    Auth,
    World,
}

impl Service {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "mysql" | "mysqld" => Ok(Service::Mysql),
            "auth" | "authserver" => Ok(Service::Auth),
            "world" | "worldserver" => Ok(Service::World),
            other => Err(format!("Unknown service: {other}")),
        }
    }

    pub fn image(self) -> &'static str {
        match self {
            Service::Mysql => "mysqld.exe",
            Service::Auth => "authserver.exe",
            Service::World => "worldserver.exe",
        }
    }

    pub fn display(self) -> &'static str {
        match self {
            Service::Mysql => "MySQL",
            Service::Auth => "Authserver",
            Service::World => "Worldserver",
        }
    }

    /// How long to wait after launch before verifying it came up.
    fn startup_wait(self) -> Duration {
        match self {
            Service::Mysql => Duration::from_millis(3000),
            Service::Auth => Duration::from_millis(2000),
            Service::World => Duration::from_millis(5000), // loads maps, etc.
        }
    }
}

/// (executable-or-bat, working-dir, extra-args) for launching a service.
fn launch_spec(settings: &Settings, svc: Service) -> Option<(PathBuf, PathBuf, Vec<String>)> {
    match svc {
        Service::Mysql => {
            let server = PathBuf::from(settings.server_path.as_deref()?);
            let bat = server.join("MySQL.bat");
            if bat.is_file() {
                return Some((bat, server, vec![]));
            }
            let mysqld = server.join("mysql").join("bin").join("mysqld.exe");
            if mysqld.is_file() {
                return Some((
                    mysqld,
                    server,
                    vec!["--defaults-file=mysql/bin/my.cnf".into(), "--standalone".into(), "--console".into()],
                ));
            }
            None
        }
        Service::Auth | Service::World => {
            let repack = PathBuf::from(settings.repack_path.as_deref()?);
            let exe = repack.join(svc.image());
            exe.is_file().then_some((exe, repack, vec![]))
        }
    }
}

fn service_state(settings: &Settings, svc: Service) -> ServiceState {
    ServiceState {
        running: server_state::is_running(svc.image()),
        pid: server_state::pid(svc.image()),
        launchable: launch_spec(settings, svc).is_some(),
    }
}

pub fn status(settings: &Settings) -> ServerStatus {
    ServerStatus {
        mysql: service_state(settings, Service::Mysql),
        authserver: service_state(settings, Service::Auth),
        worldserver: service_state(settings, Service::World),
    }
}

pub fn start(settings: &Settings, svc: Service) -> Result<String, String> {
    if server_state::is_running(svc.image()) {
        return Ok(format!("{} is already running.", svc.display()));
    }
    let (exe, workdir, args) = launch_spec(settings, svc).ok_or_else(|| {
        format!(
            "Can't start {} — its folder isn't set, or the program is missing. Check Settings.",
            svc.display()
        )
    })?;
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    server_state::launch_in_console(&exe, &workdir, &arg_refs)?;
    std::thread::sleep(svc.startup_wait());

    if server_state::is_running(svc.image()) {
        Ok(format!("{} started.", svc.display()))
    } else {
        Err(format!(
            "{} didn't come up. Check the console window it opened for errors.",
            svc.display()
        ))
    }
}

pub fn stop(settings: &Settings, svc: Service) -> Result<String, String> {
    if !server_state::is_running(svc.image()) {
        return Ok(format!("{} isn't running.", svc.display()));
    }
    // Worldserver: try a graceful save-and-shutdown over Remote Access first, so we
    // don't lose recent character data. Fall back to a force-stop.
    if svc == Service::World && stop_world_gracefully(settings) {
        return Ok("Worldserver shut down safely.".into());
    }
    server_state::kill(svc.image())?;
    Ok(format!("Stopped {}.", svc.display()))
}

pub fn restart(settings: &Settings, svc: Service) -> Result<String, String> {
    stop(settings, svc)?;
    std::thread::sleep(Duration::from_millis(1000));
    start(settings, svc)
}

/// Start MySQL → Authserver → Worldserver in order. Returns a per-step summary.
pub fn start_all(settings: &Settings) -> Result<String, String> {
    let mut lines = Vec::new();
    for svc in [Service::Mysql, Service::Auth, Service::World] {
        match start(settings, svc) {
            Ok(msg) => lines.push(format!("✓ {msg}")),
            Err(msg) => {
                lines.push(format!("✗ {msg}"));
                return Err(lines.join("\n"));
            }
        }
    }
    Ok(lines.join("\n"))
}

/// Send `.server shutdown 1` over RA. Returns true if the worldserver actually
/// stopped within a few seconds.
fn stop_world_gracefully(settings: &Settings) -> bool {
    if ra::probe(&settings.ra_host, settings.ra_port) == ra::RaState::Unreachable {
        return false;
    }
    let cfg = ra::RaConfig {
        host: settings.ra_host.clone(),
        port: settings.ra_port,
        user: settings.ra_user.clone(),
        password: settings.ra_password.clone(),
    };
    if ra::run_command(&cfg, ".server shutdown 1").is_err() {
        return false;
    }
    // Give it time to save and exit.
    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(1000));
        if !server_state::is_running(Service::World.image()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_parse_and_image() {
        assert_eq!(Service::parse("world").unwrap().image(), "worldserver.exe");
        assert_eq!(Service::parse("authserver").unwrap().image(), "authserver.exe");
        assert_eq!(Service::parse("mysql").unwrap().image(), "mysqld.exe");
        assert!(Service::parse("nope").is_err());
    }
}
