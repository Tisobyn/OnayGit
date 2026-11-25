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

/// One piece of a conflicted file.
#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    Text(Vec<String>),
    Conflict { ours_label: String, theirs_label: String, ours: Vec<String>, theirs: Vec<String> },
}

/// What to keep for one conflict.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Pick {
    #[default]
    Undecided,
    Ours,
    Theirs,
    Both,
    /// Use the text typed in the editor.
    Custom,
}

/// Split a file with conflict markers into plain text and conflicts. Understands the diff3 style too.
pub fn parse_conflicts(text: &str) -> Vec<Seg> {
    let mut segs = Vec::new();
    let mut plain: Vec<String> = Vec::new();
    // 0 outside, 1 ours, 2 base, 3 theirs
    let mut state = 0u8;
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    let mut ol = String::new();
    for line in text.lines() {
        match state {
            0 if line.starts_with("<<<<<<<") => {
                if !plain.is_empty() {
                    segs.push(Seg::Text(std::mem::take(&mut plain)));
                }
                ol = line[7..].trim().to_string();
                state = 1;
            }
            0 => plain.push(line.to_string()),
            1 if line.starts_with("|||||||") => state = 2,
            1 if line.starts_with("=======") => state = 3,
            1 => ours.push(line.to_string()),
            2 if line.starts_with("=======") => state = 3,
            2 => {}
            _ if line.starts_with(">>>>>>>") => {
                let tl = line[7..].trim().to_string();
                segs.push(Seg::Conflict {
                    ours_label: std::mem::take(&mut ol),
                    theirs_label: tl,
                    ours: std::mem::take(&mut ours),
                    theirs: std::mem::take(&mut theirs),
                });
                state = 0;
            }
            _ => theirs.push(line.to_string()),
        }
    }
    if state != 0 {
        // Broken markers: keep everything as plain text so nothing is lost.
        return vec![Seg::Text(text.lines().map(String::from).collect())];
    }
    if !plain.is_empty() {
        segs.push(Seg::Text(plain));
    }
    segs
}

/// The resolved text, or None while a conflict is still undecided. `edits` holds typed text per conflict.
pub fn resolve_text(segs: &[Seg], picks: &[Pick], edits: &[String]) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut c = 0;
    for seg in segs {
        match seg {
            Seg::Text(t) => out.extend(t.iter().cloned()),
            Seg::Conflict { ours, theirs, .. } => {
                match picks.get(c).copied().unwrap_or_default() {
                    Pick::Undecided => return None,
                    Pick::Ours => out.extend(ours.iter().cloned()),
                    Pick::Theirs => out.extend(theirs.iter().cloned()),
                    Pick::Both => {
                        out.extend(ours.iter().cloned());
                        out.extend(theirs.iter().cloned());
                    }
                    Pick::Custom => out.extend(edits.get(c).map(|e| e.lines().map(String::from).collect::<Vec<_>>()).unwrap_or_default()),
                }
                c += 1;
            }
        }
    }
    let mut text = out.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    Some(text)
}

/// Write the resolved text and mark the file resolved. Local repositories only.
pub fn save_resolved(repo: &Path, path: &str, text: &str) -> Result<(), String> {
    if is_remote(repo) {
        return Err("Resolve conflicts on a remote machine with Use ours or Use theirs".into());
    }
    std::fs::write(repo.join(path), text).map_err(|e| e.to_string())?;
    mark_resolved(repo, path)
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

    #[test]
    fn parse_and_resolve_conflict_blocks() {
        let text = "top\n<<<<<<< HEAD\nmine\n=======\nyours\n>>>>>>> other\nmiddle\n<<<<<<< HEAD\na\n||||||| base\nb\n=======\nc\n>>>>>>> other\nend\n";
        let segs = parse_conflicts(text);
        assert_eq!(segs.len(), 5);
        assert!(matches!(&segs[1], Seg::Conflict { ours_label, theirs_label, .. } if ours_label == "HEAD" && theirs_label == "other"));
        assert_eq!(resolve_text(&segs, &[Pick::Ours, Pick::Undecided], &[]), None);
        let r = resolve_text(&segs, &[Pick::Both, Pick::Theirs], &[]).unwrap();
        assert_eq!(r, "top\nmine\nyours\nmiddle\nc\nend\n");
        let r = resolve_text(&segs, &[Pick::Custom, Pick::Ours], &["x\ny".into(), String::new()]).unwrap();
        assert_eq!(r, "top\nx\ny\nmiddle\na\nend\n");
        // broken markers stay plain
        assert_eq!(parse_conflicts("a\n<<<<<<< x\nb\n"), vec![Seg::Text(vec!["a".into(), "<<<<<<< x".into(), "b".into()])]);
    }
}
