//! Stashes, tags and worktrees.
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, Copy)]
pub enum StashOp {
    Apply,
    Pop,
    Drop,
}

impl App {
    /// Stash every change. The summary box text is used as the message.
    pub fn stash_changes(&mut self) {
        let Some(repo) = self.repo() else { return };
        let r = git::stash_push(&repo, &self.summary);
        if self.report(r) {
            self.summary.clear();
            self.stash_open = true;
        }
        self.refresh();
    }

    pub fn select_stash(&mut self, name: Option<String>) {
        self.selected_stash = name;
        self.stash_diff.clear();
        if self.selected_stash.is_some() {
            self.sel_lines.clear();
        }
        self.refresh();
    }

    pub fn stash_op(&mut self, op: StashOp, name: &str) {
        let Some(repo) = self.repo() else { return };
        let r = match op {
            StashOp::Apply => git::stash_apply(&repo, name),
            StashOp::Pop => git::stash_pop(&repo, name),
            StashOp::Drop => git::stash_drop(&repo, name),
        };
        self.report(r);
        self.selected_stash = None;
        self.stash_diff.clear();
        self.refresh();
    }

    pub fn create_tag(&mut self) {
        let Some(repo) = self.repo() else { return };
        let name = self.new_tag.trim().replace(' ', "-");
        if name.is_empty() {
            return;
        }
        let r = git::create_tag(&repo, &name, &self.tag_target, &self.new_tag_msg);
        if self.report(r) {
            self.new_tag.clear();
            self.new_tag_msg.clear();
            self.tag_target.clear();
        }
        self.refresh();
    }

    pub fn delete_tag(&mut self, name: &str) {
        let Some(repo) = self.repo() else { return };
        let r = git::delete_tag(&repo, name);
        self.report(r);
        self.refresh();
    }

    pub fn push_tag(&mut self, ctx: &egui::Context, name: &str) {
        if let Some(repo) = self.repo() {
            let spec = format!("refs/tags/{name}");
            self.spawn_in(ctx, repo, "Pushing tag", &["push", "origin", &spec]);
        }
    }

    /// Create a worktree for `wt_name` inside a folder the user picks.
    pub fn add_worktree(&mut self) {
        let Some(repo) = self.repo() else { return };
        let branch = self.wt_name.trim().replace(' ', "-");
        if branch.is_empty() {
            return;
        }
        let Some(parent) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() else { return };
        let path = parent.join(branch.replace('/', "-"));
        let r = git::worktree_add(&repo, &path, &branch, self.wt_new_branch);
        if self.report(r) {
            self.wt_name.clear();
        }
        self.refresh();
    }

    pub fn remove_worktree(&mut self, path: &Path) {
        let Some(repo) = self.repo() else { return };
        let r = git::worktree_remove(&repo, path, false);
        self.report(r);
        self.refresh();
    }
}
