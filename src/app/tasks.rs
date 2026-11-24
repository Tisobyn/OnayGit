//! Background tasks (network, AI, ssh) and their results.
#[allow(unused_imports)]
use crate::prelude::*;

pub const T_SIGN_IN: &str = "Sign in";
pub const T_PUBLISH: &str = "Publish repository";
pub const T_PRS: &str = "Pull requests";
pub const T_NEW_PR: &str = "New pull request";
pub const T_AI: &str = "AI commit message";
pub const T_SCAN: &str = "Scan machine:";
pub const T_TEST: &str = "Test machine:";

/// Open a web page in the default browser.
pub fn open_url(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}

impl App {
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
            (T_SIGN_IN, Err(_)) => self.pending_account = None,
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
                self.summary = s.to_string();
                self.desc = b.to_string();
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
            if label != T_PRS {
                self.status_msg = format!("{label} failed: {e}");
            }
            if label.starts_with(T_TEST) {
                self.pending_machine = None;
            }
        }
        true
    }
}
