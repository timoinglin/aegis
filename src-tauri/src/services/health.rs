use chrono::Utc;
use tauri::AppHandle;

use crate::models::{HealthCheck, HealthReport, HealthStatus, Settings};
use crate::services::{backup, logging, mysql, ra, server_state, settings_store};

const CAT_CONNECTIVITY: &str = "Connectivity";
const CAT_BACKUPS: &str = "Backups";

/// Build the full report. Each active check probes real state; failures map to
/// friendly what/why/fix text here, while the raw cause is logged (redacted).
pub fn report(app: &AppHandle) -> HealthReport {
    let settings = settings_store::load(app);

    let mut checks = vec![
        check_mysql_bins(&settings),
        check_db(app, &settings),
        check_ra(&settings),
        check_worldserver(),
        check_backup_folder(app, &settings),
        check_backups(app, &settings),
        check_schedule(&settings),
    ];
    checks.extend(reserved_slots());

    let overall = worst(&checks);
    HealthReport {
        checks,
        overall,
        checked_at: Utc::now().timestamp_millis(),
    }
}

/// Re-run, then return the single requested check alongside a fresh full report
/// (the frontend swaps in the whole report; per-row recheck is just a refresh).
pub fn recheck(app: &AppHandle, _id: &str) -> HealthReport {
    report(app)
}

fn check_mysql_bins(settings: &Settings) -> HealthCheck {
    let found = settings
        .server_path
        .as_deref()
        .map(|p| mysql::has_bins(std::path::Path::new(p)))
        .unwrap_or(false);

    if found {
        ok("mysql_bins", CAT_CONNECTIVITY, "Repack tools found")
    } else {
        HealthCheck {
            id: "mysql_bins".into(),
            category: CAT_CONNECTIVITY.into(),
            status: HealthStatus::Error,
            title: "Can't find your repack's MySQL tools".into(),
            why: "Aegis uses the tools that came with your repack to back up and manage the database. Without them, nothing else will work.".into(),
            fix: vec![
                "Open Settings.".into(),
                "Click Auto-detect, or point \"_Server folder\" at your repack's Database\\_Server folder.".into(),
                "Save & re-check.".into(),
            ],
            active: true,
        }
    }
}

fn check_db(app: &AppHandle, settings: &Settings) -> HealthCheck {
    match mysql::ping(settings) {
        Ok(()) => ok("db", CAT_CONNECTIVITY, "Database connected"),
        Err(raw) => {
            logging::log_op(app, "WARN", "health/db", &raw, &settings.secrets());
            HealthCheck {
                id: "db".into(),
                category: CAT_CONNECTIVITY.into(),
                status: HealthStatus::Error,
                title: "Can't reach your database".into(),
                why: "Account, backup and restore tasks all need a working database connection.".into(),
                fix: vec![
                    "Make sure MySQL is running (the repack's MySQL.bat).".into(),
                    "Check the host, port, user and password in Settings.".into(),
                    "Save & re-check.".into(),
                ],
                active: true,
            }
        }
    }
}

fn check_ra(settings: &Settings) -> HealthCheck {
    match ra::probe(&settings.ra_host, settings.ra_port) {
        ra::RaState::Unreachable => HealthCheck {
            id: "ra".into(),
            category: CAT_CONNECTIVITY.into(),
            status: HealthStatus::Warn,
            title: "Remote Access isn't reachable".into(),
            why: "Account tasks (create account, set GM level, reset password) are done through your server's Remote Access. They'll be unavailable until it's reachable.".into(),
            fix: vec![
                "Start your worldserver if it isn't running.".into(),
                "Confirm Remote Access is enabled in worldserver.conf (Ra.Enable = 1).".into(),
                "Check the Remote Access host and port in Settings.".into(),
            ],
            active: true,
        },
        _ => ok("ra", CAT_CONNECTIVITY, "Remote Access reachable"),
    }
}

fn check_worldserver() -> HealthCheck {
    let running = server_state::is_running("worldserver.exe");
    // Neutral signal: Accounts reads this as "must be up", Restore as "must be down".
    HealthCheck {
        id: "worldserver".into(),
        category: CAT_CONNECTIVITY.into(),
        status: HealthStatus::Ok,
        title: if running {
            "Worldserver is running".into()
        } else {
            "Worldserver is stopped".into()
        },
        why: "Some tasks need it running (accounts); others need it stopped (restoring a backup). Aegis will guide you per task.".into(),
        fix: vec![],
        active: true,
    }
}

fn check_backup_folder(app: &AppHandle, settings: &Settings) -> HealthCheck {
    match backup::resolve_dir(app, settings) {
        // resolve_dir creates the folder, so success means it's writable.
        Ok(dir) => HealthCheck {
            id: "backup_folder_writable".into(),
            category: CAT_BACKUPS.into(),
            status: HealthStatus::Ok,
            title: "Backup folder ready".into(),
            why: format!("Backups are saved to {}.", dir.display()),
            fix: vec![],
            active: true,
        },
        Err(_) => HealthCheck {
            id: "backup_folder_writable".into(),
            category: CAT_BACKUPS.into(),
            status: HealthStatus::Error,
            title: "Can't write to your backup folder".into(),
            why: "Aegis can't save backups until it has a folder it can write to.".into(),
            fix: vec![
                "Open Settings and pick a backup folder on a drive with free space.".into(),
                "Make sure that folder isn't read-only.".into(),
                "Save & re-check.".into(),
            ],
            active: true,
        },
    }
}

fn check_backups(app: &AppHandle, settings: &Settings) -> HealthCheck {
    let newest = backup::resolve_dir(app, settings)
        .ok()
        .and_then(|dir| backup::newest_backup(&dir));

    match newest {
        None => HealthCheck {
            id: "backups".into(),
            category: CAT_BACKUPS.into(),
            status: HealthStatus::Warn,
            title: "No backups yet".into(),
            why: "A backup is your safety net if something goes wrong. It's worth taking one now."
                .into(),
            fix: vec!["Open Backup and click \"Back up now\".".into()],
            active: true,
        },
        Some((_, modified, _)) => {
            let secs = modified.elapsed().map(|d| d.as_secs()).unwrap_or(0);
            let days = secs / 86_400;
            let stale = days >= 14;
            HealthCheck {
                id: "backups".into(),
                category: CAT_BACKUPS.into(),
                status: if stale { HealthStatus::Warn } else { HealthStatus::Ok },
                title: format!("Last backup: {}", humanize_age(secs)),
                why: if stale {
                    "It's been a while. Consider taking a fresh backup.".into()
                } else {
                    String::new()
                },
                fix: if stale {
                    vec!["Open Backup and click \"Back up now\".".into()]
                } else {
                    vec![]
                },
                active: true,
            }
        }
    }
}

fn humanize_age(secs: u64) -> String {
    if secs < 3_600 {
        "less than an hour ago".into()
    } else if secs < 86_400 {
        let h = secs / 3_600;
        format!("{h} hour{} ago", if h == 1 { "" } else { "s" })
    } else {
        let d = secs / 86_400;
        format!("{d} day{} ago", if d == 1 { "" } else { "s" })
    }
}

fn check_schedule(settings: &Settings) -> HealthCheck {
    let status = crate::services::scheduler::status(settings);
    HealthCheck {
        id: "backup_schedule".into(),
        category: CAT_BACKUPS.into(),
        status: HealthStatus::Ok, // informational — never nags
        title: if status.enabled {
            match status.next_run {
                Some(next) => format!("Automatic backups: {} (next: {next})", status.summary),
                None => format!("Automatic backups: {}", status.summary),
            }
        } else {
            "Automatic backups are off".into()
        },
        why: if status.enabled {
            String::new()
        } else {
            "Turn them on in Backup so you always have a recent copy.".into()
        },
        fix: vec![],
        active: true,
    }
}

/// Still-dormant slots — wired now, populated as their features land.
fn reserved_slots() -> Vec<HealthCheck> {
    let slot = |id: &str, title: &str| HealthCheck {
        id: id.into(),
        category: CAT_BACKUPS.into(),
        status: HealthStatus::Unknown,
        title: title.into(),
        why: String::new(),
        fix: vec![],
        active: false,
    };
    vec![slot("server_stopped_for_restore", "Safe-to-restore check (coming in a later update)")]
}

fn ok(id: &str, category: &str, title: &str) -> HealthCheck {
    HealthCheck {
        id: id.into(),
        category: category.into(),
        status: HealthStatus::Ok,
        title: title.into(),
        why: String::new(),
        fix: vec![],
        active: true,
    }
}

/// Worst status across active checks (dormant slots don't count).
fn worst(checks: &[HealthCheck]) -> HealthStatus {
    let mut overall = HealthStatus::Ok;
    for c in checks.iter().filter(|c| c.active) {
        overall = match (overall, c.status) {
            (HealthStatus::Error, _) | (_, HealthStatus::Error) => HealthStatus::Error,
            (HealthStatus::Warn, _) | (_, HealthStatus::Warn) => HealthStatus::Warn,
            _ => overall,
        };
    }
    overall
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chk(status: HealthStatus, active: bool) -> HealthCheck {
        HealthCheck {
            id: "x".into(),
            category: "c".into(),
            status,
            title: String::new(),
            why: String::new(),
            fix: vec![],
            active,
        }
    }

    #[test]
    fn error_beats_warn_beats_ok() {
        let checks = vec![
            chk(HealthStatus::Ok, true),
            chk(HealthStatus::Warn, true),
            chk(HealthStatus::Error, true),
        ];
        assert_eq!(worst(&checks), HealthStatus::Error);
    }

    #[test]
    fn dormant_slots_do_not_affect_overall() {
        // An (unknown) reserved slot must not drag the banner away from "ok".
        let checks = vec![
            chk(HealthStatus::Ok, true),
            chk(HealthStatus::Unknown, false),
        ];
        assert_eq!(worst(&checks), HealthStatus::Ok);
    }

    #[test]
    fn reserved_slots_are_inactive_and_unknown() {
        for slot in reserved_slots() {
            assert!(!slot.active);
            assert_eq!(slot.status, HealthStatus::Unknown);
        }
    }
}
