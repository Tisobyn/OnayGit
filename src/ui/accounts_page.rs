//! Settings pages: Accounts (GitHub / GitLab) and AI.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn accounts_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        page_title(ui, "Accounts", p);
        let mut out: Option<usize> = None;
        if self.accounts.is_empty() {
            ui.label(RichText::new("No accounts yet.").color(p.muted));
        }
        for (i, a) in self.accounts.iter().enumerate() {
            ui.horizontal(|ui| {
                icon(ui, if a.provider == git::Provider::GitHub { "github" } else { "gitlab" }, 18.0, p.text);
                ui.label(format!("{} on {}", a.user, a.host));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Sign out").clicked() {
                        out = Some(i);
                    }
                });
            });
        }
        if let Some(i) = out {
            self.sign_out(i);
        }
        ui.add_space(14.0);
        ui.label(RichText::new("Add account").size(15.0));
        ui.label(RichText::new("Use a personal access token. It is kept in the macOS Keychain.").color(p.muted));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for pr in [git::Provider::GitHub, git::Provider::GitLab] {
                if ui.selectable_label(self.acct_provider == pr, pr.label()).clicked() {
                    self.acct_provider = pr;
                    self.acct_host.clear();
                }
            }
        });
        ui.add(
            egui::TextEdit::singleline(&mut self.acct_host)
                .hint_text(self.acct_provider.default_host())
                .desired_width(360.0),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.acct_token)
                .password(true)
                .hint_text("Access token")
                .desired_width(360.0),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let busy = self.task_running(T_SIGN_IN);
            if ui.add_enabled(!busy && !self.acct_token.trim().is_empty(), egui::Button::new("Sign in")).clicked() {
                self.sign_in(ctx);
            }
            if busy {
                ui.spinner();
            }
        });
        if !self.status_msg.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.status_msg).color(p.muted));
        }
    }

    pub fn ai_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_title(ui, "AI commit messages", p);
        setting_row(ui, "Provider", "Who writes the message.", p, |ui| {
            for pr in git::AiProvider::ALL {
                ui.selectable_value(&mut self.settings.ai.provider, pr, pr.label());
            }
        });
        match self.settings.ai.provider {
            git::AiProvider::ClaudeCode | git::AiProvider::Codex => {
                ui.label(
                    RichText::new("Uses the command line tool already installed and signed in on this Mac.")
                        .color(p.muted),
                );
            }
            git::AiProvider::Custom => {
                ui.label("Command (gets the prompt in $ONAY_PROMPT and the diff on stdin)");
                ui.add(egui::TextEdit::singleline(&mut self.settings.ai.command).desired_width(420.0));
            }
            git::AiProvider::Api => {
                ui.label("Base URL (OpenAI compatible)");
                ui.add(
                    egui::TextEdit::singleline(&mut self.settings.ai.base_url)
                        .hint_text("https://api.openai.com/v1")
                        .desired_width(420.0),
                );
                ui.label("Model");
                ui.add(egui::TextEdit::singleline(&mut self.settings.ai.model).desired_width(240.0));
                ui.add_space(6.0);
                ui.label("API key");
                if self.ai_key_set {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Saved in the Keychain").color(p.muted));
                        if ui.button("Remove key").clicked() {
                            self.clear_ai_key();
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.ai_key_input).password(true).desired_width(300.0));
                        if ui.button("Save key").clicked() {
                            self.save_ai_key();
                        }
                    });
                }
            }
        }
        ui.add_space(10.0);
        ui.label(RichText::new("Use the sparkle button next to the commit summary.").color(p.muted));
    }
}
