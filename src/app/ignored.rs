//! Ignored changes: files git ignores, with a local snapshot to compare and restore.
#[allow(unused_imports)]
use crate::prelude::*;

const MAX_FILES: usize = 400;
const MAX_BYTES: u64 = 1_000_000;

fn snap_dir(repo: &Path) -> Option<PathBuf> {
    let d = git::run(repo, &["rev-parse", "--absolute-git-dir"]).ok()?;
    Some(PathBuf::from(d.trim()).join("onaygit-ignored"))
}

fn snap_path(repo: &Path, file: &str) -> Option<PathBuf> {
    Some(snap_dir(repo)?.join(file.replace('/', "%2F")))
}

/// Ignored files small enough to track.
pub fn ignored_files(repo: &Path) -> Vec<String> {
    let out = git::run_any(repo, &["ls-files", "-o", "-i", "--exclude-standard", "--directory"]);
    out.lines()
        .filter(|l| !l.is_empty() && !l.ends_with('/'))
        .filter(|l| std::fs::metadata(repo.join(l)).is_ok_and(|m| m.is_file() && m.len() <= MAX_BYTES))
        .take(MAX_FILES)
        .map(String::from)
        .collect()
}

impl App {
    pub fn load_ignored(&mut self) {
        let Some(repo) = self.repo() else { return };
        if git::is_remote(&repo) {
            self.ignored_files.clear();
            return;
        }
        self.ignored_files = ignored_files(&repo);
        self.ignored_refresh_status();
        if self.ignored_sel.as_ref().is_some_and(|s| !self.ignored_files.contains(s)) {
            self.ignored_sel = None;
        }
        self.ignored_diff_load();
    }

    pub fn ignored_diff_load(&mut self) {
        self.ignored_diff.clear();
        self.ignored_changed.clear();
        let (Some(repo), Some(file)) = (self.repo(), self.ignored_sel.clone()) else { return };
        let cur = repo.join(&file);
        let snap = snap_path(&repo, &file).filter(|s| s.exists());
        let a = snap.clone().unwrap_or_else(|| PathBuf::from("/dev/null"));
        let text = git::run_any(&repo, &["diff", "--no-index", "--no-color", "--", &a.display().to_string(), &cur.display().to_string()]);
        self.ignored_diff = git::parse_diff(&text, false);
        self.ignored_has_snap = snap.is_some();
    }

    /// Status of one file, from the cache filled by `ignored_refresh_status`.
    pub fn ignored_status(&self, file: &str) -> char {
        self.ignored_st.get(file).copied().unwrap_or(' ')
    }

    /// Compare every listed file with its snapshot once, not on every frame.
    pub fn ignored_refresh_status(&mut self) {
        self.ignored_st.clear();
        let Some(repo) = self.repo() else { return };
        let Some(dir) = snap_dir(&repo) else { return };
        for f in &self.ignored_files {
            let snap = dir.join(f.replace('/', "%2F"));
            let st = if !snap.exists() {
                'A'
            } else if std::fs::read(&snap).ok() == std::fs::read(repo.join(f)).ok() {
                ' '
            } else {
                'M'
            };
            self.ignored_st.insert(f.clone(), st);
        }
    }

    pub fn ignored_snapshot(&mut self) {
        let Some(repo) = self.repo() else { return };
        let Some(dir) = snap_dir(&repo) else { return };
        let _ = std::fs::create_dir_all(&dir);
        for f in &self.ignored_files {
            if let Some(s) = snap_path(&repo, f) {
                let _ = std::fs::copy(repo.join(f), s);
            }
        }
        self.status_msg = format!("Saved {} ignored files", self.ignored_files.len());
        self.ignored_refresh_status();
        self.ignored_diff_load();
    }

    pub fn ignored_restore(&mut self) {
        let (Some(repo), Some(file)) = (self.repo(), self.ignored_sel.clone()) else { return };
        if let Some(s) = snap_path(&repo, &file).filter(|s| s.exists()) {
            match std::fs::copy(s, repo.join(&file)) {
                Ok(_) => self.status_msg = format!("Restored {file}"),
                Err(e) => self.status_msg = format!("Restore failed: {e}"),
            }
        }
        self.ignored_refresh_status();
        self.ignored_diff_load();
    }
}
