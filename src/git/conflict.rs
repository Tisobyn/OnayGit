//! Merge, rebase, cherry-pick and revert in progress, and conflict resolution.
use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RepoOp {
    Merge,
    Rebase,
    CherryPick,
    Revert,
}

impl RepoOp {
    pub fn label(self) -> &'static str {
        match self {
            RepoOp::Merge => "Merge",
            RepoOp::Rebase => "Rebase",
            RepoOp::CherryPick => "Cherry-pick",
            RepoOp::Revert => "Revert",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Ours,
    Theirs,
}

/// The operation that is currently stopped in this repository, if any.
pub fn repo_op(repo: &Path) -> Option<RepoOp> {
    let dir = run(repo, &["rev-parse", "--absolute-git-dir"]).ok()?;
    let dir = std::path::PathBuf::from(dir.trim());
    if dir.join("rebase-merge").exists() || dir.join("rebase-apply").exists() {
        Some(RepoOp::Rebase)
    } else if dir.join("MERGE_HEAD").exists() {
        Some(RepoOp::Merge)
    } else if dir.join("CHERRY_PICK_HEAD").exists() {
        Some(RepoOp::CherryPick)
    } else if dir.join("REVERT_HEAD").exists() {
        Some(RepoOp::Revert)
    } else {
        None
    }
}

const NO_EDITOR: &[(&str, &str)] = &[("GIT_EDITOR", "true")];

pub fn op_continue(repo: &Path, op: RepoOp) -> Result<String, String> {
    match op {
        RepoOp::Merge => run_env(repo, &["commit", "--no-edit"], NO_EDITOR),
        RepoOp::Rebase => run_env(repo, &["rebase", "--continue"], NO_EDITOR),
        RepoOp::CherryPick => run_env(repo, &["cherry-pick", "--continue"], NO_EDITOR),
        RepoOp::Revert => run_env(repo, &["revert", "--continue"], NO_EDITOR),
    }
}

pub fn op_abort(repo: &Path, op: RepoOp) -> Result<String, String> {
    match op {
        RepoOp::Merge => run(repo, &["merge", "--abort"]),
        RepoOp::Rebase => run(repo, &["rebase", "--abort"]),
        RepoOp::CherryPick => run(repo, &["cherry-pick", "--abort"]),
        RepoOp::Revert => run(repo, &["revert", "--abort"]),
    }
}

/// Take one side of a conflicted file and mark it resolved.
pub fn resolve_side(repo: &Path, path: &str, side: Side) -> Result<(), String> {
    let flag = if side == Side::Ours { "--ours" } else { "--theirs" };
    run(repo, &["checkout", flag, "--", path])?;
    run(repo, &["add", "--", path]).map(|_| ())
}

pub fn mark_resolved(repo: &Path, path: &str) -> Result<(), String> {
    run(repo, &["add", "--", path]).map(|_| ())
}

/// The conflicted file as lines: marker lines are hunks, "ours" is red, "theirs" is green.
pub fn conflict_lines(repo: &Path, path: &str) -> Vec<DiffLine> {
    let text = read_file(repo, path);
    let (mut region, mut out) = (0u8, Vec::new());
    for (i, line) in text.lines().enumerate() {
        let n = Some(i as u32 + 1);
        let (kind, old, new) = if line.starts_with("<<<<<<<") {
            region = 1;
            (Kind::Hunk, None, None)
        } else if line.starts_with("=======") && region == 1 {
            region = 2;
            (Kind::Hunk, None, None)
        } else if line.starts_with(">>>>>>>") && region == 2 {
            region = 0;
            (Kind::Hunk, None, None)
        } else {
            match region {
                1 => (Kind::Del, n, None),
                2 => (Kind::Add, None, n),
                _ => (Kind::Ctx, n, n),
            }
        };
        out.push(DiffLine { kind, old, new, text: line.to_string() });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(d: &Path, a: &[&str]) -> String {
        run(d, a).unwrap_or_else(|e| panic!("git {a:?}: {e}"))
    }

    fn repo(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("onaygit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        sh(&d, &["init", "-q", "-b", "main"]);
        sh(&d, &["config", "user.email", "t@t"]);
        sh(&d, &["config", "user.name", "t"]);
        d
    }

    fn commit_file(d: &Path, name: &str, body: &str, msg: &str) {
        std::fs::write(d.join(name), body).unwrap();
        sh(d, &["add", "."]);
        sh(d, &["commit", "-qm", msg]);
    }

    fn subjects(d: &Path) -> Vec<String> {
        log(d).into_iter().map(|c| c.subject).collect()
    }

    #[test]
    fn rewrite_drop_squash_reword() {
        let d = repo("rewrite");
        commit_file(&d, "a", "1", "one");
        commit_file(&d, "b", "2", "two");
        commit_file(&d, "c", "3", "three");
        commit_file(&d, "d", "4", "four");

        let log = log(&d);
        // newest first: four, three, two, one
        rewrite(&d, RewriteOp::Reword, &log[2].hash, "TWO renamed").unwrap();
        assert_eq!(subjects(&d), ["four", "three", "TWO renamed", "one"]);

        let log = log_range(&d, "HEAD");
        rewrite(&d, RewriteOp::Drop, &log[1].hash, "").unwrap();
        assert_eq!(subjects(&d), ["four", "TWO renamed", "one"]);
        assert!(!d.join("c").exists());

        let log = log_range(&d, "HEAD");
        rewrite(&d, RewriteOp::Squash, &log[0].hash, "").unwrap();
        assert_eq!(subjects(&d).len(), 2);
        assert!(d.join("d").exists() && d.join("b").exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn graph_lanes_for_a_merge() {
        let d = repo("graph");
        commit_file(&d, "a", "1", "base");
        sh(&d, &["checkout", "-qb", "side"]);
        commit_file(&d, "s", "s", "side work");
        sh(&d, &["checkout", "-q", "main"]);
        commit_file(&d, "m", "m", "main work");
        sh(&d, &["merge", "--no-ff", "--no-edit", "-q", "side"]);
        let commits = log(&d);
        let (rows, width) = graph(&commits);
        assert_eq!(rows.len(), 4);
        assert!(width >= 2);
        assert_eq!(rows[0].branch_to.len(), 1, "merge commit opens a second lane");
        assert!(!rows[3].bottom.iter().any(|b| *b), "root commit ends all lanes");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn conflict_flow_cherry_pick_revert_reset() {
        let d = repo("conflict");
        commit_file(&d, "f", "base\n", "base");
        sh(&d, &["checkout", "-qb", "other"]);
        commit_file(&d, "f", "theirs\n", "other change");
        let other = log(&d)[0].hash.clone();
        sh(&d, &["checkout", "-q", "main"]);
        commit_file(&d, "f", "ours\n", "main change");

        assert!(merge(&d, "other").is_err());
        assert_eq!(repo_op(&d), Some(RepoOp::Merge));
        let files = status(&d);
        assert!(files.iter().any(|f| f.conflicted()));
        let lines = conflict_lines(&d, "f");
        assert!(lines.iter().any(|l| l.kind == Kind::Del && l.text == "ours"));
        assert!(lines.iter().any(|l| l.kind == Kind::Add && l.text == "theirs"));
        resolve_side(&d, "f", Side::Theirs).unwrap();
        op_continue(&d, RepoOp::Merge).unwrap();
        assert_eq!(repo_op(&d), None);
        assert_eq!(std::fs::read_to_string(d.join("f")).unwrap(), "theirs\n");

        // cherry-pick conflict then abort
        sh(&d, &["reset", "-q", "--hard", "HEAD~1"]);
        assert!(cherry_pick(&d, &other).is_err());
        assert_eq!(repo_op(&d), Some(RepoOp::CherryPick));
        op_abort(&d, RepoOp::CherryPick).unwrap();
        assert_eq!(repo_op(&d), None);

        // revert and reset
        commit_file(&d, "g", "x", "add g");
        revert(&d, &log(&d)[0].hash).unwrap();
        assert!(!d.join("g").exists());
        let target = log(&d)[2].hash.clone();
        reset_to(&d, &target, "hard").unwrap();
        assert_eq!(subjects(&d)[0], "main change");

        let files = commit_files(&d, &log(&d)[0].hash);
        assert_eq!(files, vec![('M', "f".to_string())]);
        let _ = std::fs::remove_dir_all(&d);
    }
}
