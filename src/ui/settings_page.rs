#[allow(unused_imports)]
use crate::prelude::*;
use crate::app::{CODE_FONTS, UI_FONTS};
use crate::app::spaces::{space_icon_name, SPACE_COLORS, SPACE_ICONS};

impl App {
    pub fn open_settings(&mut self) {
        self.git_name = git::config_get("user.name");
        self.git_email = git::config_get("user.email");
        self.ai_key_set = git::secret_get(git::AI_SECRET).is_some();
        self.status_msg.clear();
        self.screen = Screen::Settings;
    }

    pub fn settings_screen(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        egui::SidePanel::left("settings_rail")
            .exact_width(190.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 16.0, right: 12.0, top: 24.0, bottom: 16.0 }))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                rail_label(ui, "Application", &p);
                for (s, icon, name) in [
                    (Section::Application, "settings", "Application"),
                    (Section::Spaces, "layers", "Spaces"),
                    (Section::Appearance, "contrast", "Appearance"),
                    (Section::Keybinds, "keyboard", "Keybinds"),
                    (Section::Accounts, "userCircle", "Accounts"),
                    (Section::Git, "branch", "Git"),
                    (Section::Editor, "pencil", "External editor"),
                    (Section::Ai, "sparkle", "AI"),
                    (Section::Help, "help", "Help"),
                    (Section::About, "info", "About"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                    }
                }
                ui.add_space(14.0);
                rail_label(ui, "Repository", &p);
                for (s, icon, name) in [
                    (Section::Remotes, "network", "Remotes"),
                    (Section::Ignore, "ignored", "Ignored files"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                        if s == Section::Ignore {
                            self.ignore_text = self.repo().map(|r| git::read_ignore(&r)).unwrap_or_default();
                        }
                    }
                }
            });

        let adding = self.section == Section::Accounts && self.acct_adding;
        if adding {
            egui::SidePanel::right("account_add")
                .exact_width(514.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
                .show(ctx, |ui| self.account_add_panel(ctx, ui, &p));
        }
        self.space_dialog_ui(ctx);
        if let (Section::Editor, Some(target)) = (self.section, self.rule_dialog) {
            egui::SidePanel::right("rule_panel")
                .exact_width(514.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
                .show(ctx, |ui| self.rule_panel(ui, &p, target));
        }
        if self.section == Section::Ai && self.ai_adding {
            egui::SidePanel::right("ai_panel")
                .exact_width(514.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
                .show(ctx, |ui| self.ai_panel(ui, &p));
        }
        if self.section == Section::Git && self.git_editing {
            egui::SidePanel::right("identity_panel")
                .exact_width(514.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
                .show(ctx, |ui| self.identity_panel(ui, &p));
        }
        if self.section == Section::Appearance {
            egui::SidePanel::right("appearance_preview")
                .exact_width(330.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
                .show(ctx, |ui| appearance_preview(ui, &p));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 24.0, right: 32.0, top: 20.0, bottom: 20.0 }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_max_width(match self.section {
                        Section::Spaces | Section::Git | Section::Editor | Section::Ai | Section::Accounts => 480.0,
                        Section::Privacy | Section::Help | Section::About => 560.0,
                        Section::Keybinds => 760.0,
                        _ => 580.0,
                    });
                    match self.section {
                        Section::Application => self.application_page(ui, &p),
                        Section::Spaces => self.spaces_page(ui, &p),
                        Section::Appearance => self.appearance_page(ctx, ui, &p),
                        Section::Keybinds => self.keybinds_page(ui, &p),
                        Section::Privacy => privacy_page(ui, &p),
                        Section::Git => self.git_page(ui, &p),
                        Section::Editor => self.editor_page(ui, &p),
                        Section::Ignore => self.ignore_page(ui, &p),
                        Section::Accounts => self.accounts_page(ui, &p),
                        Section::Ai => self.ai_page(ui, &p),
                        Section::Remotes => self.remotes_page(ui, &p),
                        Section::Help => self.help_page(ui, &p),
                        Section::About => self.about_page(ui, &p),
                    }
                });
            });
    }

    fn application_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Application", "Install the onaygit command so local paths open in this app.", |_| {});
        let user_bin = home_dir().join(".local/bin/onaygit");
        let sys_bin = PathBuf::from("/usr/local/bin/onaygit");
        let mut act: Option<(bool, bool)> = None;
        settings_group(ui, p, "Command-line launcher", "", |_| {}, |ui| {
            for (system, name, desc, bin) in [
                (true, "All users", "Install in /usr/local/bin. macOS may ask for administrator approval.", &sys_bin),
                (false, "Only me", "Install in ~/.local/bin without administrator access.", &user_bin),
            ] {
                let installed = bin.exists();
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(name).size(13.5).color(p.text));
                            ui.label(RichText::new(desc).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(if installed { "Uninstall" } else { "Install" }).clicked() {
                                act = Some((system, installed));
                            }
                            ui.label(RichText::new(if installed { "Installed" } else { "Not installed" }).size(12.0).color(p.muted));
                        });
                    });
                });
            }
        });
        if !self.launcher_msg.is_empty() {
            ui.label(RichText::new(&self.launcher_msg).size(12.0).color(p.muted));
        } else {
            ui.label(
                RichText::new(format!("The per-user command will be installed at {}. Add ~/.local/bin to PATH if your shell cannot find it.", user_bin.parent().unwrap().display()))
                    .size(12.0)
                    .color(p.muted),
            );
        }
        ui.add_space(14.0);
        if let Some((system, installed)) = act {
            let bin = if system { sys_bin } else { user_bin };
            self.launcher_msg = match toggle_launcher(&bin, installed, system) {
                Ok(m) => m,
                Err(e) => e,
            };
        }
        settings_group(ui, p, "General", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Auto refresh").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.refresh_secs, 1..=30).suffix(" s"));
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Guided tips").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.settings.tips, "");
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Welcome window").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Show again").clicked() {
                            self.settings.onboarded = false;
                        }
                    });
                });
            });
        });
    }

    fn spaces_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut new = false;
        page_header(ui, p, "Spaces", "Keep separate repositories and tabs for each part of your day.", |ui| {
            if ui.button("New space...").clicked() {
                new = true;
            }
        });
        let mut edit: Option<usize> = None;
        let mut switch: Option<usize> = None;
        let count = format!("{} space{}", self.spaces.len(), if self.spaces.len() == 1 { "" } else { "s" });
        settings_group(ui, p, "Spaces", &count, |_| {}, |ui| {
            if self.spaces.is_empty() {
                group_row(ui, p, |ui| {
                    ui.label(RichText::new("No spaces yet").color(p.muted));
                });
            }
            for (i, s) in self.spaces.iter().enumerate() {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        icon(ui, &space_icon_name(&s.icon), 18.0, if s.color.is_some() { space_color32(s.color) } else { p.muted });
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&s.name).color(p.text));
                            let n = s.repos.len();
                            ui.label(RichText::new(format!("{n} repositor{}", if n == 1 { "y" } else { "ies" })).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.selected_space == Some(i) {
                                ui.label(RichText::new("Current").size(12.5).color(p.muted));
                            } else if ui.button("Switch").clicked() {
                                switch = Some(i);
                            }
                            if ui.button("Edit...").clicked() {
                                edit = Some(i);
                            }
                        });
                    });
                });
            }
        });
        if new {
            self.space_name.clear();
            self.space_icon = "layers".into();
            self.space_color = None;
            self.space_dialog = Some(None);
        }
        if let Some(i) = edit {
            self.space_name = self.spaces[i].name.clone();
            self.space_icon = self.spaces[i].icon.clone();
            self.space_color = self.spaces[i].color;
            self.space_dialog = Some(Some(i));
        }
        if let Some(i) = switch {
            self.set_space(Some(i));
        }
    }

    /// Right-hand panel to create or edit a space, as in 1git.
    pub fn space_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(target) = self.space_dialog else { return };
        if self.section != Section::Spaces || self.screen != Screen::Settings {
            return;
        }
        let p = self.pal();
        let mut save = false;
        let mut delete = false;
        let mut close = false;
        egui::SidePanel::right("space_panel")
            .exact_width(514.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.label(RichText::new(if target.is_some() { "Edit space" } else { "New space" }).size(14.0).color(p.text));
                ui.add_space(16.0);
                ui.label(RichText::new("Name").size(13.0).color(p.text));
                ui.add_space(4.0);
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.space_name)
                        .hint_text("Work, Personal...")
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(10.0, 8.0)),
                );
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    save = true;
                }
                ui.add_space(16.0);
                ui.label(RichText::new("Icon").size(13.0).color(p.text));
                ui.add_space(8.0);
                let tint = self.space_color.map(|c| space_color32(Some(c))).unwrap_or(p.text);
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                ui.horizontal_wrapped(|ui| {
                    for (id, label) in SPACE_ICONS {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), Sense::click());
                        let on = self.space_icon == id;
                        if on || resp.hovered() {
                            ui.painter().rect_filled(rect, 6.0, p.hover);
                        }
                        paint_icon(ui, &space_icon_name(id), egui::Rect::from_center_size(rect.center(), egui::vec2(20.0, 20.0)), if on { tint } else { p.muted });
                        if resp.on_hover_text(label).clicked() {
                            self.space_icon = id.to_string();
                        }
                    }
                });
                ui.add_space(16.0);
                ui.label(RichText::new("Colour").size(13.0).color(p.text));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    let def = ui.add(egui::Button::new(RichText::new("Default").color(p.text)).fill(if self.space_color.is_none() { p.hover } else { Color32::TRANSPARENT }).min_size(egui::vec2(0.0, 28.0)));
                    if def.clicked() {
                        self.space_color = None;
                    }
                    for (i, (name, rgb)) in SPACE_COLORS.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
                        ui.painter().circle_filled(rect.center(), 10.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                        if self.space_color == Some(i) {
                            ui.painter().circle_stroke(rect.center(), 12.5, egui::Stroke::new(1.5_f32, p.text));
                        }
                        if resp.on_hover_text(*name).clicked() {
                            self.space_color = Some(i);
                        }
                    }
                });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    let w = ui.available_width();
                    ui.allocate_ui_with_layout(egui::vec2(w, 32.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ok = !self.space_name.trim().is_empty();
                        if primary_button(ui, if target.is_some() { "Save" } else { "Create space" }, ok).clicked() {
                            save = true;
                        }
                        if plain_button(ui, &p, "Cancel").clicked() {
                            close = true;
                        }
                        if target.is_some() && danger_button(ui, "Delete").clicked() {
                            delete = true;
                        }
                    });
                });
            });
        if save && !self.space_name.trim().is_empty() {
            match target {
                Some(i) => self.edit_space(i),
                None => self.add_space(),
            }
            close = true;
        }
        if delete {
            if let Some(i) = target {
                self.remove_space(i);
            }
            close = true;
        }
        if close {
            self.space_dialog = None;
        }
    }

    fn appearance_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Appearance", "Theme, text and interface.", |_| {});
        settings_group(ui, p, "Theme", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                let w = (ui.available_width() - 16.0) / 3.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for (label, key) in [("Light", 0), ("Dark", 1), ("Auto", 2)] {
                        let on = match key {
                            2 => self.settings.auto_theme,
                            0 => !self.settings.auto_theme && !self.settings.theme.is_dark(),
                            _ => !self.settings.auto_theme && self.settings.theme.is_dark(),
                        };
                        let preview = match key {
                            0 => pal(Theme::Light),
                            1 => pal(Theme::Dark),
                            _ => pal(self.theme_now()),
                        };
                        if theme_tile(ui, p, &preview, label, on, w).clicked() {
                            match key {
                                0 => {
                                    self.settings.auto_theme = false;
                                    self.settings.theme = Theme::Light;
                                }
                                1 => {
                                    self.settings.auto_theme = false;
                                    self.settings.theme = Theme::Dark;
                                }
                                _ => self.settings.auto_theme = true,
                            }
                        }
                    }
                });
                ui.add_space(6.0);
                ui.label(RichText::new("More themes").size(12.0).color(p.muted));
                ui.horizontal_wrapped(|ui| {
                    for t in [Theme::Midnight, Theme::Nord, Theme::Sand] {
                        if ui.selectable_label(self.settings.theme == t && !self.settings.auto_theme, t.label()).clicked() {
                            self.settings.auto_theme = false;
                            self.settings.theme = t;
                        }
                    }
                });
            });
        });
        let mut reset = false;
        settings_group(ui, p, "Text", "", |ui| {
            if ui.button("Reset").clicked() {
                reset = true;
            }
        }, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Font size").size(13.5).color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.0} pt", self.settings.diff_font)).size(12.5).color(p.muted));
                        ui.add(egui::Slider::new(&mut self.settings.diff_font, 10.0..=18.0).show_value(false));
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Interface font").size(13.5).color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("ui_font").width(170.0).selected_text(self.settings.ui_font.clone()).show_ui(ui, |ui| {
                            for (n, path) in UI_FONTS {
                                if path.is_empty() || std::path::Path::new(path).exists() {
                                    ui.selectable_value(&mut self.settings.ui_font, n.to_string(), n);
                                }
                            }
                        });
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Code font").size(13.5).color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("code_font").width(170.0).selected_text(self.settings.code_font.clone()).show_ui(ui, |ui| {
                            for (n, path) in CODE_FONTS {
                                if std::path::Path::new(path).exists() {
                                    ui.selectable_value(&mut self.settings.code_font, n.to_string(), n);
                                }
                            }
                        });
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Interface zoom").size(13.5).color(p.text));
                        ui.label(RichText::new("Scale the whole app.").size(12.0).color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = ui.add(egui::Slider::new(&mut self.settings.zoom, 0.8..=1.5));
                        if r.drag_stopped() || (r.changed() && !r.dragged()) {
                            ctx.set_zoom_factor(self.settings.zoom);
                        }
                    });
                });
            });
        });
        if reset {
            self.settings.diff_font = 12.0;
            self.settings.ui_font = "System font".into();
            self.settings.code_font = "Menlo".into();
            self.settings.zoom = 1.0;
            ctx.set_zoom_factor(1.0);
        }
        settings_group(ui, p, "Grain texture", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Show grain", "Add a subtle noise texture to window chrome.", &mut self.settings.grain);
            });
            for (label, which) in [("Pattern", 0), ("Strength", 1)] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).size(13.5).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let names: [&str; 3] = if which == 0 { ["Fine", "Classic", "Coarse"] } else { ["Subtle", "Balanced", "Strong"] };
                            let cur = if which == 0 { &mut self.settings.grain_pattern } else { &mut self.settings.grain_strength };
                            ui.add_enabled_ui(self.settings.grain, |ui| {
                                egui::ComboBox::from_id_salt(label).width(170.0).selected_text(names[*cur]).show_ui(ui, |ui| {
                                    for (i, n) in names.iter().enumerate() {
                                        ui.selectable_value(cur, i, *n);
                                    }
                                });
                            });
                        });
                    });
                });
            }
        });
        settings_group(ui, p, "Interface", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Pane scrollbars", "Show scrollbars in lists, previews and output panes.", &mut self.settings.scrollbars);
            });
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Smooth transitions", "Animate tab selection and theme colors. Turn off for instant changes.", &mut self.settings.smooth);
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Accent color").size(13.5).color(p.text));
                        ui.label(RichText::new("Used for buttons and selection.").size(12.0).color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (i, (name, color)) in ACCENTS.iter().enumerate().rev() {
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
                            ui.painter().circle_filled(rect.center(), 9.0, *color);
                            if self.settings.accent == i {
                                paint_icon(ui, "check", rect.shrink(5.0), Color32::WHITE);
                            }
                            if resp.on_hover_text(*name).clicked() {
                                self.settings.accent = i;
                            }
                        }
                    });
                });
            });
        });
        settings_group(ui, p, "Commit messages", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Profile pictures", "Look up pictures of commit authors online (GitHub CLI, GitHub, Gravatar). Off shows initials.", &mut self.settings.avatars);
            });
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Commit type badges", "Compact labels for feat, fix, refactor and other conventional prefixes. Original messages are unchanged.", &mut self.settings.commit_badges);
            });
        });
    }

    fn keybinds_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        use crate::keybinds as kb;
        let mut restore = false;
        page_header(ui, p, "Keybinds", "Click a shortcut to change it. Changes apply immediately and are saved.", |ui| {
            if ui.add_enabled(!self.keybinds.is_empty(), egui::Button::new("Restore defaults")).clicked() {
                restore = true;
            }
        });
        ui.add_space(4.0);
        framed(ui, &p, egui::TextEdit::singleline(&mut self.keybind_search).hint_text("Find an action or shortcut...").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
        ui.add_space(14.0);
        // Record a new shortcut for the chosen command.
        if let Some(id) = self.recording {
            let (esc, got) = ui.input(|i| (i.key_pressed(egui::Key::Escape), kb::captured(i)));
            if esc {
                self.recording = None;
                self.keybind_msg.clear();
            } else if let Some(b) = got {
                let has_mod = b.contains('-') && b != "-";
                let taken = kb::COMMANDS.iter().find(|c| c.id != id && kb::bindings(c, &self.keybinds).contains(&b));
                if kb::reserved(&b) {
                    self.keybind_msg = format!("{} is reserved by the system.", kb::format(&b));
                } else if !has_mod && id != "expand-diff" {
                    self.keybind_msg = "Add a modifier key such as \u{2318}, \u{2303} or \u{2325}.".into();
                } else if let Some(c) = taken {
                    self.keybind_msg = format!("{} is already used by {}.", kb::format(&b), c.label);
                } else {
                    let default = kb::COMMANDS.iter().find(|c| c.id == id).map(|c| c.key == b && c.alias.is_none()).unwrap_or(false);
                    if default {
                        self.keybinds.remove(id);
                    } else {
                        self.keybinds.insert(id.to_string(), b);
                    }
                    kb::save(&self.keybinds);
                    self.recording = None;
                    self.keybind_msg.clear();
                }
            }
        }
        let q = self.keybind_search.to_lowercase();
        let mut start: Option<&'static str> = None;
        for group in kb::GROUPS {
            let rows: Vec<&kb::Cmd> = kb::COMMANDS
                .iter()
                .filter(|c| c.group == group)
                .filter(|c| q.is_empty() || format!("{} {} {} {}", c.label, c.group, c.desc, kb::label(c, &self.keybinds)).to_lowercase().contains(&q))
                .collect();
            if rows.is_empty() {
                continue;
            }
            let count = format!("{} shortcut{}", rows.len(), if rows.len() == 1 { "" } else { "s" });
            settings_group(ui, p, group, &count, |_| {}, |ui| {
                for c in rows {
                    group_row(ui, p, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(c.label).size(13.5).color(p.text));
                                if !c.desc.is_empty() {
                                    ui.label(RichText::new(c.desc).size(12.0).color(p.muted));
                                }
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if c.fixed {
                                    key_chip(ui, p, &kb::label(c, &self.keybinds));
                                } else if self.recording == Some(c.id) {
                                    egui::Frame::none().fill(p.hover).stroke(egui::Stroke::new(1.0_f32, accent())).rounding(5.0).inner_margin(egui::Margin::symmetric(10.0, 4.0)).show(ui, |ui| {
                                        ui.label(RichText::new("Press shortcut...").size(12.5).color(p.text));
                                    });
                                } else {
                                    let label = kb::label(c, &self.keybinds);
                                    let r = ui.scope(|ui| key_chip(ui, p, &label)).response.interact(Sense::click());
                                    if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        start = Some(c.id);
                                    }
                                }
                            });
                        });
                    });
                }
            });
            ui.add_space(14.0);
        }
        if !self.keybind_msg.is_empty() {
            ui.label(RichText::new(&self.keybind_msg).size(12.5).color(Color32::from_rgb(235, 110, 100)));
        }
        if let Some(id) = start {
            self.recording = Some(id);
            self.keybind_msg.clear();
        }
        if restore {
            self.keybinds.clear();
            kb::save(&self.keybinds);
            self.recording = None;
        }
    }

    fn git_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Git", "", |_| {});
        settings_group(ui, p, "Committing", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                toggle_row(ui, p, "Automatically include changed files", "Include all changed files in your next commit automatically.", &mut self.settings.auto_include);
            });
        });
        let mut edit = false;
        settings_group(ui, p, "Identity", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 18.0, p.hover);
                    let letter = self.git_name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".into());
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, letter, FontId::proportional(13.0), p.text);
                    ui.vertical(|ui| {
                        let name = if self.git_name.is_empty() { "Name not configured" } else { self.git_name.as_str() };
                        let email = if self.git_email.is_empty() { "Email not configured" } else { self.git_email.as_str() };
                        ui.label(RichText::new(name).size(13.5).color(p.text));
                        ui.label(RichText::new(email).size(12.0).color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(!self.git_editing, egui::Button::new("Edit").min_size(egui::vec2(0.0, 28.0))).clicked() {
                            edit = true;
                        }
                    });
                });
            });
        });
        if edit {
            self.git_editing = true;
            self.git_src = None;
        }
    }

    /// Right-hand panel to change the commit identity: an account or custom values.
    pub fn identity_panel(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.label(RichText::new("Edit commit identity").size(14.0).color(p.text));
        ui.add_space(12.0);
        let n = self.accounts.len() + 1;
        let w = (ui.available_width() - 10.0 * (n.min(2) as f32 - 1.0)) / n.min(2) as f32;
        let mut pick: Option<Option<usize>> = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
            for (i, a) in self.accounts.iter().enumerate() {
                if identity_tile(ui, p, if a.provider == git::Provider::GitHub { "github" } else { "gitlab" }, &a.user, &a.host, self.git_src == Some(i), w).clicked() {
                    pick = Some(Some(i));
                }
            }
            if identity_tile(ui, p, "pencil", "Custom", "Enter name and email", self.git_src.is_none(), w).clicked() {
                pick = Some(None);
            }
        });
        if let Some(src) = pick {
            self.git_src = src;
            if let Some(a) = src.and_then(|i| self.accounts.get(i)) {
                self.git_name = a.user.clone();
                self.git_email = format!("{}@users.noreply.{}", a.user, a.host);
            }
        }
        ui.add_space(8.0);
        if self.git_src.is_none() {
            ui.label(RichText::new("Name").size(13.0).color(p.text));
            framed(ui, &p, egui::TextEdit::singleline(&mut self.git_name).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
            ui.add_space(8.0);
            ui.label(RichText::new("Commit email").size(13.0).color(p.text));
            framed(ui, &p, egui::TextEdit::singleline(&mut self.git_email).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
        } else {
            ui.label(RichText::new(format!("{} <{}>", self.git_name, self.git_email)).size(12.5).color(p.muted));
        }
        if !self.status_msg.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.status_msg).size(12.5).color(Color32::from_rgb(235, 110, 100)));
        }
        let mut save = false;
        let mut cancel = false;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let w = ui.available_width();
            ui.allocate_ui_with_layout(egui::vec2(w, 36.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let ok = !self.git_name.trim().is_empty() && !self.git_email.trim().is_empty();
                if primary_button(ui, "Save identity", ok).clicked() {
                    save = true;
                }
                if plain_button(ui, p, "Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if cancel {
            self.git_editing = false;
            self.status_msg.clear();
            self.git_name = git::config_get("user.name");
            self.git_email = git::config_get("user.email");
        }
        if save {
            let a = git::config_set("user.name", self.git_name.trim());
            let b = git::config_set("user.email", self.git_email.trim());
            match a.and(b) {
                Ok(_) => {
                    self.git_editing = false;
                    self.status_msg.clear();
                }
                Err(e) => self.status_msg = e,
            }
        }
    }

    fn editor_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        use crate::app::editors::{editor_label, installed_editors};
        page_header(ui, p, "External editor", "", |_| {});
        let mut list = installed_editors();
        if !list.iter().any(|(_, c, _)| *c == self.settings.editor) {
            list.push((editor_label(&self.settings.editor), self.settings.editor.clone(), "terminal"));
        }
        let mut choose: Option<String> = None;
        let mut other = false;
        settings_group(ui, p, "Default editor", "", |_| {}, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (label, cmd, ic) in &list {
                if list_row(ui, p, ic, label, self.settings.editor == *cmd, true).clicked() {
                    choose = Some(cmd.clone());
                }
            }
            if list_row(ui, p, "add", "Other application...", false, false).clicked() {
                other = true;
            }
        });
        if other {
            if let Some(app) = rfd::FileDialog::new().set_directory("/Applications").add_filter("Application", &["app"]).pick_file() {
                choose = Some(format!("open -a {}", git::quote(&app.to_string_lossy())));
            }
        }
        if let Some(c) = choose {
            self.settings.editor = c;
        }
        ui.add_space(14.0);
        let mut add = false;
        let mut edit: Option<usize> = None;
        let mut del: Option<usize> = None;
        settings_group(ui, p, "File type rules", "", |ui| {
            if ui.add_enabled(self.rule_dialog.is_none(), egui::Button::new("Add rule")).clicked() {
                add = true;
            }
        }, |ui| {
            if self.editor_rules.is_empty() {
                group_row(ui, p, |ui| {
                    ui.label(RichText::new("All files use the default editor.").size(13.0).color(p.muted));
                });
            }
            for (i, r) in self.editor_rules.iter().enumerate() {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&r.patterns).size(13.5).color(p.text));
                            ui.label(RichText::new(crate::app::editors::editor_label(&r.editor)).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                del = Some(i);
                            }
                            if ui.button("Edit...").clicked() {
                                edit = Some(i);
                            }
                        });
                    });
                });
            }
        });
        if add {
            self.rule_patterns.clear();
            self.rule_editor = "open".into();
            self.rule_dialog = Some(None);
        }
        if let Some(i) = edit {
            self.rule_patterns = self.editor_rules[i].patterns.clone();
            self.rule_editor = self.editor_rules[i].editor.clone();
            self.rule_dialog = Some(Some(i));
        }
        if let Some(i) = del {
            self.editor_rules.remove(i);
            self.save_rules();
        }
    }

    /// Right-hand panel to add or edit a file type rule.
    pub fn rule_panel(&mut self, ui: &mut egui::Ui, p: &Pal, target: Option<usize>) {
        use crate::app::editors::{editor_label, installed_editors};
        ui.label(RichText::new(if target.is_some() { "Edit file type default" } else { "Add file type default" }).size(14.0).color(p.text));
        ui.add_space(14.0);
        ui.label(RichText::new("File types or names").size(13.0).color(p.text));
        framed(ui, &p, egui::TextEdit::singleline(&mut self.rule_patterns).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
        ui.add_space(4.0);
        ui.label(RichText::new(".ts, .tsx, Dockerfile").size(12.5).color(p.muted));
        ui.add_space(12.0);
        let mut list = installed_editors();
        if !list.iter().any(|(_, c, _)| *c == self.rule_editor) {
            list.push((editor_label(&self.rule_editor), self.rule_editor.clone(), "terminal"));
        }
        let mut choose = false;
        settings_group(ui, p, "Application", "", |ui| {
            if ui.button("Choose application...").clicked() {
                choose = true;
            }
        }, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (label, cmd, ic) in &list {
                if list_row(ui, p, ic, label, self.rule_editor == *cmd, true).clicked() {
                    self.rule_editor = cmd.clone();
                }
            }
        });
        if choose {
            if let Some(app) = rfd::FileDialog::new().set_directory("/Applications").add_filter("Application", &["app"]).pick_file() {
                self.rule_editor = format!("open -a {}", git::quote(&app.to_string_lossy()));
            }
        }
        let mut save = false;
        let mut cancel = false;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let w = ui.available_width();
            ui.allocate_ui_with_layout(egui::vec2(w, 36.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_button(ui, "Save", !self.rule_patterns.trim().is_empty()).clicked() {
                    save = true;
                }
                if plain_button(ui, p, "Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if save {
            let rule = crate::app::editors::EditorRule { patterns: self.rule_patterns.trim().to_string(), editor: self.rule_editor.clone() };
            match target {
                Some(i) if i < self.editor_rules.len() => self.editor_rules[i] = rule,
                _ => self.editor_rules.push(rule),
            }
            self.save_rules();
            self.rule_dialog = None;
        }
        if cancel {
            self.rule_dialog = None;
        }
    }

    fn ignore_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Ignored files", "The .gitignore of this repository.", |_| {});
        let Some(repo) = self.repo() else {
            ui.label(RichText::new("Open a repository first.").color(p.muted));
            return;
        };
        settings_group(ui, p, ".gitignore", &repo_name(&repo), |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.ignore_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(14)
                        .desired_width(f32::INFINITY),
                );
                if ui.button("Save .gitignore").clicked() {
                    let r = git::write_ignore(&repo, &self.ignore_text);
                    self.report(r);
                    self.refresh();
                }
            });
        });
    }

    fn accounts_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut add = false;
        page_header(ui, p, "Accounts", "Connected hosting accounts.", |ui| {
            if ui.button("Add account").clicked() {
                add = true;
            }
        });
        if add {
            self.acct_adding = true;
        }
        let mut out: Option<usize> = None;
        let mut edit: Option<usize> = None;
        let mut check: Option<usize> = None;
        let count = format!("{} account{}", self.accounts.len(), if self.accounts.len() == 1 { "" } else { "s" });
        settings_group(ui, p, "Connected accounts", &count, |_| {}, |ui| {
            if self.accounts.is_empty() {
                group_row(ui, p, |ui| {
                    ui.label(RichText::new("No accounts connected").color(p.muted));
                });
            }
            for (i, a) in self.accounts.iter().enumerate() {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        icon(ui, if a.provider == git::Provider::GitHub { "github" } else { "gitlab" }, 18.0, p.text);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&a.user).color(p.text));
                            let via = if a.cli { "GitHub CLI" } else { "Access token" };
                            let st = self.acct_status.get(&a.secret_name()).map(String::as_str).unwrap_or("Not checked");
                            ui.label(RichText::new(format!("{} \u{b7} {via} \u{b7} {st}", a.host)).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
                            paint_icon(ui, "remove", rect.shrink(4.0), Color32::from_rgb(220, 90, 90));
                            if resp.on_hover_text("Remove account").clicked() {
                                out = Some(i);
                            }
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
                            paint_icon(ui, "pencil", rect.shrink(4.0), p.muted);
                            if resp.on_hover_text("Reconnect").clicked() {
                                edit = Some(i);
                            }
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
                            paint_icon(ui, "sync", rect.shrink(4.0), p.muted);
                            if resp.on_hover_text("Check connection").clicked() {
                                check = Some(i);
                            }
                        });
                    });
                });
            }
        });
        if let Some(i) = out {
            self.sign_out(i);
        }
        if let Some(i) = edit {
            if let Some(a) = self.accounts.get(i) {
                self.acct_provider = a.provider;
                self.acct_host = if a.host == a.provider.default_host() { String::new() } else { a.host.clone() };
                self.acct_method = if a.cli { 2 } else { 1 };
            }
            self.acct_adding = true;
        }
        if let Some(i) = check {
            if let Some(a) = self.accounts.get(i).cloned() {
                self.acct_status.insert(a.secret_name(), "Checking...".into());
                let ctx = ui.ctx().clone();
                let label = format!("{T_CHECK}{}", a.secret_name());
                self.spawn_task(&ctx, &label, move || {
                    let token = a.token().ok_or("Token missing")?;
                    git::whoami(a.provider, &a.host, &token).map(|_| "Connected".to_string())
                });
            }
        }
    }

    fn account_add_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        let gh = self.acct_provider == git::Provider::GitHub;
        if !gh && self.acct_method == 2 {
            self.acct_method = 0;
        }
        let provider_label = self.acct_provider.label();
        ui.label(RichText::new("Add account").size(14.0).color(p.text));
        ui.label(RichText::new("Git credentials stay separate.").size(12.5).color(p.muted));
        ui.add_space(8.0);
        let footer_h = 58.0;
        let busy = self.task_running(T_SIGN_IN) || self.task_running(T_DEVICE);
        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - footer_h).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            let tile_w = (ui.available_width() - 10.0) / 2.0;
            ui.label(RichText::new("Provider").size(13.0).color(p.text));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                for (icon_name, label, which) in [("github", "GitHub", 0), ("gitlab", "GitLab", 1)] {
                    let on = if which == 0 { gh } else { !gh };
                    if choice_tile(ui, p, icon_name, label, on, tile_w).clicked() {
                        self.acct_provider = if which == 1 { git::Provider::GitLab } else { git::Provider::GitHub };
                        self.acct_advanced = false;
                        self.acct_host.clear();
                        self.device = None;
                    }
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Connect with").size(13.0).color(p.text));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let n = if gh { 3.0 } else { 2.0 };
                let w = (ui.available_width() - 10.0 * (n - 1.0)) / n;
                if choice_tile(ui, p, "browser", "Browser", self.acct_method == 0, w).clicked() {
                    self.acct_method = 0;
                }
                if choice_tile(ui, p, "key", "Access token", self.acct_method == 1, w).clicked() {
                    self.acct_method = 1;
                }
                if gh && choice_tile(ui, p, "github", "GitHub CLI", self.acct_method == 2, w).clicked() {
                    self.acct_method = 2;
                }
            });
            ui.add_space(8.0);
            let settings_url = if gh { "https://github.com/settings/developers" } else { "https://gitlab.com/-/user_settings/applications" };
            let token_url = if gh { "https://github.com/settings/personal-access-tokens/new" } else { "https://gitlab.com/-/user_settings/personal_access_tokens" };
            match self.acct_method {
                0 => {
                    if let Some(d) = self.device.clone() {
                        ui.label(RichText::new("Approve in your browser").size(13.0).color(p.text));
                        ui.label(RichText::new("Enter this code on the page that just opened.").size(12.0).color(p.muted));
                        ui.add_space(6.0);
                        egui::Frame::none().fill(p.hover).rounding(8.0).inner_margin(egui::Margin::same(14.0)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(&d.user_code).size(24.0).monospace().color(p.text));
                            ui.label(RichText::new(&d.uri).size(12.0).color(p.muted));
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Copy code").clicked() {
                                ui.ctx().copy_text(d.user_code.clone());
                            }
                            if ui.button("Open page").clicked() {
                                open_url(&d.uri);
                            }
                            ui.spinner();
                            ui.label(RichText::new("Waiting for approval...").size(12.0).color(p.muted));
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Application setup").size(13.0).color(p.text));
                                ui.label(RichText::new("Register an OAuth app using these values.").size(12.0).color(p.muted));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(format!("Open {provider_label} settings")).clicked() {
                                    open_url(settings_url);
                                }
                            });
                        });
                        copy_field(ui, p, "Application name", "OnayGit");
                        copy_field(ui, p, "Homepage URL", "https://github.com/Tisobyn/OnayGit");
                        copy_field(ui, p, "Callback URL", "http://127.0.0.1:8765/oauth/callback");
                        ui.label(RichText::new(if gh { "Enable Device Flow, then register the application." } else { "Enable the device flow scope, then save the application." }).size(12.0).color(p.muted));
                        ui.add_space(6.0);
                        ui.label(RichText::new("Client ID").size(13.0).color(p.text));
                        framed(ui, &p, egui::TextEdit::singleline(&mut self.acct_client_id).hint_text("Client ID").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                        ui.label(RichText::new("Use the public ID, not the client secret.").size(12.0).color(p.muted));
                    }
                }
                1 => {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Token setup").size(13.0).color(p.text));
                            ui.label(RichText::new("Create a token using the details below.").size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(format!("Open {provider_label} settings")).clicked() {
                                open_url(token_url);
                            }
                        });
                    });
                    copy_field(ui, p, "Token name", "OnayGit");
                    ui.label(
                        RichText::new(if gh {
                            "Use a fine-grained token with an expiry date. Keep repository access public-only; no extra permissions are needed for account connection."
                        } else {
                            "Choose an expiry date and select only read_user, then create the token."
                        })
                        .size(12.0)
                        .color(p.muted),
                    );
                    ui.add_space(6.0);
                    ui.label(RichText::new("Access token").size(13.0).color(p.text));
                    framed(ui, &p, egui::TextEdit::singleline(&mut self.acct_token).password(true).hint_text("Paste an access token").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
                    ui.label(RichText::new("Stored securely in macOS Keychain.").size(12.0).color(p.muted));
                }
                _ => {
                    ui.label(RichText::new("Uses the token from gh auth login.").size(12.5).color(p.muted));
                }
            }
            if self.acct_method != 2 && self.device.is_none() {
                ui.add_space(8.0);
                ui.label(RichText::new("Repository access").size(13.0).color(p.text));
                ui.checkbox(&mut self.acct_private, "Browse and clone private repositories");
                ui.label(RichText::new("Profile-only sign-in. GitHub can list public repositories; GitLab repository browsing needs read_api.").size(12.0).color(p.muted));
            }
            ui.add_space(4.0);
            let open = self.acct_advanced || !self.acct_host.is_empty();
            egui::CollapsingHeader::new("Advanced").default_open(open).show(ui, |ui| {
                ui.label(RichText::new("Host").size(12.5).color(p.text));
                framed(ui, &p, egui::TextEdit::singleline(&mut self.acct_host).hint_text(self.acct_provider.default_host()).desired_width(f32::INFINITY));
                ui.label(RichText::new("Change this for GitHub Enterprise or a self-hosted GitLab.").size(12.0).color(p.muted));
            });
            if !self.status_msg.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new(&self.status_msg).size(12.5).color(Color32::from_rgb(235, 110, 100)));
            }
        });
        let mut cancel = false;
        let mut go = false;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let w = ui.available_width();
            ui.allocate_ui_with_layout(egui::vec2(w, 36.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, ready) = match self.acct_method {
                    0 => ("Sign in with browser", !self.acct_client_id.trim().is_empty() && self.device.is_none()),
                    1 => ("Connect", !self.acct_token.trim().is_empty()),
                    _ => ("Connect", true),
                };
                if primary_button(ui, label, ready && !busy).clicked() {
                    go = true;
                }
                if plain_button(ui, p, "Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if go {
            self.status_msg.clear();
            if self.acct_method == 0 {
                self.browser_sign_in(ctx);
            } else {
                self.acct_cli = self.acct_method == 2;
                self.sign_in(ctx);
            }
        }
        if self.device.is_some() && !self.device_waiting {
            self.wait_for_browser(ctx);
        }
        if cancel {
            self.acct_adding = false;
            self.acct_token.clear();
            self.device = None;
            self.device_waiting = false;
            self.status_msg.clear();
        }
    }

    fn about_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let version = env!("CARGO_PKG_VERSION");
        let hash = env!("ONAYGIT_GIT_HASH");
        let date = env!("ONAYGIT_BUILD_DATE");
        let git_v = git::run(&home_dir(), &["--version"]).unwrap_or_default().trim().trim_start_matches("git version ").to_string();
        let platform = format!("{} {}", if std::env::consts::OS == "macos" { "macOS" } else { std::env::consts::OS }, std::env::consts::ARCH);
        let info = format!("OnayGit {version} ({hash}, built {date})\nPlatform: {platform}\nGit: {git_v}");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add(egui::Image::from_bytes("bytes://about-icon.png", include_bytes!("../../assets/icon-512.png").as_slice()).fit_to_exact_size(egui::vec2(72.0, 72.0)).rounding(16.0));
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("OnayGit").size(24.0).strong().color(p.text));
                ui.label(RichText::new(format!("Version {version}")).size(13.5).color(p.muted));
                ui.label(RichText::new("A simple Git client for macOS, written in Rust.").size(12.5).color(p.muted));
            });
        });
        ui.add_space(16.0);
        let mut url: Option<&str> = None;
        let mut copy = false;
        settings_group(ui, p, "Details", "", |_| {}, |ui| {
            for (k, v) in [
                ("Version", version.to_string()),
                ("Build", format!("{hash} \u{b7} {date}")),
                ("Platform", platform.clone()),
                ("Git", if git_v.is_empty() { "not found".into() } else { git_v.clone() }),
                ("Author", env!("CARGO_PKG_AUTHORS").to_string()),
                ("Settings files", "~/.onaygit_*".to_string()),
            ] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(k).size(13.5).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(v).size(13.0).color(p.muted));
                        });
                    });
                });
            }
        });
        settings_group(ui, p, "Links", "", |_| {}, |ui| {
            for (title, button, link) in [
                ("Source code", "Open GitHub", "https://github.com/Tisobyn/OnayGit"),
                ("Report a problem", "Create an issue", "https://github.com/Tisobyn/OnayGit/issues/new"),
                ("Releases", "Open releases", "https://github.com/Tisobyn/OnayGit/releases"),
            ] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(title).size(13.5).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(button).clicked() {
                                url = Some(link);
                            }
                        });
                    });
                });
            }
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Version details").size(13.5).color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Copy").clicked() {
                            copy = true;
                        }
                    });
                });
            });
        });
        settings_group(ui, p, "Credits", "", |_| {}, |ui| {
            for (k, v) in [
                ("Interface", "egui and eframe"),
                ("Icons", "Phosphor Icons (MIT)"),
                ("Design", "Inspired by 1git"),
                ("Git engine", "Your own git, run as a command"),
            ] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(k).size(13.5).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(v).size(13.0).color(p.muted));
                        });
                    });
                });
            }
        });
        ui.label(RichText::new("OnayGit has no telemetry and sends nothing to servers of its own.").size(12.0).color(p.muted));
        if copy {
            ui.ctx().copy_text(info);
            self.status_msg = "Copied".into();
        }
        if let Some(u) = url {
            ui.ctx().open_url(egui::OpenUrl::new_tab(u));
        }
    }

    fn help_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Help", "Guides, setup, and support for OnayGit.", |_| {});
        let mut url: Option<&str> = None;
        settings_group(ui, p, "Getting started", "", |_| {}, |ui| {
            for (title, desc, button) in [
                ("Guided tips", "Rediscover shortcuts and workspace features.", "Show again"),
                ("First-run setup", "", "Run setup again"),
            ] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).size(13.5).color(p.text));
                            if !desc.is_empty() {
                                ui.label(RichText::new(desc).size(12.0).color(p.muted));
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(button).clicked() {
                                if title == "Guided tips" {
                                    self.settings.tips = true;
                                    self.settings.tour = 0;
                                    self.settings.tour_skipped = false;
                                } else {
                                    self.settings.onboarded = false;
                                }
                            }
                        });
                    });
                });
            }
        });
        settings_group(ui, p, "Support", "", |_| {}, |ui| {
            for (title, desc, button, link) in [
                ("Create an issue", "Report a bug or request a feature on GitHub.", "Create an issue", "https://github.com/Tisobyn/OnayGit/issues/new"),
                ("Existing issues", "Check reported problems and planned improvements.", "Browse issues", "https://github.com/Tisobyn/OnayGit/issues"),
                ("Project on GitHub", "Read about OnayGit, explore the source, or contribute.", "Open GitHub", "https://github.com/Tisobyn/OnayGit"),
            ] {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).size(13.5).color(p.text));
                            ui.label(RichText::new(desc).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(button).clicked() {
                                url = Some(link);
                            }
                        });
                    });
                });
            }
        });
        if let Some(u) = url {
            ui.ctx().open_url(egui::OpenUrl::new_tab(u));
        }
    }

}

fn privacy_page(ui: &mut egui::Ui, p: &Pal) {
    page_header(ui, p, "Privacy", "", |_| {});
    settings_group(ui, p, "Usage statistics", "", |_| {}, |ui| {
        group_row(ui, p, |ui| {
            ui.label(RichText::new("No usage statistics").size(13.5).color(p.text));
            ui.label(RichText::new("OnayGit has no telemetry. It sends nothing to servers of its own.").size(12.0).color(p.muted));
        });
    });
    settings_group(ui, p, "Profile pictures", "", |_| {}, |ui| {
        group_row(ui, p, |ui| {
            ui.label(RichText::new("Commit author pictures are looked up with the GitHub CLI, GitHub, or Gravatar. Author emails are used for this lookup. Turn it off in Settings > Appearance.").size(12.0).color(p.muted));
        });
    });
    settings_group(ui, p, "What's sent", "", |_| {}, |ui| {
        group_row(ui, p, |ui| {
            ui.label(RichText::new("Sent").size(12.0).color(p.muted));
        });
        group_row(ui, p, |ui| {
            ui.label(RichText::new("Nothing").size(13.5).color(p.text));
        });
        group_row(ui, p, |ui| {
            ui.label(RichText::new("Never sent").size(12.0).color(p.muted));
        });
        for t in ["Repository names", "Paths", "Code", "Commit messages", "Account details"] {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    icon(ui, "shield", 14.0, p.muted);
                    ui.label(RichText::new(t).size(13.5).color(p.text));
                });
            });
        }
    });
    ui.label(
        RichText::new("Git commands run on this Mac, or on your machines over SSH. Tokens and API keys stay in the macOS Keychain. Network calls happen only when you use GitHub, GitLab or an AI provider.")
            .size(12.5)
            .color(p.muted),
    );
}

/// Link or remove the `onaygit` command. The system folder asks macOS for admin approval.
fn toggle_launcher(bin: &Path, installed: bool, system: bool) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if system {
        let cmd = if installed {
            format!("rm -f '{}'", bin.display())
        } else {
            format!("mkdir -p /usr/local/bin && ln -sf '{}' '{}'", exe.display(), bin.display())
        };
        let script = format!("do shell script \"{}\" with administrator privileges", cmd.replace('\\', "\\\\").replace('"', "\\\""));
        let out = std::process::Command::new("osascript").arg("-e").arg(script).output().map_err(|e| e.to_string())?;
        return if out.status.success() {
            Ok(if installed { "Removed".into() } else { format!("Installed at {}", bin.display()) })
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        };
    }
    if installed {
        std::fs::remove_file(bin).map_err(|e| e.to_string())?;
        return Ok("Removed".into());
    }
    if let Some(dir) = bin.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::os::unix::fs::symlink(exe, bin).map_err(|e| e.to_string())?;
    Ok(format!("Installed at {}", bin.display()))
}

fn toggle_row(ui: &mut egui::Ui, p: &Pal, label: &str, desc: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).size(13.5).color(p.text));
            ui.label(RichText::new(desc).size(12.0).color(p.muted));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(value, "");
        });
    });
}

/// Mini window drawing used as a theme choice, like 1git's theme previews.
fn theme_tile(ui: &mut egui::Ui, p: &Pal, c: &Pal, label: &str, selected: bool, w: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 124.0), Sense::click());
    let prev = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 8.0), egui::vec2(w - 16.0, 76.0));
    let pt = ui.painter();
    pt.rect_filled(prev, 4.0, c.input);
    pt.rect_stroke(prev, 4.0, egui::Stroke::new(1.0_f32, c.border));
    pt.rect_filled(egui::Rect::from_min_size(prev.min + egui::vec2(6.0, 6.0), egui::vec2(prev.width() * 0.5, 4.0)), 2.0, c.muted);
    let body = egui::Rect::from_min_max(prev.min + egui::vec2(6.0, 14.0), prev.max - egui::vec2(6.0, 6.0));
    let side = egui::Rect::from_min_size(body.min, egui::vec2(body.width() * 0.24, body.height()));
    pt.rect_filled(side, 2.0, c.hover);
    let main = egui::Rect::from_min_max(egui::pos2(side.max.x + 4.0, body.min.y), body.max);
    pt.rect_filled(main, 2.0, c.bg);
    for (i, f) in [0.6, 0.85, 0.45].iter().enumerate() {
        let y = main.min.y + 8.0 + i as f32 * 7.0;
        pt.rect_filled(egui::Rect::from_min_size(egui::pos2(main.min.x + 8.0, y), egui::vec2((main.width() - 16.0) * f, 3.0)), 2.0, c.muted);
    }
    let col = if selected { accent() } else { p.text };
    pt.text(egui::pos2(rect.center().x, rect.max.y - 16.0), Align2::CENTER_CENTER, label, FontId::proportional(13.0), col);
    if selected {
        pt.rect_stroke(rect.shrink(1.0), 8.0, egui::Stroke::new(1.0_f32, accent()));
    }
    resp
}

/// Live sample of workspace, history and diff, so theme changes are visible at once.
fn appearance_preview(ui: &mut egui::Ui, p: &Pal) {
    ui.label(RichText::new("Preview").size(14.0).color(p.text));
    ui.add_space(12.0);
    egui::Frame::none().stroke(egui::Stroke::new(1.0_f32, p.border)).rounding(6.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Frame::none().fill(p.hover).inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Workspace").size(13.5).color(p.text));
            ui.label(RichText::new("Sample commits and code").size(12.0).color(p.muted));
        });
        egui::Frame::none().inner_margin(egui::Margin::same(12.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("History").size(13.5).color(p.text));
            ui.add_space(4.0);
            for (tag, msg) in [("feat", "(core) add workspace shortcuts"), ("refactor", "(db) simplify queries"), ("", "Update the getting started guide")] {
                ui.horizontal(|ui| {
                    if !tag.is_empty() {
                        let c = crate::ui::history::badge_color(tag);
                        ui.label(RichText::new(tag).size(11.0).color(p.text).background_color(c.gamma_multiply(0.18)));
                    }
                    ui.label(RichText::new(msg).size(12.5).color(p.text));
                });
            }
        });
        ui.separator();
        let mono = |t: &str, c: Color32| RichText::new(t).monospace().size(11.0).color(c);
        egui::Frame::none().inner_margin(egui::Margin::symmetric(12.0, 8.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(mono("src/greeting.ts", p.muted));
            ui.add_space(6.0);
            ui.label(mono(" export function greet(name: string) {", p.text));
        });
        for (bg, line) in [(p.del_bg, "- return \"Hello\";"), (p.add_bg, "+ return `Hello, ${name}!`;")] {
            egui::Frame::none().fill(bg).inner_margin(egui::Margin::symmetric(12.0, 4.0)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(mono(line, p.text));
            });
        }
        egui::Frame::none().inner_margin(egui::Margin::symmetric(12.0, 6.0)).show(ui, |ui| {
            ui.label(mono("  }", p.text));
        });
    });
    ui.add_space(8.0);
    ui.label(RichText::new("Updates as you adjust your appearance settings.").size(12.0).color(p.muted));
}

/// Colour of a space, or the muted default when none is chosen.
pub fn space_color32(c: Option<usize>) -> Color32 {
    match c.and_then(|i| SPACE_COLORS.get(i)) {
        Some((_, rgb)) => Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
        None => Color32::from_rgb(160, 165, 175),
    }
}

/// Read-only value with a Copy button, for setup values the user pastes elsewhere.
fn copy_field(ui: &mut egui::Ui, p: &Pal, label: &str, value: &str) {
    ui.add_space(2.0);
    ui.label(RichText::new(label).size(13.0).color(p.text));
    egui::Frame::none().stroke(egui::Stroke::new(1.0_f32, p.border)).rounding(6.0).inner_margin(egui::Margin::symmetric(10.0, 8.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(value).size(13.0).color(p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("Copy").size(12.5).color(p.text)).frame(false)).clicked() {
                    ui.ctx().copy_text(value.to_string());
                }
            });
        });
    });
}

/// Tile with icon, title and a detail line, as used for identity choices.
fn identity_tile(ui: &mut egui::Ui, p: &Pal, icon_name: &str, title: &str, detail: &str, selected: bool, w: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 100.0), Sense::click());
    ui.painter().rect_filled(rect, 8.0, if selected { p.hover } else { Color32::TRANSPARENT });
    ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0_f32, if selected { accent() } else { p.border }));
    paint_icon(ui, icon_name, egui::Rect::from_center_size(rect.center() - egui::vec2(0.0, 26.0), egui::vec2(20.0, 20.0)), if selected { accent() } else { p.muted });
    ui.painter().text(rect.center() + egui::vec2(0.0, 6.0), Align2::CENTER_CENTER, title, FontId::proportional(13.5), p.text);
    ui.painter().text(rect.center() + egui::vec2(0.0, 28.0), Align2::CENTER_CENTER, detail, FontId::proportional(12.0), p.muted);
    resp
}
