#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn open_settings(&mut self) {
        self.git_name = git::config_get("user.name");
        self.git_email = git::config_get("user.email");
        self.screen = Screen::Settings;
    }

    pub fn settings_screen(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut back = false;
        egui::SidePanel::left("settings_rail")
            .exact_width(260.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.home).inner_margin(egui::Margin::symmetric(14.0, 22.0)))
            .show(ctx, |ui| {
                if rail_item(ui, "arrowLeft", "Back", false, &p).clicked() {
                    back = true;
                }
                ui.add_space(18.0);
                rail_label(ui, "APPLICATION", &p);
                for (s, icon, name) in [
                    (Section::Application, "settings", "Application"),
                    (Section::Appearance, "contrast", "Appearance"),
                    (Section::Git, "branch", "Git"),
                    (Section::Editor, "pencil", "External editor"),
                    (Section::Ignore, "stop", "Ignored files"),
                    (Section::Help, "help", "Help"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                        if s == Section::Ignore {
                            self.ignore_text = self.repo().map(|r| git::read_ignore(&r)).unwrap_or_default();
                        }
                    }
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.home).inner_margin(egui::Margin::symmetric(40.0, 26.0)))
            .show(ctx, |ui| {
                ui.set_max_width(640.0);
                match self.section {
                    Section::Application => {
                        page_title(ui, "Application", &p);
                        setting_row(ui, "Auto refresh", "How often changes are re-read from disk.", &p, |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.settings.refresh_secs, 1..=30).suffix(" s"),
                            );
                        });
                        setting_row(ui, "Repositories", "Saved list of repositories.", &p, |ui| {
                            ui.label(RichText::new(format!("{} saved", self.repos.len())).color(p.muted));
                        });
                    }
                    Section::Appearance => {
                        page_title(ui, "Appearance", &p);
                        setting_row(ui, "Theme", "Dark or light colors.", &p, |ui| {
                            ui.selectable_value(&mut self.settings.dark, true, "Dark");
                            ui.selectable_value(&mut self.settings.dark, false, "Light");
                        });
                        setting_row(ui, "Diff font size", "Text size in the diff view.", &p, |ui| {
                            ui.add(egui::Slider::new(&mut self.settings.diff_font, 10.0..=18.0));
                        });
                        setting_row(ui, "Interface zoom", "Scale the whole app.", &p, |ui| {
                            let r = ui.add(egui::Slider::new(&mut self.settings.zoom, 0.8..=1.5));
                            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                                ctx.set_zoom_factor(self.settings.zoom);
                            }
                        });
                    }
                    Section::Git => {
                        page_title(ui, "Git", &p);
                        ui.label(RichText::new("Commit identity (global git config)").color(p.muted));
                        ui.add_space(8.0);
                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut self.git_name).desired_width(360.0));
                        ui.add_space(6.0);
                        ui.label("Email");
                        ui.add(egui::TextEdit::singleline(&mut self.git_email).desired_width(360.0));
                        ui.add_space(10.0);
                        if ui.button("Save identity").clicked() {
                            let a = git::config_set("user.name", self.git_name.trim());
                            let b = git::config_set("user.email", self.git_email.trim());
                            if let Err(e) = a.and(b) {
                                self.status_msg = e;
                            }
                        }
                    }
                    Section::Editor => {
                        page_title(ui, "External editor", &p);
                        setting_row(ui, "Editor command", "Used by Open in editor.", &p, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.settings.editor).desired_width(200.0));
                        });
                        ui.horizontal(|ui| {
                            for (label, cmd) in [
                                ("VS Code", "code"),
                                ("Cursor", "cursor"),
                                ("Zed", "zed"),
                                ("Sublime", "subl"),
                                ("Xcode", "open -a Xcode"),
                            ] {
                                if ui.button(label).clicked() {
                                    self.settings.editor = cmd.to_string();
                                }
                            }
                        });
                    }
                    Section::Ignore => {
                        page_title(ui, "Ignored files", &p);
                        match self.repo() {
                            None => {
                                ui.label(RichText::new("Open a repository first.").color(p.muted));
                            }
                            Some(repo) => {
                                ui.label(
                                    RichText::new(format!(".gitignore of {}", repo_name(&repo))).color(p.muted),
                                );
                                ui.add_space(6.0);
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.ignore_text)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(14)
                                        .desired_width(f32::INFINITY),
                                );
                                ui.add_space(8.0);
                                if ui.button("Save .gitignore").clicked() {
                                    let r = git::write_ignore(&repo, &self.ignore_text);
                                    self.report(r);
                                    self.refresh();
                                }
                            }
                        }
                    }
                    Section::Help => {
                        page_title(ui, "Help", &p);
                        ui.label(format!("OnayGit {}", env!("CARGO_PKG_VERSION")));
                        ui.label(RichText::new("A simple Git client written in Rust.").color(p.muted));
                        ui.add_space(10.0);
                        ui.label(RichText::new(format!("Repository list: {}", repos_path().display())).color(p.muted));
                        ui.label(RichText::new(format!("Settings file: {}", settings_path().display())).color(p.muted));
                    }
                }
            });

        if back {
            self.screen = if self.current.is_some() { Screen::Repo } else { Screen::Home };
        }
    }
}
