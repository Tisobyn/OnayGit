//! Secrets (access tokens, API keys) live in the macOS Keychain.
use super::*;

const SERVICE: &str = "OnayGit";

pub fn secret_set(account: &str, secret: &str) -> Result<(), String> {
    let out = Command::new("security")
        .args(["add-generic-password", "-U", "-s", SERVICE, "-a", account, "-w", secret])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
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
