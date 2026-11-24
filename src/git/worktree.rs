//! Worktrees.
#[allow(unused_imports)]
use super::*;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Worktree {
    pub path: PathBuf,
    /// Branch name, or empty when detached.
    pub branch: String,
    pub head: String,
    pub main: bool,
}

pub fn worktrees(repo: &Path) -> Vec<Worktree> {
    let out = run(repo, &["worktree", "list", "--porcelain"]).unwrap_or_default();
    let mut list: Vec<Worktree> = Vec::new();
    for block in out.split("\n\n") {
        let mut wt = Worktree { path: PathBuf::new(), branch: String::new(), head: String::new(), main: false };
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                wt.path = PathBuf::from(p);
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                wt.head = h.chars().take(7).collect();
            } else if let Some(b) = line.strip_prefix("branch ") {
                wt.branch = b.trim_start_matches("refs/heads/").to_string();
            }
        }
        if !wt.path.as_os_str().is_empty() {
            wt.main = list.is_empty();
            list.push(wt);
        }
    }
    list
}

/// Add a worktree at `path`. With `new_branch` the branch is created.
pub fn worktree_add(repo: &Path, path: &Path, branch: &str, new_branch: bool) -> Result<String, String> {
    let p = path.to_string_lossy();
    if new_branch {
        run(repo, &["worktree", "add", "-b", branch, &p])
    } else {
        run(repo, &["worktree", "add", &p, branch])
    }
}

pub fn worktree_remove(repo: &Path, path: &Path, force: bool) -> Result<String, String> {
    let p = path.to_string_lossy();
    if force {
        run(repo, &["worktree", "remove", "--force", &p])
    } else {
        run(repo, &["worktree", "remove", &p])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_list_remove() {
        let base = std::env::temp_dir().join(format!("onaygit-wt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let repo = base.join("main");
        std::fs::create_dir_all(&repo).unwrap();
        run(&repo, &["init", "-q", "-b", "main"]).unwrap();
        run(&repo, &["config", "user.email", "t@t"]).unwrap();
        run(&repo, &["config", "user.name", "t"]).unwrap();
        std::fs::write(repo.join("a"), "1").unwrap();
        run(&repo, &["add", "."]).unwrap();
        run(&repo, &["commit", "-qm", "init"]).unwrap();

        let wt_path = base.join("feature");
        worktree_add(&repo, &wt_path, "feature", true).unwrap();
        let list = worktrees(&repo);
        assert_eq!(list.len(), 2);
        assert!(list[0].main && list[0].branch == "main");
        assert_eq!(list[1].branch, "feature");

        // stash and tags in the same repo
        std::fs::write(repo.join("a"), "2").unwrap();
        std::fs::write(repo.join("new.txt"), "x").unwrap();
        stash_push(&repo, "my stash").unwrap();
        let st = stashes(&repo);
        assert_eq!(st.len(), 1);
        assert!(st[0].message.contains("my stash"));
        assert!(stash_show(&repo, &st[0].name).contains("+2"));
        assert!(status(&repo).is_empty());
        stash_pop(&repo, &st[0].name).unwrap();
        assert_eq!(status(&repo).len(), 2);

        create_tag(&repo, "v1", "", "release").unwrap();
        create_tag(&repo, "light", "", "").unwrap();
        let t = tags(&repo);
        assert_eq!(t.len(), 2);
        delete_tag(&repo, "light").unwrap();
        assert_eq!(tags(&repo).len(), 1);

        worktree_remove(&repo, &wt_path, false).unwrap();
        assert_eq!(worktrees(&repo).len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }
}
