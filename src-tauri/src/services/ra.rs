use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Remote Access reachability for the Health check. The full 3-state model is:
///   Unreachable   -> port closed / server down
///   Reachable     -> port open (TCP connect succeeds)
///   Authenticated -> a real login + command round-trip succeeds (used by account ops)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaState {
    Unreachable,
    Reachable,
    #[allow(dead_code)]
    Authenticated,
}

/// TCP-connect probe with a short timeout (Health check only — does not log in).
pub fn probe(host: &str, port: u16) -> RaState {
    match resolve(host, port) {
        Some(addr) => match TcpStream::connect_timeout(&addr, Duration::from_millis(1500)) {
            Ok(_) => RaState::Reachable,
            Err(_) => RaState::Unreachable,
        },
        None => RaState::Unreachable,
    }
}

pub struct RaConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

/// Run one command over a fresh, authenticated RA session and return the cleaned
/// server response. This mirrors wow-server-mcp's proven flow:
/// connect -> Username: -> Password: -> wait for prompt -> send command -> read to prompt.
///
/// The worldserver does the actual work (e.g. SRP6 hashing for `.account create`),
/// which is exactly why we go through RA instead of writing the DB directly.
pub fn run_command(cfg: &RaConfig, command: &str) -> Result<String, String> {
    let mut out = run_session(cfg, std::slice::from_ref(&command.to_string()))?;
    Ok(out.pop().unwrap_or_default())
}

/// Run several commands over ONE authenticated connection (e.g. dumping many
/// characters), returning each command's cleaned response in order.
pub fn run_session(cfg: &RaConfig, commands: &[String]) -> Result<Vec<String>, String> {
    let addr = resolve(&cfg.host, cfg.port).ok_or("Could not resolve the Remote Access address")?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| format!("connect failed: {e}"))?;
    // Short read timeout so the loop can re-check the per-step deadline.
    stream
        .set_read_timeout(Some(Duration::from_millis(400)))
        .map_err(|e| e.to_string())?;

    let step = || Instant::now() + Duration::from_secs(15);

    read_until(&mut stream, step(), |b| lc_contains(b, "username"))?;
    write_line(&mut stream, &cfg.user)?;
    read_until(&mut stream, step(), |b| lc_contains(b, "password"))?;
    write_line(&mut stream, &cfg.password)?;

    let auth = read_until(&mut stream, step(), |b| is_prompt(b) || auth_failed(b))?;
    if auth_failed(&auth) {
        return Err("Remote Access login was rejected (check the RA user/password)".into());
    }

    let mut results = Vec::with_capacity(commands.len());
    for command in commands {
        write_line(&mut stream, command)?;
        let out = read_until(&mut stream, step(), is_prompt)?;
        results.push(clean_response(&out));
    }
    let _ = write_line(&mut stream, "quit"); // best-effort clean close
    Ok(results)
}

// --- helpers ---------------------------------------------------------------

fn resolve(host: &str, port: u16) -> Option<std::net::SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

fn write_line(stream: &mut TcpStream, line: &str) -> Result<(), String> {
    stream
        .write_all(format!("{line}\r\n").as_bytes())
        .map_err(|e| e.to_string())
}

fn lc_contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

/// A command prompt is the trimmed buffer ending in `>` (e.g. `TC>`, `SF>`, `MoP>`).
fn is_prompt(s: &str) -> bool {
    s.trim_end().ends_with('>')
}

fn auth_failed(s: &str) -> bool {
    let l = s.to_lowercase();
    l.contains("wrong") || l.contains("denied") || l.contains("failed") || l.contains("invalid")
}

/// Strip the core's prompt markers and trim. (The RA server doesn't echo input,
/// so the command text won't be present.)
fn clean_response(raw: &str) -> String {
    raw.replace("TC>", "")
        .replace("SF>", "")
        .replace("MoP>", "")
        .replace("mop>", "")
        .trim()
        .to_string()
}

/// Read until `pred(buffer)` is true or the deadline passes.
fn read_until<F>(stream: &mut TcpStream, deadline: Instant, pred: F) -> Result<String, String>
where
    F: Fn(&str) -> bool,
{
    let mut buf = String::new();
    let mut tmp = [0u8; 2048];
    loop {
        if Instant::now() > deadline {
            return Err("Remote Access timed out waiting for the server".into());
        }
        match stream.read(&mut tmp) {
            Ok(0) => return Err("Remote Access connection closed unexpectedly".into()),
            Ok(n) => {
                buf.push_str(&String::from_utf8_lossy(&tmp[..n]));
                if pred(&buf) {
                    return Ok(buf);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue; // re-check deadline, keep waiting
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_detection() {
        assert!(is_prompt("Account created.\r\nTC> "));
        assert!(is_prompt("foo MoP>"));
        assert!(!is_prompt("Username:"));
        assert!(!is_prompt("No online players."));
    }

    #[test]
    fn auth_failure_keywords() {
        assert!(auth_failed("Authentication failed"));
        assert!(auth_failed("Wrong password"));
        assert!(!auth_failed("Account created."));
    }

    #[test]
    fn clean_strips_prompt() {
        assert_eq!(clean_response("Account created.\r\nTC> "), "Account created.");
    }
}
