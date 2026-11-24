//! Publish a repository and work with pull / merge requests.
use crate::prelude::*;

impl App {
    pub fn publish(&mut self, ctx: &egui::Context) {
        let Some(repo) = self.repo() else { return };
        if git::is_remote(&repo) {
            self.status_msg = "Publishing is not available for repositories on other machines".into();
            return;
        }
        let Some(acc) = self.accounts.get(self.publish_account).cloned() else {
            self.status_msg = "Add an account in Settings > Accounts first".into();
            return;
        };
        let Some(token) = acc.token() else {
            self.status_msg = "The token for this account is missing from the Keychain".into();
            return;
        };
        let (name, private, desc) = (self.publish_name.trim().to_string(), self.publish_private, self.publish_desc.clone());
        self.spawn_task(ctx, T_PUBLISH, move || {
            let url = git::create_repo(&acc, &token, &name, private, &desc)?;
            git::run(&repo, &["remote", "add", "origin", &url])?;
            let env = git::auth_env(&acc, &token)?;
            let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            git::run_env(&repo, &["push", "-u", "origin", "HEAD"], &env)
        });
    }

    pub fn load_prs(&mut self, ctx: &egui::Context) {
        let Some((acc, slug)) = self.account_for_repo() else {
            self.pr_error = "No saved account matches this repository's remote. Add one in Settings > Accounts.".into();
            return;
        };
        let Some(token) = acc.token() else {
            self.pr_error = "The token for this account is missing from the Keychain".into();
            return;
        };
        self.pr_error.clear();
        self.spawn_task(ctx, T_PRS, move || {
            let list = git::list_pull_requests(&acc, &token, &slug)?;
            Ok(list.iter().map(|p| p.to_line()).collect::<Vec<_>>().join("\n"))
        });
    }

    /// Push the current branch, then open a pull request for it.
    pub fn create_pr(&mut self, ctx: &egui::Context) {
        let (Some(repo), Some((acc, slug))) = (self.repo(), self.account_for_repo()) else {
            self.status_msg = "No saved account matches this repository's remote".into();
            return;
        };
        let Some(token) = acc.token() else {
            self.status_msg = "The token for this account is missing from the Keychain".into();
            return;
        };
        let (title, body, base) = (self.pr_title.trim().to_string(), self.pr_body.clone(), self.pr_base.trim().to_string());
        let (head, remote) = (self.branch.clone(), self.remote_name.clone());
        self.spawn_task(ctx, T_NEW_PR, move || {
            let env = git::auth_env(&acc, &token)?;
            let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            git::run_env(&repo, &["push", "-u", &remote, "HEAD"], &env)?;
            git::create_pull_request(&acc, &token, &slug, &title, &body, &head, &base)
        });
    }

    /// Fill the pull request form with sensible defaults.
    pub fn prepare_pr_form(&mut self) {
        if self.pr_title.is_empty() {
            self.pr_title = self.last_commit.as_ref().map(|c| c.0.clone()).unwrap_or_default();
        }
        if self.pr_base.is_empty() {
            self.pr_base = ["main", "master", "develop"]
                .iter()
                .find(|b| self.branches.iter().any(|x| x == **b) && **b != self.branch)
                .map(|b| b.to_string())
                .unwrap_or_else(|| "main".to_string());
        }
    }
}
