//! Settings pages: Accounts (GitHub / GitLab) and AI.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
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
