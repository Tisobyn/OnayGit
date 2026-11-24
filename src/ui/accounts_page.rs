//! Settings pages: Accounts (GitHub / GitLab) and AI.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn ai_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut add = false;
        page_header(ui, p, "AI", "Saved AI connections for commit messages.", |ui| {
            if ui.button("Add connection").clicked() {
                add = true;
            }
        });
        if add {
            self.ai_adding = !self.ai_adding;
        }
        let mut edit = false;
        settings_group(ui, p, "Connections", "1 connection", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    icon(ui, "sparkle", 18.0, p.muted);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.settings.ai.provider.label()).size(13.5).color(p.text));
                        let sub = match self.settings.ai.provider {
                            git::AiProvider::ClaudeCode | git::AiProvider::Codex => "Local CLI agent \u{b7} Default".to_string(),
                            git::AiProvider::Custom => "Custom command \u{b7} Default".to_string(),
                            git::AiProvider::Api => format!("{} \u{b7} Default", self.settings.ai.model),
                        };
                        ui.label(RichText::new(sub).size(12.0).color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Edit").clicked() {
                            edit = true;
                        }
                    });
                });
            });
        });
        if edit {
            self.ai_adding = true;
        }
        if !self.ai_adding {
            return;
        }
        settings_group(ui, p, "Connection", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                field_label(ui, p, "Provider");
                ui.horizontal_wrapped(|ui| {
                    for pr in git::AiProvider::ALL {
                        ui.selectable_value(&mut self.settings.ai.provider, pr, pr.label());
                    }
                });
                match self.settings.ai.provider {
                    git::AiProvider::ClaudeCode | git::AiProvider::Codex => {
                        ui.label(RichText::new("Uses the command line tool already installed and signed in on this Mac.").size(12.0).color(p.muted));
                    }
                    git::AiProvider::Custom => {
                        field_label(ui, p, "Command");
                        ui.label(RichText::new("Gets the prompt in $ONAY_PROMPT and the diff on stdin.").size(12.0).color(p.muted));
                        ui.add(egui::TextEdit::singleline(&mut self.settings.ai.command).desired_width(f32::INFINITY));
                    }
                    git::AiProvider::Api => {
                        field_label(ui, p, "Base URL (OpenAI compatible)");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.ai.base_url).hint_text("https://api.openai.com/v1").desired_width(f32::INFINITY));
                        field_label(ui, p, "Model");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.ai.model).desired_width(f32::INFINITY));
                        field_label(ui, p, "API key");
                        if self.ai_key_set {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Saved in the Keychain").color(p.muted));
                                if ui.button("Remove key").clicked() {
                                    self.clear_ai_key();
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.ai_key_input).password(true).desired_width(280.0));
                                if ui.button("Save key").clicked() {
                                    self.save_ai_key();
                                }
                            });
                        }
                    }
                }
            });
        });
    }
}
