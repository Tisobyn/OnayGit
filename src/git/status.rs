#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: String,
    pub index: char,
    pub work: char,
}

impl FileChange {
    pub fn staged(&self) -> bool {
        self.index != ' ' && self.index != '?'
    }

    /// Unmerged path (merge, rebase, cherry-pick or revert conflict).
    pub fn conflicted(&self) -> bool {
        matches!((self.index, self.work), ('D', 'D') | ('A', 'A')) || self.index == 'U' || self.work == 'U'
    }

    /// One letter badge like GitHub Desktop.
    pub fn badge(&self) -> char {
        if self.conflicted() {
            return '!';
        }
        if self.work == '?' {
            return 'A';
        }
        let c = if self.index != ' ' { self.index } else { self.work };
        match c {
            'M' | 'T' => 'M',
            'A' | 'C' => 'A',
            'D' => 'D',
            'R' => 'R',
            'U' => '!',
            _ => 'M',
        }
    }
}

pub fn status(repo: &Path) -> Vec<FileChange> {
    let out = run(repo, &["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        .unwrap_or_default();
    let mut files = Vec::new();
    let mut parts = out.split('\0');
    while let Some(entry) = parts.next() {
        let chars: Vec<char> = entry.chars().collect();
        if chars.len() < 4 {
            continue;
        }
        let (x, y) = (chars[0], chars[1]);
        let path: String = chars[3..].iter().collect();
        if x == 'R' || x == 'C' {
            parts.next(); // skip original name
        }
        files.push(FileChange { path, index: x, work: y });
    }
    files
}

/// Diff of one file. `staged` shows index vs HEAD, otherwise worktree vs index.
pub fn file_diff(repo: &Path, f: &FileChange, staged: bool) -> String {
    if staged {
        return run(repo, &["diff", "--cached", "--", &f.path]).unwrap_or_default();
    }
    if f.work == '?' {
        return run_any(repo, &["diff", "--no-index", "--", "/dev/null", &f.path]);
    }
    run(repo, &["diff", "--", &f.path]).unwrap_or_default()
}

pub fn stage(repo: &Path, path: &str) -> Result<(), String> {
    run(repo, &["add", "-A", "--", path]).map(|_| ())
}

pub fn unstage(repo: &Path, path: &str) -> Result<(), String> {
    run(repo, &["restore", "--staged", "--", path])
        .or_else(|_| run(repo, &["rm", "--cached", "-q", "--", path]))
        .map(|_| ())
}

pub fn discard(repo: &Path, f: &FileChange) -> Result<(), String> {
    if f.work == '?' {
        return run(repo, &["clean", "-fdq", "--", &f.path]).map(|_| ());
    }
    run(repo, &["restore", "--source=HEAD", "--staged", "--worktree", "--", &f.path])
        .or_else(|_| run(repo, &["rm", "-f", "--cached", "-q", "--", &f.path]))
        .map(|_| ())
}

pub fn discard_all(repo: &Path) -> Result<(), String> {
    let _ = run(repo, &["reset", "--hard", "-q"]);
    run(repo, &["clean", "-fdq"]).map(|_| ())
}
