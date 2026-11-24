use crate::prelude::*;

impl App {
    pub fn save_repos(&self) {
        let text: Vec<String> = self.repos.iter().map(|p| p.display().to_string()).collect();
        let _ = std::fs::write(repos_path(), text.join("\n"));
    }

    /// Add a repo to the list. Returns false if the path is not a repository.

    pub fn add_repo(&mut self, path: PathBuf) -> bool {
        if !git::is_repo(&path) {
            self.status_msg = format!("{} is not a git repository", path.display());
            return false;
        }
        if !self.repos.contains(&path) {
            self.repos.push(path);
            self.save_repos();
        }
        true
    }

    pub fn open_repo(&mut self, path: PathBuf) {
        if self.add_repo(path.clone()) {
            self.current = self.repos.iter().position(|p| *p == path);
            self.selected_file = None;
            self.selected_commit = None;
            self.commit_diff.clear();
            self.screen = Screen::Repo;
            self.refresh();
        }
    }

    pub fn scan_folder(&mut self, root: &Path) {
        let found = git::scan(root, 3);
        let n = found.len();
        for p in found {
            self.add_repo(p);
        }
        self.status_msg = if n == 0 { "No repositories found".into() } else { String::new() };
    }

    pub fn remove_repo(&mut self, i: usize) {
        let removed = self.repos.remove(i);
        if let Some(t) = self.tabs.iter().position(|t| *t == removed) {
            self.close_tab(t);
        }
        self.save_repos();
        self.current = match self.current {
            Some(c) if c == i => None,
            Some(c) if c > i => Some(c - 1),
            other => other,
        };
        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        self.ensure_tab();
        let Some(repo) = self.repo() else {
            self.files.clear();
            self.commits.clear();
            self.branches.clear();
            self.branch.clear();
            self.diff.clear();
            return;
        };
        self.files = git::status(&repo);
        self.branch = git::current_branch(&repo);
        self.branches = git::branches(&repo);
        self.ahead_behind = git::ahead_behind(&repo);
        self.has_remote = git::has_remote(&repo);
        self.commits = git::log(&repo);
        self.last_commit = git::last_commit(&repo);
        self.remote_branches = git::remote_branches(&repo);

        self.selected_file = self
            .selected_file
            .clone()
            .filter(|p| self.files.iter().any(|f| &f.path == p))
            .or_else(|| self.files.first().map(|f| f.path.clone()));
        let (graph, width) = git::graph(&self.commits);
        self.graph = graph;
        self.graph_width = width;
        self.repo_op = git::repo_op(&repo);
        self.remotes = git::remotes(&repo);
        self.remote_name = git::primary_remote(&self.remotes);
        self.stashes = git::stashes(&repo);
        self.tags = git::tags(&repo);
        self.worktrees = git::worktrees(&repo);
        if let Some(name) = self.selected_stash.clone() {
            if self.stashes.iter().any(|s| s.name == name) {
                self.stash_diff = git::parse_diff(&git::stash_show(&repo, &name), true);
            } else {
                self.selected_stash = None;
                self.stash_diff.clear();
            }
        }
        self.upstream = git::upstream(&repo);
        self.load_diff(&repo);

        // Compare view
        if let Some(other) = self.compare.clone() {
            let range = if self.compare_outgoing { format!("{other}..HEAD") } else { format!("HEAD..{other}") };
            self.compare_commits = git::log_range(&repo, &range);
            self.compare_counts = git::compare_counts(&repo, &other);
        } else {
            self.compare_commits.clear();
        }
        if self.tab == Tab::Graph {
            self.load_graph();
        }
    }

    /// Reload the diff of the selected file in staged or unstaged mode.
    fn load_diff(&mut self, repo: &Path) {
        let file = self
            .selected_file
            .as_ref()
            .and_then(|p| self.files.iter().find(|f| &f.path == p))
            .cloned();
        self.img_before = None;
        if let Some(f) = &file {
            if git::is_image(&f.path) && !git::is_remote(repo) && f.work != '?' {
                self.img_before = git::blob_to_temp(repo, "HEAD", &f.path);
            }
        }
        let Some(f) = file else {
            self.raw_diff.clear();
            self.diff.clear();
            self.sel_lines.clear();
            return;
        };
        if f.conflicted() {
            self.raw_diff.clear();
            self.sel_lines.clear();
            self.diff = git::conflict_lines(repo, &f.path);
            return;
        }
        // Stay on a side that still has changes.
        if self.diff_staged && !f.staged() {
            self.diff_staged = false;
        }
        if !self.diff_staged && f.work == ' ' {
            self.diff_staged = true;
        }
        let raw = git::file_diff(repo, &f, self.diff_staged);
        if raw != self.raw_diff {
            self.sel_lines.clear();
            self.diff = git::parse_diff(&raw, false);
            self.raw_diff = raw;
        }
    }

    pub fn report<T>(&mut self, r: Result<T, String>) -> bool {
        match r {
            Ok(_) => true,
            Err(e) => {
                self.status_msg = e;
                false
            }
        }
    }

    /// Run a slow command (network) on a thread.

    pub fn spawn_in(&mut self, ctx: &egui::Context, dir: PathBuf, label: &str, args: &[&str]) {
        if self.busy.is_some() {
            return;
        }
        self.busy = Some(label.to_string());
        let env = if self.repo().as_ref() == Some(&dir) { self.auth_env_for(&dir) } else { vec![] };
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let label = label.to_string();
        std::thread::spawn(move || {
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let env_refs: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let res = git::run_env(&dir, &refs, &env_refs);
            let _ = tx.send((label, res));
            ctx.request_repaint();
        });
    }

    pub fn apply(&mut self, action: Action) {
        let Some(repo) = self.repo() else { return };
        match action {
            Action::Stage(p) => {
                let r = git::stage(&repo, &p);
                self.report(r);
            }
            Action::Unstage(p) => {
                let r = git::unstage(&repo, &p);
                self.report(r);
            }
            Action::StageAll => {
                let r = git::run(&repo, &["add", "-A"]);
                self.report(r);
            }
            Action::UnstageAll => {
                let r = git::run(&repo, &["reset", "-q"]);
                self.report(r);
            }
            Action::Select(p) => {
                self.selected_file = Some(p);
                self.selected_stash = None;
                self.sel_lines.clear();
                self.raw_diff.clear();
            }
        }
        self.refresh();
    }
}
