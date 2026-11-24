//! Settings pages: Accounts (GitHub / GitLab) and AI.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn ai_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut add = false;
        page_header(ui, p, "AI", "Saved AI connections for commit messages.", |ui| {
            if ui.add_enabled(!self.ai_adding, egui::Button::new("Add connection")).clicked() {
                add = true;
            }
        });
        if add {
            self.ai_adding = true;
        }
        let mut edit = false;
        let mut remove = false;
        let count = if self.settings.ai_added { "1 connection" } else { "0 connections" };
        settings_group(ui, p, "Connections", count, |_| {}, |ui| {
            if !self.settings.ai_added {
                ui.add_space(26.0);
                let w = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 130.0), Sense::hover());
                let c = rect.center();
                let st = egui::Stroke::new(1.2_f32, p.muted);
                let pt = ui.painter();
                pt.rect_stroke(egui::Rect::from_center_size(c, egui::vec2(88.0, 96.0)), 14.0, st);
                for dx in [-72.0_f32, 72.0] {
                    pt.rect_stroke(egui::Rect::from_center_size(c + egui::vec2(dx, 0.0), egui::vec2(22.0, 48.0)), 6.0, st);
                }
                paint_icon(ui, "userCircle", egui::Rect::from_center_size(c - egui::vec2(0.0, 8.0), egui::vec2(46.0, 46.0)), p.muted);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new("No AI accounts added.").size(15.0).color(p.text));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Add an AI account or local CLI agent, then choose a default for commit messages.").size(12.5).color(p.muted));
                });
                ui.add_space(20.0);
            } else {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        icon(ui, ai_icon(self.settings.ai.provider), 18.0, p.muted);
                        ui.vertical(|ui| {
                            let name = if self.settings.ai.name.trim().is_empty() { self.settings.ai.provider.label().to_string() } else { self.settings.ai.name.clone() };
                            ui.label(RichText::new(name).size(13.5).color(p.text));
                            let sub = match self.settings.ai.provider {
                                git::AiProvider::OpenCode | git::AiProvider::ClaudeCode | git::AiProvider::Codex => "Local CLI agent \u{b7} Default".to_string(),
                                git::AiProvider::Custom => "Custom command \u{b7} Default".to_string(),
                                git::AiProvider::Api => format!("{} \u{b7} Default", self.settings.ai.model),
                            };
                            ui.label(RichText::new(sub).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Remove").clicked() {
                                remove = true;
                            }
                            if ui.button("Edit...").clicked() {
                                edit = true;
                            }
                        });
                    });
                });
            }
        });
        if edit {
            self.ai_adding = true;
        }
        if remove {
            self.settings.ai_added = false;
            self.settings.ai.name.clear();
        }
    }

    /// Right-hand panel with the connection form, as in 1git.
    pub fn ai_panel(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.label(RichText::new(if self.settings.ai_added { "Edit account" } else { "Add account" }).size(14.0).color(p.text));
        ui.add_space(8.0);
        let footer_h = 58.0;
        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - footer_h).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.label(RichText::new("Provider").size(13.0).color(p.text));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let w = (ui.available_width() - 40.0) / 5.0;
                for (pr, label) in [
                    (git::AiProvider::OpenCode, "OpenCode"),
                    (git::AiProvider::ClaudeCode, "Claude Code"),
                    (git::AiProvider::Codex, "ChatGPT"),
                    (git::AiProvider::Custom, "Custom"),
                    (git::AiProvider::Api, "API"),
                ] {
                    if choice_tile(ui, p, ai_icon(pr), label, self.settings.ai.provider == pr, w).clicked() {
                        self.settings.ai.provider = pr;
                    }
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Name").size(13.0).color(p.text));
            let hint = self.settings.ai.provider.label();
            ui.add(egui::TextEdit::singleline(&mut self.settings.ai.name).hint_text(hint).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
            match self.settings.ai.provider {
                git::AiProvider::OpenCode | git::AiProvider::ClaudeCode | git::AiProvider::Codex => {
                    let pr = self.settings.ai.provider;
                    if pr == git::AiProvider::OpenCode {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Plan").size(13.0).color(p.text));
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            let w = (ui.available_width() - 20.0) / 3.0;
                            for (key, label) in [("free", "Free"), ("go", "Go"), ("zen", "Zen")] {
                                if choice_tile(ui, p, "sp_bolt", label, self.settings.ai.plan == key, w).clicked() {
                                    self.settings.ai.plan = key.to_string();
                                }
                            }
                        });
                        ui.label(RichText::new(match self.settings.ai.plan.as_str() {
                            "go" => "Uses your Go subscription. Check OpenCode for usage limits.",
                            "zen" => "Zen uses your OpenCode balance. Generation may incur charges.",
                            _ => "Free models may keep prompts or use them for training. Avoid sensitive code.",
                        }).size(12.0).color(p.muted));
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new("Model").size(13.0).color(p.text));
                    let hint = if pr == git::AiProvider::OpenCode { "provider/model, or empty for the default" } else { "Default model" };
                    ui.add(egui::TextEdit::singleline(&mut self.settings.ai.cli_model).hint_text(hint).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                    ui.label(RichText::new("Effort").size(13.0).color(p.text));
                    egui::ComboBox::from_id_salt("ai_effort")
                        .width(180.0)
                        .selected_text(match self.settings.ai.effort.as_str() { "" => "Default", "low" => "Low", "medium" => "Medium", _ => "High" })
                        .show_ui(ui, |ui| {
                            for (k, l) in [("", "Default"), ("low", "Low"), ("medium", "Medium"), ("high", "High")] {
                                ui.selectable_value(&mut self.settings.ai.effort, k.to_string(), l);
                            }
                        });
                    ui.label(RichText::new("Uses the command line tool already installed and signed in on this Mac.").size(12.0).color(p.muted));
                }
                git::AiProvider::Custom => {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Command").size(13.0).color(p.text));
                    ui.add(egui::TextEdit::singleline(&mut self.settings.ai.command).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                    ui.label(RichText::new("Gets the prompt in $ONAY_PROMPT and the diff on stdin.").size(12.0).color(p.muted));
                }
                git::AiProvider::Api => {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Base URL (OpenAI compatible)").size(13.0).color(p.text));
                    ui.add(egui::TextEdit::singleline(&mut self.settings.ai.base_url).hint_text("https://api.openai.com/v1").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                    ui.label(RichText::new("Model").size(13.0).color(p.text));
                    ui.add(egui::TextEdit::singleline(&mut self.settings.ai.model).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                    ui.label(RichText::new("API key").size(13.0).color(p.text));
                    if self.ai_key_set {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Saved in the Keychain").color(p.muted));
                            if ui.button("Remove key").clicked() {
                                self.clear_ai_key();
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.ai_key_input).password(true).desired_width(280.0).margin(egui::vec2(10.0, 8.0)));
                            if ui.button("Save key").clicked() {
                                self.save_ai_key();
                            }
                        });
                    }
                }
            }
            ui.add_space(6.0);
            ui.label(RichText::new("Commit instructions").size(13.0).color(p.text));
            ui.add(
                egui::TextEdit::multiline(&mut self.settings.ai.instructions)
                    .hint_text("Use Conventional Commits and keep the summary short.")
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(10.0, 8.0)),
            );
            if ui.link("Preset: caveman-commit").clicked() {
                self.settings.ai.instructions = "Write terse commit messages in plain, simple words. One short summary line, no filler.".into();
            }
            ui.add_space(4.0);
            let mut on = true;
            ui.add_enabled(false, egui::Checkbox::new(&mut on, "Use as default for commit messages"));
        });
        let mut save = false;
        let mut cancel = false;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let w = ui.available_width();
            ui.allocate_ui_with_layout(egui::vec2(w, 36.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_button(ui, "Save preferences", true).clicked() {
                    save = true;
                }
                if plain_button(ui, p, "Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if save {
            self.settings.ai_added = true;
            self.settings.save();
            self.ai_adding = false;
        }
        if cancel {
            self.settings = Settings::load();
            self.ai_adding = false;
        }
    }
}

fn ai_icon(pr: git::AiProvider) -> &'static str {
    match pr {
        git::AiProvider::OpenCode => "sp_code",
        git::AiProvider::ClaudeCode => "sparkle",
        git::AiProvider::Codex => "chatgpt",
        git::AiProvider::Custom => "terminal",
        git::AiProvider::Api => "network",
    }
}
