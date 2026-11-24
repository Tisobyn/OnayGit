//! Repositories on other machines. A remote repository path looks like
//! `ssh://user@host/abs/path`. Everything runs through the `ssh` command with key login.
use super::*;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

pub const PREFIX: &str = "ssh://";

pub fn is_remote(p: &Path) -> bool {
    p.to_string_lossy().starts_with(PREFIX)
}

/// Split `ssh://host/abs/path` into (host, "/abs/path").
pub fn split_remote(p: &Path) -> Option<(String, String)> {
    let s = p.to_string_lossy();
    let rest = s.strip_prefix(PREFIX)?;
    let i = rest.find('/')?;
    Some((rest[..i].to_string(), rest[i..].to_string()))
}

pub fn remote_path(host: &str, path: &str) -> PathBuf {
    PathBuf::from(format!("{PREFIX}{host}{path}"))
}

/// Shell-quote a word with single quotes.
pub fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn ssh_command(host: &str, remote_cmd: &str) -> Command {
    let mut cmd = Command::new("ssh");
    cmd.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        "-o",
        "ControlMaster=auto",
        "-o",
        "ControlPath=~/.ssh/onaygit-%C",
        "-o",
        "ControlPersist=120",
        host,
        remote_cmd,
    ]);
    cmd
}

/// Check that key login to the machine works.
pub fn test_machine(host: &str) -> Result<(), String> {
    let out = ssh_command(host, "echo ok").stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Find repositories (absolute paths) below the home folder of a machine.
pub fn scan_machine(host: &str) -> Result<Vec<String>, String> {
    let cmd = "find \"$HOME\" -maxdepth 4 -name .git -type d -not -path '*/node_modules/*' 2>/dev/null";
    let out = ssh_command(host, cmd).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    if !out.status.success() && out.stdout.is_empty() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let mut list: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.strip_suffix("/.git").map(String::from))
        .collect();
    list.sort();
    Ok(list)
}

/// Read a file of a repository (local or remote). Empty when missing.
pub fn read_file(repo: &Path, rel: &str) -> String {
    if let Some((host, path)) = split_remote(repo) {
        let cmd = format!("cat -- {}", quote(&format!("{path}/{rel}")));
        return ssh_command(&host, &cmd)
            .stdin(Stdio::null())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
    }
    std::fs::read_to_string(repo.join(rel)).unwrap_or_default()
}

pub fn write_file(repo: &Path, rel: &str, text: &str) -> Result<(), String> {
    if let Some((host, path)) = split_remote(repo) {
        let cmd = format!("cat > {}", quote(&format!("{path}/{rel}")));
        let mut child = ssh_command(&host, &cmd)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        return if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        };
    }
    std::fs::write(repo.join(rel), text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_paths() {
        let p = remote_path("me@box", "/home/me/code/app");
        assert!(is_remote(&p));
        assert_eq!(split_remote(&p), Some(("me@box".into(), "/home/me/code/app".into())));
        assert!(!is_remote(Path::new("/Users/x/app")));
        assert_eq!(p.file_name().unwrap().to_string_lossy(), "app");
    }

    #[test]
    fn quoting_is_safe() {
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), "'it'\\''s'");
        let c = git_command(&remote_path("h", "/r p"), &["log", "-m", "x'y"], &[("GIT_EDITOR", "true")]);
        let args: Vec<String> = c.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        let last = args.last().unwrap();
        assert_eq!(last, "env 'GIT_EDITOR=true' git -C '/r p' 'log' '-m' 'x'\\''y'");
    }

    #[test]
    fn local_file_roundtrip() {
        let d = std::env::temp_dir().join(format!("onaygit-ssh-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        write_file(&d, "a.txt", "hi").unwrap();
        assert_eq!(read_file(&d, "a.txt"), "hi");
        let _ = std::fs::remove_dir_all(&d);
    }
}
