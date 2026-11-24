//! GitHub and GitLab accounts. Tokens are kept in the Keychain, never in files.
#[allow(unused_imports)]
use crate::prelude::*;

fn accounts_path() -> PathBuf {
    home_dir().join(".onaygit_accounts")
}

pub fn load_accounts() -> Vec<git::Account> {
    std::fs::read_to_string(accounts_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            Some(git::Account {
                provider: git::Provider::from_key(p.next()?)?,
                host: p.next()?.to_string(),
                user: p.next()?.to_string(),
            })
        })
        .collect()
}

impl App {
    fn save_accounts(&self) {
        let text: Vec<String> = self
            .accounts
            .iter()
            .map(|a| format!("{}\t{}\t{}", a.provider.key(), a.host, a.user))
            .collect();
        let _ = std::fs::write(accounts_path(), text.join("\n"));
    }

    /// Check the token on a background thread, then save the account.
    pub fn sign_in(&mut self, ctx: &egui::Context) {
        let provider = self.acct_provider;
        let host = if self.acct_host.trim().is_empty() {
            provider.default_host().to_string()
        } else {
            self.acct_host.trim().to_string()
        };
        let cli = self.acct_cli && provider == git::Provider::GitHub;
        let token = self.acct_token.trim().to_string();
        if token.is_empty() && !cli {
            return;
        }
        self.pending_account = Some((provider, host.clone(), token.clone()));
        self.spawn_task(ctx, T_SIGN_IN, move || {
            let token = if cli { git::gh_cli_token(&host)? } else { token };
            let user = git::whoami(provider, &host, &token)?;
            Ok(format!("{user}\u{1f}{token}"))
        });
    }

    pub fn finish_sign_in(&mut self, result: &str) {
        let Some((provider, host, token)) = self.pending_account.take() else { return };
        let (user, token) = match result.split_once('\u{1f}') {
            Some((u, t)) => (u, t.to_string()),
            None => (result, token),
        };
        let acc = git::Account { provider, host, user: user.to_string() };
        match git::secret_set(&acc.secret_name(), &token) {
            Ok(()) => {
                self.accounts.retain(|a| a.secret_name() != acc.secret_name());
                self.accounts.push(acc);
                self.save_accounts();
                self.acct_token.clear();
                self.acct_adding = false;
                self.status_msg.clear();
            }
            Err(e) => self.status_msg = format!("Could not save the token in the Keychain: {e}"),
        }
    }

    pub fn sign_out(&mut self, i: usize) {
        if i < self.accounts.len() {
            let acc = self.accounts.remove(i);
            git::secret_delete(&acc.secret_name());
            self.save_accounts();
        }
    }

    /// The saved account and `owner/repo` that match this repository's remote.
    pub fn account_for_repo(&self) -> Option<(git::Account, String)> {
        let url = self.remotes.iter().find(|r| r.name == self.remote_name)?.url.clone();
        let (host, slug) = git::parse_remote_url(&url)?;
        let acc = self.accounts.iter().find(|a| a.host == host)?.clone();
        Some((acc, slug))
    }
}
