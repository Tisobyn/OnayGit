//! History actions: pick commits, cherry-pick, revert, reset, rewrite, conflicts.
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum CommitAction {
    Tag,
    CherryPick,
    Revert,
    ResetSoft,
    ResetMixed,
    ResetHard,
    Reword,
    Squash,
    Drop,
    MoveOlder,
    MoveNewer,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConflictAct {
    Ours,
    Theirs,
    Resolved,
}

impl App {
    /// Load every branch and tag and the commits of the whole repository for the Graph tab.
    pub fn load_graph(&mut self) {
        let Some(repo) = self.repo() else { return };
        self.graph_list = git::ref_list(&repo);
        self.graph_commits = git::log_range(&repo, "--all");
        let (rows, lanes) = git::graph(&self.graph_commits);
        self.graph_rows = rows;
        self.graph_lanes = lanes;
        self.graph_refs.clear();
        for (name, _, hash) in &self.graph_list {
            self.graph_refs.entry(hash.clone()).or_default().push(name.clone());
        }
    }

    pub fn select_commit(&mut self, hash: String) {
        if let Some(repo) = self.repo() {
            self.commit_files = git::commit_files(&repo, &hash);
            self.commit_diff = git::parse_diff(&git::show(&repo, &hash), true);
            let skip = self.commit_diff.iter().position(|l| matches!(l.kind, Kind::File | Kind::Hunk)).unwrap_or(0);
            self.commit_diff.drain(..skip);
            self.commit_info = git::run(
                &repo,
                &["show", "-s", "--date=format:%b %-d, %Y at %-I:%M %p", "--format=%an%x1f%ad%x1f%b", &hash],
            )
            .ok()
            .and_then(|t| {
                let mut p = t.splitn(3, '\u{1f}');
                Some((p.next()?.trim().to_string(), p.next()?.trim().to_string(), p.next().unwrap_or("").trim().to_string()))
            });
        }
        self.selected_commit_file = None;
        self.selected_commit = Some(hash);
    }

    /// Show one file of the selected commit, or all files with `None`.
    pub fn select_commit_file(&mut self, path: Option<String>) {
        let (Some(repo), Some(hash)) = (self.repo(), self.selected_commit.clone()) else { return };
        self.commit_diff = match &path {
            Some(p) => git::parse_diff(&git::show_file(&repo, &hash, p), false),
            None => {
                let mut d = git::parse_diff(&git::show(&repo, &hash), true);
                let skip = d.iter().position(|l| matches!(l.kind, Kind::File | Kind::Hunk)).unwrap_or(0);
                d.drain(..skip);
                d
            }
        };
        self.selected_commit_file = path;
    }

    pub fn commit_action(&mut self, action: CommitAction, hash: String) {
        let Some(repo) = self.repo() else { return };
        let r = match action {
            CommitAction::Tag => {
                self.tag_target = hash;
                self.dialog = Some(Dialog::Tags);
                return;
            }
            CommitAction::ResetHard => {
                self.confirm = Some(Confirm::ResetHard(hash));
                return;
            }
            CommitAction::Drop => {
                self.confirm = Some(Confirm::Rewrite(git::RewriteOp::Drop, hash));
                return;
            }
            CommitAction::Squash => {
                self.confirm = Some(Confirm::Rewrite(git::RewriteOp::Squash, hash));
                return;
            }
            CommitAction::Reword => {
                self.reword_text = self
                    .commits
                    .iter()
                    .find(|c| c.hash == hash)
                    .map(|c| c.subject.clone())
                    .unwrap_or_default();
                self.reword_hash = hash;
                self.dialog = Some(Dialog::Reword);
                return;
            }
            CommitAction::MoveOlder | CommitAction::MoveNewer => {
                let target = if action == CommitAction::MoveOlder {
                    Some(hash.clone())
                } else {
                    self.commits.iter().find(|c| c.parents.contains(&hash)).map(|c| c.hash.clone())
                };
                match target {
                    Some(t) => git::rewrite(&repo, git::RewriteOp::MoveOlder, &t, ""),
                    None => Err("This is the newest commit".into()),
                }
            }
            CommitAction::CherryPick => git::cherry_pick(&repo, &hash),
            CommitAction::Revert => git::revert(&repo, &hash),
            CommitAction::ResetSoft => git::reset_to(&repo, &hash, "soft"),
            CommitAction::ResetMixed => git::reset_to(&repo, &hash, "mixed"),
        };
        self.report(r);
        self.refresh();
    }

    pub fn do_reword(&mut self) {
        let Some(repo) = self.repo() else { return };
        let r = git::rewrite(&repo, git::RewriteOp::Reword, &self.reword_hash, &self.reword_text);
        self.report(r);
        self.refresh();
    }

    pub fn continue_op(&mut self) {
        let (Some(repo), Some(op)) = (self.repo(), self.repo_op) else { return };
        let r = git::op_continue(&repo, op);
        self.report(r);
        self.refresh();
    }

    pub fn resolve_conflict(&mut self, path: &str, act: ConflictAct) {
        let Some(repo) = self.repo() else { return };
        let r = match act {
            ConflictAct::Ours => git::resolve_side(&repo, path, git::Side::Ours),
            ConflictAct::Theirs => git::resolve_side(&repo, path, git::Side::Theirs),
            ConflictAct::Resolved => git::mark_resolved(&repo, path),
        };
        self.report(r);
        self.refresh();
    }
}
