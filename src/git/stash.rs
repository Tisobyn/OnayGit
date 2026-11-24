//! Stash list and actions.
#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug)]
pub struct Stash {
    /// Reference such as `stash@{0}`.
    pub name: String,
    pub message: String,
}

pub fn stashes(repo: &Path) -> Vec<Stash> {
    run(repo, &["stash", "list", "--format=%gd%x1f%s"])
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (name, message) = l.split_once('\x1f')?;
            Some(Stash { name: name.to_string(), message: message.to_string() })
        })
        .collect()
}

/// Stash all changes, untracked files included.
pub fn stash_push(repo: &Path, message: &str) -> Result<String, String> {
    if message.trim().is_empty() {
        run(repo, &["stash", "push", "--include-untracked"])
    } else {
        run(repo, &["stash", "push", "--include-untracked", "-m", message.trim()])
    }
}

pub fn stash_apply(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["stash", "apply", name])
}

pub fn stash_pop(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["stash", "pop", name])
}

pub fn stash_drop(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["stash", "drop", name])
}

pub fn stash_show(repo: &Path, name: &str) -> String {
    run(repo, &["stash", "show", "-p", "--include-untracked", name])
        .or_else(|_| run(repo, &["stash", "show", "-p", name]))
        .unwrap_or_default()
}
