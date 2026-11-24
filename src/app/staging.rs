//! Partial staging, compare and ignore actions.
use crate::prelude::*;

impl App {
    /// Stage (or unstage) the selected lines, or one whole hunk.
    pub fn apply_lines(&mut self, only_hunk: Option<usize>) {
        let Some(repo) = self.repo() else { return };
        let Some(path) = self.selected_file.clone() else { return };
        let Some(file) = self.files.iter().find(|f| f.path == path).cloned() else { return };
        let reverse = self.diff_staged;
        let Some(patch) =
            git::build_patch(&self.raw_diff, &path, &self.diff, &self.sel_lines, only_hunk, reverse)
        else {
            self.status_msg = "Select at least one changed line".into();
            return;
        };
        // An untracked file must be known to the index before it can be patched.
        if !reverse && file.work == '?' {
            let r = git::run(&repo, &["add", "-N", "--", &path]);
            if !self.report(r) {
                return;
            }
        }
        let r = git::apply_patch(&repo, &patch, reverse);
        if self.report(r) {
            self.sel_lines.clear();
        }
        self.refresh();
    }

    /// Pick the branch to compare against, or None to show plain history.
    pub fn set_compare(&mut self, branch: Option<String>) {
        self.compare = branch;
        self.selected_commit = None;
        self.commit_diff.clear();
        self.refresh();
    }

    /// Append a pattern to the repository `.gitignore`.
    pub fn ignore(&mut self, pattern: &str) {
        let Some(repo) = self.repo() else { return };
        let r = git::ignore_pattern(&repo, pattern);
        self.report(r);
        self.refresh();
    }
}
