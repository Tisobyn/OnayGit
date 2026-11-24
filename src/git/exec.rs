//! Running git, on this Mac or on another machine over SSH.
use super::*;
use std::io::Write;
use std::process::Stdio;

/// Build the command for `git -C repo args`. Remote repositories go through ssh.
pub fn git_command(repo: &Path, args: &[&str], env: &[(&str, &str)]) -> Command {
    if let Some((host, path)) = split_remote(repo) {
        let mut remote = String::new();
        if !env.is_empty() {
            remote.push_str("env");
            for (k, v) in env {
                remote.push(' ');
                remote.push_str(&quote(&format!("{k}={v}")));
            }
            remote.push(' ');
        }
        remote.push_str("git -C ");
        remote.push_str(&quote(&path));
        for a in args {
            remote.push(' ');
            remote.push_str(&quote(a));
        }
        return ssh_command(&host, &remote);
    }
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd
}

fn finish(out: std::process::Output) -> Result<String, String> {
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if err.is_empty() {
            Err(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            Err(err)
        }
    }
}

/// Run git and return stdout. Err holds stderr.
pub fn run(repo: &Path, args: &[&str]) -> Result<String, String> {
    run_env(repo, args, &[])
}

/// Run git with extra environment variables.
pub fn run_env(repo: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<String, String> {
    git_command(repo, args, env)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run git: {e}"))
        .and_then(finish)
}

/// Run git and return stdout even when the exit code is non zero.
pub fn run_any(repo: &Path, args: &[&str]) -> String {
    git_command(repo, args, &[])
        .stdin(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Run git with `input` on stdin.
pub fn run_stdin(repo: &Path, args: &[&str], input: &str) -> Result<String, String> {
    let mut child = git_command(repo, args, &[])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    }
    child.wait_with_output().map_err(|e| e.to_string()).and_then(finish)
}
