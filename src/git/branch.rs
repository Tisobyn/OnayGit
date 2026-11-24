#[allow(unused_imports)]
use super::*;

pub fn current_branch(repo: &Path) -> String {
    run(repo, &["symbolic-ref", "--short", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "HEAD".into())
}

pub fn branches(repo: &Path) -> Vec<String> {
    run(repo, &["branch", "--format=%(refname:short)"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn checkout(repo: &Path, branch: &str) -> Result<String, String> {
    run(repo, &["checkout", branch])
}

pub fn create_branch(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["checkout", "-b", name])
}

/// (ahead, behind) of the upstream, if there is one.
pub fn ahead_behind(repo: &Path) -> Option<(u32, u32)> {
    let out = run(repo, &["rev-list", "--left-right", "--count", "HEAD...@{u}"]).ok()?;
    let mut it = out.split_whitespace();
    Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
}

pub fn has_remote(repo: &Path) -> bool {
    run(repo, &["remote"])
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

pub fn remote_branches(repo: &Path) -> Vec<String> {
    run(repo, &["branch", "-r", "--format=%(refname:short)"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.ends_with("/HEAD") && s.contains('/'))
        .collect()
}

pub fn merge(repo: &Path, branch: &str) -> Result<String, String> {
    run(repo, &["merge", "--no-edit", branch])
}

pub fn delete_branch(repo: &Path, branch: &str) -> Result<String, String> {
    run(repo, &["branch", "-D", branch])
}

/// The upstream branch of the current branch, for example `origin/main`.
pub fn upstream(repo: &Path) -> Option<String> {
    run(repo, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// (commits only in current, commits only in `other`).
pub fn compare_counts(repo: &Path, other: &str) -> (u32, u32) {
    let spec = format!("HEAD...{other}");
    let out = run(repo, &["rev-list", "--left-right", "--count", &spec]).unwrap_or_default();
    let mut it = out.split_whitespace();
    (
        it.next().and_then(|n| n.parse().ok()).unwrap_or(0),
        it.next().and_then(|n| n.parse().ok()).unwrap_or(0),
    )
}
