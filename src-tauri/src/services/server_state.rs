use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// True if a process with the given image name (e.g. "worldserver.exe") is running.
pub fn is_running(image_name: &str) -> bool {
    pid(image_name).is_some() || running_no_pid(image_name)
}

/// PID of the first matching process, if any (parsed from tasklist CSV).
pub fn pid(image_name: &str) -> Option<u32> {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image_name}"), "/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.to_lowercase().contains(&image_name.to_lowercase()) {
            // CSV: "name","pid",...
            if let Some(p) = line.split(',').nth(1) {
                return p.trim().trim_matches('"').parse().ok();
            }
        }
    }
    None
}

/// Fallback running check that doesn't require PID parsing.
fn running_no_pid(image_name: &str) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image_name}"), "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&image_name.to_lowercase()))
        .unwrap_or(false)
}

/// Force-stop a process by image name. Returns Ok once it's confirmed gone.
pub fn kill(image_name: &str) -> Result<(), String> {
    if !is_running(image_name) {
        return Ok(());
    }
    let _ = Command::new("taskkill")
        .args(["/IM", image_name, "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    // taskkill can report non-zero even on success — verify instead.
    std::thread::sleep(std::time::Duration::from_millis(1500));
    if is_running(image_name) {
        Err(format!("{image_name} is still running after the stop attempt."))
    } else {
        Ok(())
    }
}

/// Launch an executable in its own new console window (so the user sees its logs),
/// mirroring the proven `cmd /c start "" /D <cwd> <exe> [args]` pattern.
pub fn launch_in_console(exe: &Path, working_dir: &Path, extra_args: &[&str]) -> Result<(), String> {
    if !exe.is_file() {
        return Err(format!("Couldn't find {}.", exe.display()));
    }
    let mut args: Vec<String> = vec![
        "/c".into(),
        "start".into(),
        String::new(), // empty window title
        "/D".into(),
        working_dir.to_string_lossy().into_owned(),
        exe.to_string_lossy().into_owned(),
    ];
    args.extend(extra_args.iter().map(|s| s.to_string()));

    Command::new("cmd.exe")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Couldn't launch {}: {e}", exe.display()))
}
