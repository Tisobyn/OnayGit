//! AI commit messages.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn generate_message(&mut self, ctx: &egui::Context) {
        let Some(repo) = self.repo() else { return };
        let mut diff = git::run(&repo, &["diff", "--cached"]).unwrap_or_default();
        if diff.trim().is_empty() {
            diff = git::run(&repo, &["diff", "HEAD"]).unwrap_or_default();
        }
        if diff.trim().is_empty() {
            self.status_msg = "There are no changes to describe".into();
            return;
        }
        let cfg = self.settings.ai.clone();
        let key = git::secret_get(git::AI_SECRET);
        self.spawn_task(ctx, T_AI, move || {
            let (summary, body) = git::generate(&cfg, key.as_deref(), &diff)?;
            Ok(format!("{summary}\u{1f}{body}"))
        });
    }

    pub fn save_ai_key(&mut self) {
        let key = self.ai_key_input.trim().to_string();
        if key.is_empty() {
            return;
        }
        match git::secret_set(git::AI_SECRET, &key) {
            Ok(()) => {
                self.ai_key_set = true;
                self.ai_key_input.clear();
            }
            Err(e) => self.status_msg = format!("Could not save the key: {e}"),
        }
    }

    pub fn clear_ai_key(&mut self) {
        git::secret_delete(git::AI_SECRET);
        self.ai_key_set = false;
    }
}
