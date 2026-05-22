use std::process::Command;

/// True if a process with the given image name (e.g. "worldserver.exe") is running.
/// Uses Windows `tasklist` with a filter; absence of the name in output = not running.
pub fn is_running(image_name: &str) -> bool {
    let output = Command::new("tasklist")
        .arg("/FI")
        .arg(format!("IMAGENAME eq {image_name}"))
        .arg("/NH")
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
            text.contains(&image_name.to_lowercase())
        }
        Err(_) => false,
    }
}
