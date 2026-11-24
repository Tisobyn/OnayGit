//! Background tasks (network, AI, ssh) and their results.
#[allow(unused_imports)]
use crate::prelude::*;

pub const T_SIGN_IN: &str = "Sign in";
pub const T_DEVICE: &str = "Device code";
pub const T_CHECK: &str = "Check account:";
pub const T_PUBLISH: &str = "Publish repository";
pub const T_PRS: &str = "Pull requests";
pub const T_NEW_PR: &str = "New pull request";
pub const T_AI: &str = "AI commit message";
pub const T_SCAN: &str = "Scan machine:";
pub const T_INFO: &str = "Repo info";
pub const T_TEST: &str = "Test machine:";
pub const T_AVATAR: &str = "Avatar:";

/// Open a web page in the default browser.
pub fn open_url(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}

impl App {
    /// Refresh branch and change counts of the listed repositories (home screen).
    pub fn load_repo_info(&mut self, ctx: &egui::Context) {
        if self.info_at.is_some_and(|t| t.elapsed().as_secs() < 8) || self.task_running(T_INFO) {
            return;
        }
        self.info_at = Some(Instant::now());
        let repos: Vec<PathBuf> = self.repos.iter().filter(|r| !git::is_remote(r) && r.exists()).cloned().collect();
        self.spawn_task(ctx, T_INFO, move || {
            let mut out = vec![];
            for r in repos {
                let branch = git::current_branch(&r);
                let dirty = git::run(&r, &["status", "--porcelain"]).map(|s| s.lines().count()).unwrap_or(0);
                let (a, b) = match git::ahead_behind(&r) {
                    Some((a, b)) => (a.to_string(), b.to_string()),
                    None => ("-".to_string(), "-".to_string()),
                };
                out.push(format!("{}\t{branch}\t{dirty}\t{a}\t{b}", r.display()));
            }
            Ok(out.join("\n"))
        });
    }

    pub fn spawn_task(
        &mut self,
        ctx: &egui::Context,
        label: &str,
        f: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        if self.tasks.contains(label) {
            return;
        }
        self.tasks.insert(label.to_string());
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let label = label.to_string();
        std::thread::spawn(move || {
            let result = f();
            let _ = tx.send((label, result));
            ctx.request_repaint();
        });
    }

    /// Start fetching the picture of `email` unless known or already running.
    /// `sha` is a commit by this person, if the email is the commit author's.
    pub fn want_avatar(&mut self, ctx: &egui::Context, email: &str, sha: Option<&str>) {
        let key = email.trim().to_lowercase();
        if !self.settings.avatars || key.is_empty() || self.avatars.contains_key(&key) {
            return;
        }
        let label = format!("{T_AVATAR}{key}");
        if self.task_running(&label) || self.tasks.iter().filter(|t| t.starts_with(T_AVATAR)).count() >= 4 {
            return;
        }
        let slug = self
            .remotes
            .iter()
            .find(|r| r.name == self.remote_name)
            .and_then(|r| git::parse_remote_url(&r.url))
            .filter(|(h, _)| h == "github.com")
            .map(|(_, s)| s);
        let hint = slug.zip(sha.map(String::from));
        self.spawn_task(ctx, &label, move || {
            let h = hint.as_ref().map(|(s, c)| (s.as_str(), c.as_str()));
            git::fetch_avatar(&key, h).map(|p| p.display().to_string())
        });
    }

    pub fn task_running(&self, label: &str) -> bool {
        self.tasks.contains(label)
    }

    /// Environment that lets git use the saved account of this repository's remote.
    pub fn auth_env_for(&self, repo: &Path) -> Vec<(String, String)> {
        if git::is_remote(repo) {
            return vec![];
        }
        let Some(url) = self.remotes.iter().find(|r| r.name == self.remote_name).map(|r| r.url.clone()) else {
            return vec![];
        };
        if !url.starts_with("http") {
            return vec![];
        }
        let Some((host, _)) = git::parse_remote_url(&url) else { return vec![] };
        let Some(acc) = self.accounts.iter().find(|a| a.host == host) else { return vec![] };
        let Some(token) = acc.token() else { return vec![] };
        git::auth_env(acc, &token).unwrap_or_default()
    }

    /// Handle the result of a task started with `spawn_task`. False if it was not one.
    pub fn handle_task_result(&mut self, label: &str, res: &Result<String, String>) -> bool {
        if !self.tasks.remove(label) {
            return false;
        }
        match (label, res) {
            (T_SIGN_IN, Ok(user)) => self.finish_sign_in(user),
            (T_SIGN_IN, Err(e)) => {
                self.pending_account = None;
                self.device = None;
                self.device_waiting = false;
                if self.acct_adding {
                    self.status_msg = e.clone();
                }
            }
            (l, r) if l.starts_with(T_CHECK) => {
                let key = l[T_CHECK.len()..].to_string();
                self.acct_status.insert(key, match r { Ok(s) => s.clone(), Err(e) => format!("Failed: {}", e.chars().take(40).collect::<String>()) });
            }
            (T_DEVICE, Ok(s)) => {
                let mut p = s.split('\u{1f}');
                self.device = Some(git::DeviceCode {
                    device_code: p.next().unwrap_or_default().to_string(),
                    user_code: p.next().unwrap_or_default().to_string(),
                    uri: p.next().unwrap_or_default().to_string(),
                    interval: p.next().and_then(|v| v.parse().ok()).unwrap_or(5),
                });
                self.device_waiting = false;
            }
            (l, r) if l.starts_with(crate::ui::clone::T_REPOS) => {
                let key = l[crate::ui::clone::T_REPOS.len()..].to_string();
                self.handle_repos_result(&key, r);
            }
            (l, r) if l.starts_with(T_AVATAR) => {
                self.avatars.insert(l[T_AVATAR.len()..].to_string(), r.as_ref().ok().map(PathBuf::from));
            }
            (T_DEVICE, Err(e)) => self.status_msg = e.clone(),
            (T_PRS, Ok(tsv)) => {
                self.prs = tsv.lines().filter_map(git::PullRequest::from_line).collect();
                self.pr_error.clear();
            }
            (T_PRS, Err(e)) => self.pr_error = e.clone(),
            (T_NEW_PR, Ok(url)) => {
                open_url(url);
                self.pr_title.clear();
                self.pr_body.clear();
                self.status_msg.clear();
            }
            (T_PUBLISH, Ok(_)) => {
                self.dialog = None;
                self.status_msg.clear();
                self.refresh();
            }
            (T_AI, Ok(text)) => {
                let (s, b) = text.split_once('\u{1f}').unwrap_or((text.as_str(), ""));
                self.set_message(s, b);
            }
            (T_INFO, Ok(lines)) => {
                for l in lines.lines() {
                    let f: Vec<&str> = l.split('\t').collect();
                    if f.len() == 5 {
                        let sync = (f[3] != "-").then(|| (f[3].parse().unwrap_or(0), f[4].parse().unwrap_or(0)));
                        self.repo_info.insert(
                            PathBuf::from(f[0]),
                            RepoInfo { branch: f[1].to_string(), dirty: f[2].parse().unwrap_or(0), sync },
                        );
                    }
                }
            }
            (l, Ok(lines)) if l.starts_with(T_SCAN) => {
                let host = l[T_SCAN.len()..].to_string();
                self.machine_repos.insert(host, lines.lines().map(String::from).collect());
            }
            (l, Ok(_)) if l.starts_with(T_TEST) => {
                if let Some(m) = self.pending_machine.take() {
                    self.add_machine_confirmed(m);
                }
            }
            _ => {}
        }
        if let Err(e) = res {
            let quiet = label == T_PRS || label == T_SIGN_IN || label == T_DEVICE || label.starts_with(T_CHECK) || label.starts_with(T_AVATAR) || label.starts_with(crate::ui::clone::T_REPOS);
            if !quiet {
                self.status_msg = match label.strip_prefix(T_TEST) {
                    Some(host) => format!("Could not connect to {host}: {e}"),
                    None => format!("{label} failed: {e}"),
                };
            }
            if label.starts_with(T_TEST) {
                self.pending_machine = None;
            }
        }
        true
    }
}
