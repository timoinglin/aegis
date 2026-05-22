use tauri::AppHandle;

use crate::models::Settings;
use crate::services::{logging, ra};

fn ra_config(s: &Settings) -> ra::RaConfig {
    ra::RaConfig {
        host: s.ra_host.clone(),
        port: s.ra_port,
        user: s.ra_user.clone(),
        password: s.ra_password.clone(),
    }
}

/// Reject usernames that would break RA argument parsing or are obviously empty.
fn validate_username(username: &str) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Please enter an account name.".into());
    }
    if username.split_whitespace().count() != 1 {
        return Err("Account names can't contain spaces.".into());
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.trim().is_empty() {
        return Err("Please enter a password.".into());
    }
    if password.split_whitespace().count() != 1 {
        return Err("Passwords can't contain spaces.".into());
    }
    Ok(())
}

/// Run an RA command, then log + return with all secrets masked. `op_secrets` are
/// this operation's passwords; settings passwords are always masked too.
fn run_logged(
    app: &AppHandle,
    settings: &Settings,
    context: &str,
    summary: &str,
    command: &str,
    op_secrets: &[String],
) -> Result<String, String> {
    let mut secrets = settings.secrets();
    secrets.extend(op_secrets.iter().filter(|s| !s.is_empty()).cloned());

    match ra::run_command(&ra_config(settings), command) {
        Ok(response) => {
            let safe = redact(&response, &secrets);
            // The RA server reports refusals (low security, already-exists, bad syntax)
            // as ordinary replies, so we classify them rather than show a false success.
            match interpret(&safe) {
                Ok(ok_msg) => {
                    logging::log_op(app, "INFO", context, &format!("{summary} -> {ok_msg}"), &secrets);
                    Ok(ok_msg)
                }
                Err(err_msg) => {
                    logging::log_op(app, "WARN", context, &format!("{summary} -> refused: {safe}"), &secrets);
                    Err(err_msg)
                }
            }
        }
        Err(raw) => {
            logging::log_op(app, "ERROR", context, &format!("{summary} -> {raw}"), &secrets);
            // Friendly, since these surface directly to the user.
            Err(friendly_ra_error(&raw))
        }
    }
}

/// Turn the server's reply into success or a friendly failure. Server messages
/// observed live on this build drive the matching here.
fn interpret(response: &str) -> Result<String, String> {
    let l = response.to_lowercase();
    if l.contains("low security level") {
        Err("Your Remote Access account can only assign GM levels below its own. Use a higher-level RA account, or pick a lower level.".into())
    } else if l.contains("already exist") {
        Err("An account with that name already exists.".into())
    } else if l.contains("not exist") || l.contains("no such") {
        Err("There's no account with that name.".into())
    } else if l.contains("syntax") {
        Err("The server didn't accept that request. See the log for details.".into())
    } else {
        Ok(response.to_string())
    }
}

fn redact(text: &str, secrets: &[String]) -> String {
    let mut out = text.to_string();
    for s in secrets {
        if !s.is_empty() {
            out = out.replace(s, "***");
        }
    }
    out
}

fn friendly_ra_error(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.contains("connect") || l.contains("closed") || l.contains("timed out") {
        "Couldn't reach your server's Remote Access. Make sure the worldserver is running, then try again.".into()
    } else if l.contains("login") || l.contains("rejected") {
        "Remote Access refused the login. Check the RA user and password in Settings.".into()
    } else {
        "Something went wrong talking to your server. Check the log for details.".into()
    }
}

pub fn create_account(
    app: &AppHandle,
    settings: &Settings,
    username: &str,
    password: &str,
) -> Result<String, String> {
    validate_username(username)?;
    validate_password(password)?;
    run_logged(
        app,
        settings,
        "account/create",
        &format!("create '{username}'"),
        &format!(".account create {username} {password}"),
        &[password.to_string()],
    )
}

pub fn set_gm_level(
    app: &AppHandle,
    settings: &Settings,
    username: &str,
    level: u8,
    realm_id: i32,
) -> Result<String, String> {
    validate_username(username)?;
    if level > 9 {
        return Err("GM level must be between 0 and 9.".into());
    }
    run_logged(
        app,
        settings,
        "account/set_gmlevel",
        &format!("set gmlevel '{username}' -> {level} (realm {realm_id})"),
        &format!(".account set gmlevel {username} {level} {realm_id}"),
        &[],
    )
}

pub fn set_password(
    app: &AppHandle,
    settings: &Settings,
    username: &str,
    new_password: &str,
) -> Result<String, String> {
    validate_username(username)?;
    validate_password(new_password)?;
    run_logged(
        app,
        settings,
        "account/set_password",
        &format!("reset password '{username}'"),
        &format!(".account set password {username} {new_password} {new_password}"),
        &[new_password.to_string()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_rules() {
        assert!(validate_username("Bob").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("two words").is_err());
    }

    #[test]
    fn redacts_password_in_output() {
        let out = redact("created with hunter2", &["hunter2".into()]);
        assert!(!out.contains("hunter2"));
    }

    #[test]
    fn interpret_classifies_server_replies() {
        // Real replies observed live on this build.
        assert!(interpret("Account created: BOB").is_ok());
        assert!(interpret("You change security level of account BOB to 8.").is_ok());
        assert!(interpret("The password was changed").is_ok());
        assert!(interpret("You have low security level for this.").is_err());
        assert!(interpret("Account with this name already exist!").is_err());
    }
}
