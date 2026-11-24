#[allow(unused_imports)]
use crate::prelude::*;
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
                    (Section::Privacy, "shield", "Privacy"),
                    (Section::Accounts, "userCircle", "Accounts"),
                    (Section::Git, "branch", "Git"),
                    (Section::Editor, "pencil", "External editor"),
                    (Section::Ai, "sparkle", "AI"),
                    (Section::Help, "help", "Help"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                    }
                }
                ui.add_space(14.0);
                rail_label(ui, "Repository", &p);
                for (s, icon, name) in [
                    (Section::Remotes, "network", "Remotes"),
                    (Section::RepoStyle, "palette", "Appearance"),
                    (Section::Identity, "idCard", "Commit identity"),
                    (Section::Ignore, "stop", "Ignored files"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                        if s == Section::Ignore {
                            self.ignore_text = self.repo().map(|r| git::read_ignore(&r)).unwrap_or_default();
                        }
                        if s == Section::Identity {
                            self.load_repo_identity();
                        }
                    }
                }
            });

        let adding = self.section == Section::Accounts && self.acct_adding;
        if adding {
            egui::SidePanel::right("account_add")
                .exact_width(330.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(18.0)))
                .show(ctx, |ui| self.account_add_panel(ctx, ui, &p));
        }
        self.space_dialog_ui(ctx);
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
                        Section::Privacy | Section::Help => 560.0,
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
                        Section::RepoStyle => self.repo_style_page(ui, &p),
                        Section::Identity => self.identity_page(ui, &p),
                        Section::Help => self.help_page(ui, &p),
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
            self.selected_space = Some(i);
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
                            ui.painter().circle_stroke(rect.center(), 12.5, egui::Stroke::new(1.5, p.text));
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
        settings_group(ui, p, "Text", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Font size").size(13.5).color(p.text));
                        ui.label(RichText::new("Text size in the diff view.").size(12.0).color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.diff_font, 10.0..=18.0));
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
        ui.add(egui::TextEdit::singleline(&mut self.keybind_search).hint_text("Find an action or shortcut...").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 8.0)));
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
                                    ui.label(RichText::new(kb::label(c, &self.keybinds)).size(12.5).color(p.muted));
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
        let mut save = false;
        let mut cancel = false;
        let mut edit = false;
        settings_group(ui, p, "Identity", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                if self.git_editing {
                    field_label(ui, p, "Name");
                    ui.add(egui::TextEdit::singleline(&mut self.git_name).desired_width(f32::INFINITY));
                    field_label(ui, p, "Email");
                    ui.add(egui::TextEdit::singleline(&mut self.git_email).desired_width(f32::INFINITY));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 16.0, p.hover);
                        let letter = self.git_name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".into());
                        ui.painter().text(r.center(), Align2::CENTER_CENTER, letter, FontId::proportional(13.0), p.text);
                        ui.vertical(|ui| {
                            let name = if self.git_name.is_empty() { "Not set" } else { self.git_name.as_str() };
                            ui.label(RichText::new(name).size(13.5).color(p.text));
                            ui.label(RichText::new(&self.git_email).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Edit").clicked() {
                                edit = true;
                            }
                        });
                    });
                }
            });
        });
        if edit {
            self.git_editing = true;
        }
        if cancel {
            self.git_editing = false;
        }
        if save {
            let a = git::config_set("user.name", self.git_name.trim());
            let b = git::config_set("user.email", self.git_email.trim());
            match a.and(b) {
                Ok(_) => self.git_editing = false,
                Err(e) => self.status_msg = e,
            }
        }
    }

    fn editor_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "External editor", "", |_| {});
        let known = [("System default", "open", "pencil"), ("VS Code", "code", "terminal"), ("Cursor", "cursor", "terminal"), ("Zed", "zed", "terminal"), ("Sublime Text", "subl", "terminal"), ("Xcode", "open -a Xcode", "terminal")];
        let mut choose: Option<String> = None;
        let mut other = false;
        settings_group(ui, p, "Default editor", "", |_| {}, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (label, cmd, ic) in known {
                if list_row(ui, p, ic, label, self.settings.editor == cmd, true).clicked() {
                    choose = Some(cmd.to_string());
                }
            }
            if !known.iter().any(|(_, c, _)| *c == self.settings.editor) {
                let label = self.settings.editor.trim_start_matches("open -a ").trim_matches('\'').rsplit('/').next().unwrap_or("").trim_end_matches(".app").to_string();
                list_row(ui, p, "pencil", &label, true, true);
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

    fn load_repo_identity(&mut self) {
        let Some(repo) = self.repo() else { return };
        let get = |k: &str| git::run(&repo, &["config", "--local", k]).map(|s| s.trim().to_string()).unwrap_or_default();
        self.repo_name_cfg = get("user.name");
        self.repo_email_cfg = get("user.email");
    }

    fn identity_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Commit identity", "Override your name and email for this repository only.", |_| {});
        let Some(repo) = self.repo() else {
            ui.label(RichText::new("Open a repository first.").color(p.muted));
            return;
        };
        settings_group(ui, p, "Identity", &repo_name(&repo), |_| {}, |ui| {
            group_row(ui, p, |ui| {
                field_label(ui, p, "Name");
                ui.add(egui::TextEdit::singleline(&mut self.repo_name_cfg).hint_text(&self.git_name).desired_width(f32::INFINITY));
                field_label(ui, p, "Email");
                ui.add(egui::TextEdit::singleline(&mut self.repo_email_cfg).hint_text(&self.git_email).desired_width(f32::INFINITY));
                ui.add_space(6.0);
                if ui.button("Save").clicked() {
                    for (k, v) in [("user.name", self.repo_name_cfg.trim()), ("user.email", self.repo_email_cfg.trim())] {
                        let args: Vec<&str> = if v.is_empty() { vec!["config", "--local", "--unset", k] } else { vec!["config", "--local", k, v] };
                        let _ = git::run(&repo, &args);
                    }
                    self.status_msg = "Saved".into();
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
                            ui.label(RichText::new(format!("{} \u{b7} Connected", a.host)).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
                            paint_icon(ui, "remove", rect.shrink(3.0), Color32::from_rgb(220, 90, 90));
                            if resp.on_hover_text("Sign out").clicked() {
                                out = Some(i);
                            }
                        });
                    });
                });
            }
        });
        if let Some(i) = out {
            self.sign_out(i);
        }
    }

    fn account_add_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        ui.label(RichText::new("Add account").size(16.0).color(p.text));
        ui.label(RichText::new("Tokens are kept in the macOS Keychain.").size(12.5).color(p.muted));
        ui.add_space(12.0);
        field_label(ui, p, "Provider");
        ui.horizontal(|ui| {
            for pr in [git::Provider::GitHub, git::Provider::GitLab] {
                let icon = if pr == git::Provider::GitHub { "github" } else { "gitlab" };
                if choice_tile(ui, p, icon, pr.label(), self.acct_provider == pr, 140.0).clicked() {
                    self.acct_provider = pr;
                    self.acct_host.clear();
                    if pr != git::Provider::GitHub {
                        self.acct_cli = false;
                    }
                }
            }
        });
        ui.add_space(8.0);
        if self.acct_provider == git::Provider::GitHub {
            field_label(ui, p, "Connect with");
            ui.horizontal(|ui| {
                if choice_tile(ui, p, "key", "Access token", !self.acct_cli, 140.0).clicked() {
                    self.acct_cli = false;
                }
                if choice_tile(ui, p, "terminal", "GitHub CLI", self.acct_cli, 140.0).clicked() {
                    self.acct_cli = true;
                }
            });
            ui.add_space(8.0);
        }
        field_label(ui, p, "Host");
        ui.add(egui::TextEdit::singleline(&mut self.acct_host).hint_text(self.acct_provider.default_host()).desired_width(f32::INFINITY));
        if !self.acct_cli {
            field_label(ui, p, "Access token");
            ui.add(egui::TextEdit::singleline(&mut self.acct_token).password(true).desired_width(f32::INFINITY));
        } else {
            ui.add_space(6.0);
            ui.label(RichText::new("Uses the token from gh auth login.").size(12.0).color(p.muted));
        }
        if !self.status_msg.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.status_msg).size(12.0).color(p.muted));
        }
        ui.add_space(12.0);
        let busy = self.task_running(T_SIGN_IN);
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.acct_adding = false;
                self.acct_token.clear();
            }
            let ready = self.acct_cli || !self.acct_token.trim().is_empty();
            if ui.add_enabled(!busy && ready, egui::Button::new("Connect")).clicked() {
                self.sign_in(ctx);
            }
            if busy {
                ui.spinner();
            }
        });
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
                        ui.label(RichText::new(tag).size(11.0).color(p.muted).background_color(p.hover));
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
