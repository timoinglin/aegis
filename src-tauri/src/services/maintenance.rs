use tauri::AppHandle;

use crate::models::{DbSize, MaintenanceResult, Settings};
use crate::services::{backup, logging, mysql};

/// On-disk size (data + indexes) and table count for each game database, queried
/// from information_schema. Used by the Maintenance page summary card.
pub fn db_sizes(settings: &Settings) -> Vec<DbSize> {
    let dbs = backup::databases_to_backup(settings);
    let mut out = Vec::with_capacity(dbs.len());
    for db in dbs {
        // data_length + index_length is the standard "size on disk" approximation;
        // file_per_table makes this very close to actual .ibd footprint on InnoDB.
        let sql = format!(
            "SELECT IFNULL(SUM(data_length + index_length), 0), COUNT(*) \
             FROM information_schema.tables WHERE table_schema='{db}'"
        );
        let (size_bytes, table_count) = mysql::query(settings, &sql)
            .ok()
            .and_then(|s| {
                let mut p = s.split('\t');
                Some((
                    p.next()?.trim().parse::<u64>().ok()?,
                    p.next()?.trim().parse::<u32>().ok()?,
                ))
            })
            .unwrap_or((0, 0));
        out.push(DbSize { name: db, size_bytes, table_count });
    }
    out
}

/// Run a maintenance pass over the live game databases using the bundled
/// mysqlcheck. mode: "analyze" | "optimize" | "repair".
///
/// Honest about InnoDB: a plain "repair" is a no-op there, so the repair button
/// uses --auto-repair (check, and repair anything that supports it).
pub fn run(app: &AppHandle, settings: &Settings, mode: &str) -> Result<MaintenanceResult, String> {
    let (flag, verb) = match mode {
        "analyze" => ("--analyze", "Analyze"),
        "optimize" => ("--optimize", "Optimize"),
        "repair" => ("--auto-repair", "Check"),
        _ => return Err("Unknown maintenance action.".into()),
    };

    let dbs = backup::databases_to_backup(settings);
    if dbs.is_empty() {
        return Err("No databases found — check the connection in Settings.".into());
    }

    let output = mysql::base_cmd(settings, "mysqlcheck")?
        .arg(flag)
        .arg("--databases")
        .args(&dbs)
        .output()
        .map_err(|e| format!("Couldn't start mysqlcheck: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let secrets = settings.secrets();

    if !output.status.success() && stdout.is_empty() {
        logging::log_op(app, "ERROR", "maintenance", &format!("{mode} failed: {stderr}"), &secrets);
        return Err(format!("{verb} couldn't run. See the log for details."));
    }

    // Lines containing error/corrupt are real problems; everything else is fine.
    let issues: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            let lc = l.to_lowercase();
            lc.contains("error") || lc.contains("corrupt")
        })
        .collect();
    let healthy = issues.is_empty();

    let message = if healthy {
        format!("{verb} complete — everything looks healthy across {} databases.", dbs.len())
    } else {
        format!("{verb} complete, but found {} possible issue(s). Check the details below.", issues.len())
    };

    logging::log_op(app, "INFO", "maintenance", &format!("{mode}: healthy={healthy}"), &secrets);
    Ok(MaintenanceResult {
        mode: mode.to_string(),
        message,
        healthy,
        details: stdout,
    })
}
