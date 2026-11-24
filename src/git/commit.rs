#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub when: String,
    pub subject: String,
    pub parents: Vec<String>,
}

pub fn commit(repo: &Path, summary: &str, desc: &str) -> Result<String, String> {
    if desc.trim().is_empty() {
        run(repo, &["commit", "-m", summary])
    } else {
        run(repo, &["commit", "-m", summary, "-m", desc])
    }
}

pub fn log(repo: &Path) -> Vec<Commit> {
    log_range(repo, "HEAD")
}

/// Commits in a revision range such as `HEAD..feature`.
pub fn log_range(repo: &Path, range: &str) -> Vec<Commit> {
    run(
        repo,
        &["log", "--max-count=300", "--topo-order", "--format=%H%x1f%an%x1f%ar%x1f%s%x1f%P", range],
    )
    .unwrap_or_default()
    .lines()
    .filter_map(|l| {
        let mut p = l.split('\x1f');
        Some(Commit {
            hash: p.next()?.to_string(),
            author: p.next()?.to_string(),
            when: p.next()?.to_string(),
            subject: p.next()?.to_string(),
            parents: p.next().unwrap_or("").split_whitespace().map(String::from).collect(),
        })
    })
    .collect()
}

pub fn show(repo: &Path, hash: &str) -> String {
    run(repo, &["show", "--format=%an <%ae>%n%ad%n%n%B", hash]).unwrap_or_default()
}

/// (subject, body) of HEAD.
pub fn last_commit(repo: &Path) -> Option<(String, String)> {
    let out = run(repo, &["log", "-1", "--format=%s%x1f%b"]).ok()?;
    let (s, b) = out.split_once('\x1f')?;
    Some((s.trim().to_string(), b.trim().to_string()))
}

pub fn amend(repo: &Path, summary: &str, desc: &str) -> Result<String, String> {
    if summary.is_empty() {
        return run(repo, &["commit", "--amend", "--no-edit"]);
    }
    if desc.trim().is_empty() {
        run(repo, &["commit", "--amend", "-m", summary])
    } else {
        run(repo, &["commit", "--amend", "-m", summary, "-m", desc])
    }
}

/// Undo the last commit, keeping its changes staged.
pub fn undo_commit(repo: &Path) -> Result<String, String> {
    if run(repo, &["rev-parse", "--verify", "-q", "HEAD~1"]).is_ok() {
        run(repo, &["reset", "--soft", "HEAD~1"])
    } else {
        run(repo, &["update-ref", "-d", "HEAD"])
    }
}

/// Files changed by a commit as (status letter, path).
pub fn commit_files(repo: &Path, hash: &str) -> Vec<(char, String)> {
    run(repo, &["show", "--name-status", "--no-renames", "--format=", hash])
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (st, path) = l.split_once('\t')?;
            Some((st.chars().next()?, path.to_string()))
        })
        .collect()
}

/// Diff of one file in one commit.
pub fn show_file(repo: &Path, hash: &str, path: &str) -> String {
    run(repo, &["show", "--format=", hash, "--", path]).unwrap_or_default()
}

pub fn cherry_pick(repo: &Path, hash: &str) -> Result<String, String> {
    run(repo, &["cherry-pick", hash])
}

pub fn revert(repo: &Path, hash: &str) -> Result<String, String> {
    run(repo, &["revert", "--no-edit", hash])
}

/// `mode` is soft, mixed or hard.
pub fn reset_to(repo: &Path, hash: &str, mode: &str) -> Result<String, String> {
    run(repo, &["reset", &format!("--{mode}"), hash])
}
