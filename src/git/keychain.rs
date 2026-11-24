//! Secrets (access tokens, API keys) live in the macOS Keychain.
use super::*;

const SERVICE: &str = "OnayGit";

pub fn secret_set(account: &str, secret: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;
    // `security -i` reads the command from stdin, so the secret never shows up in the process list.
    let q = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    let mut child = Command::new("security")
        .arg("-i")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        let line = format!("add-generic-password -U -s {} -a {} -w {}\n", q(SERVICE), q(account), q(secret));
        stdin.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if secret_get(account).as_deref() == Some(secret) {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Err(if err.is_empty() { msg } else { err })
    }
}

pub fn secret_get(account: &str) -> Option<String> {
    let out = Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", account, "-w"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim_end_matches('\n').to_string())
}

pub fn secret_delete(account: &str) {
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE, "-a", account])
        .output();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_round_trip_with_quotes() {
        let name = "onaygit-test-account";
        let secret = "a\"b\\c d$e";
        if secret_set(name, secret).is_ok() {
            assert_eq!(secret_get(name).as_deref(), Some(secret));
        }
        secret_delete(name);
    }
}
