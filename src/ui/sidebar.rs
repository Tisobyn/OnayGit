use crate::prelude::*;

impl App {
    /// Subject (first line) and body (the rest) of the commit message box.
    pub fn commit_parts(&self) -> (String, String) {
        let m = self.summary.trim();
        match m.split_once('\n') {
            Some((s, b)) => (s.trim().to_string(), b.trim().to_string()),
            None => (m.to_string(), String::new()),
        }
    }

    pub fn set_message(&mut self, subject: &str, body: &str) {
        self.summary = if body.trim().is_empty() { subject.to_string() } else { format!("{subject}\n\n{}", body.trim()) };
    }

    pub fn sidebar(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut actions: Vec<Action> = vec![];
        let mut do_commit = false;
        let mut do_undo = false;
        let mut do_stash = false;
        let mut do_ai = false;
        let mut toggle_dir: Option<String> = None;
        let mut ignore: Option<String> = None;
        let mut ask: Option<Confirm> = None;

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(300.0)
            .width_range(240.0..=520.0)
            .frame(egui::Frame::none().fill(p.sidebar))
            .show(ctx, |ui| {
                // Header
                let mut do_refresh = false;
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    let title = match self.tab {
                        Tab::Changes => "Changes",
                        Tab::History => "History",
                        Tab::Ignored => "Ignored changes",
                        Tab::Graph => "Branches and tags",
                    };
                    ui.label(RichText::new(title).size(14.0).color(p.text));
                    if self.tab == Tab::History {
                        let n = if self.compare.is_some() { self.compare_commits.len() } else { self.commits.len() };
                        ui.label(RichText::new(format!("({n})")).size(12.5).color(p.muted));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        for (name, tip) in [("search", "Filter"), ("sync", "Refresh"), ("list", "Group by folder")] {
                            if (name == "list" && self.tab == Tab::History) || (self.tab == Tab::Graph && name != "sync") {
                                continue;
                            }
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                            let on = (name == "list" && self.tree_view && self.tab == Tab::Changes) || (name == "search" && self.filter_open);
                            if on || resp.hovered() {
                                ui.painter().rect_filled(rect, 6.0, p.hover);
                            }
                            paint_icon(ui, name, rect.shrink(6.0), if on { accent() } else { p.muted });
                            if resp.on_hover_text(tip).clicked() {
                                match name {
                                    "sync" => do_refresh = true,
                                    "search" => {
                                        self.filter_open = !self.filter_open;
                                        if self.filter_open {
                                            self.focus_filter = true;
                                        } else {
                                            self.filter.clear();
                                        }
                                    }
                                    _ => self.tree_view = !self.tree_view,
                                }
                            }
                        }
                    });
                });
                if do_refresh {
                    self.refresh();
                }
                ui.add_space(2.0);

                match self.tab {
                    Tab::Changes => {
                        let staged_n = self.files.iter().filter(|f| f.staged()).count();
                        self.conflict_banner(ui);

                        // Commit box pinned at the bottom.
                        egui::TopBottomPanel::bottom("commit_box")
                            .frame(egui::Frame::none().fill(p.sidebar))
                            .show_inside(ui, |ui| {
                                ui.separator();
                                egui::Frame::none().inner_margin(egui::Margin { left: 14.0, right: 14.0, top: 6.0, bottom: 10.0 }).show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.summary)
                                            .hint_text("Commit message...")
                                            .frame(false)
                                            .desired_rows(4)
                                            .desired_width(f32::INFINITY),
                                    );
                                    let ok = if self.amend {
                                        self.last_commit.is_some()
                                    } else {
                                        staged_n > 0 && !self.summary.trim().is_empty()
                                    };
                                    let label = if self.amend {
                                        "Amend last commit".to_string()
                                    } else {
                                        format!("Commit {} file{}", staged_n, if staged_n == 1 { "" } else { "s" })
                                    };
                                    let w = ui.available_width();
                                    ui.allocate_ui_with_layout(egui::vec2(w, 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let blue = Color32::from_rgb(128, 167, 255);
                                        let (fill, fg) = if ok { (Color32::from_rgb(48, 62, 83), blue) } else { (Color32::from_rgb(43, 53, 66), blue.gamma_multiply(0.55)) };
                                        let btn = egui::Button::new(RichText::new(label).color(fg)).fill(fill).min_size(egui::vec2(0.0, 28.0));
                                        if ui.add_enabled(ok, btn).clicked() {
                                            do_commit = true;
                                        }
                                        {
                                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                                            let n = self.coauthors.iter().filter(|(_, on)| *on).count();
                                            if resp.hovered() || self.coauthor_open.is_some() {
                                                ui.painter().rect_filled(rect, 6.0, p.hover);
                                            }
                                            paint_icon(ui, "users", rect.shrink(5.0), if n > 0 { accent() } else { p.muted });
                                            if n > 0 {
                                                ui.painter().text(rect.right_top() + egui::vec2(-1.0, 4.0), Align2::RIGHT_CENTER, n.to_string(), FontId::proportional(10.0), accent());
                                            }
                                            if resp.on_hover_text("Co-authors").clicked() {
                                                self.coauthor_open = if self.coauthor_open.is_some() { None } else { Some(rect.right_top() + egui::vec2(0.0, -6.0)) };
                                                self.learn_coauthors();
                                            }
                                        }
                                        if self.task_running(T_AI) {
                                            ui.spinner();
                                        } else {
                                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                                            if resp.hovered() {
                                                ui.painter().rect_filled(rect, 6.0, p.hover);
                                            }
                                            paint_icon(ui, "sparkle", rect.shrink(5.0), p.muted);
                                            if resp.on_hover_text("Write message with AI").clicked() {
                                                do_ai = true;
                                            }
                                        }
                                    });
                                });
                                if let Some((subject, body)) = self.last_commit.clone() {
                                    ui.separator();
                                    egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 8.0)).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing.y = 1.0;
                                                ui.label(RichText::new("Last commit").size(12.0).color(p.muted));
                                                ui.add(egui::Label::new(RichText::new(&subject).size(13.0).color(p.text)).truncate());
                                            });
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("Undo").on_hover_text(format!("Undo \"{subject}\" and keep the changes")).clicked() {
                                                    do_undo = true;
                                                }
                                                let amend = egui::Button::new("Amend").fill(if self.amend { accent().gamma_multiply(0.35) } else { p.hover });
                                                if ui.add(amend).clicked() {
                                                    self.amend = !self.amend;
                                                    if self.amend && self.summary.is_empty() {
                                                        self.set_message(&subject, &body);
                                                    }
                                                }
                                            });
                                        });
                                    });
                                }
                            });

                        if self.filter_open {
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                let r = ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter").desired_width(ui.available_width() - 10.0));
                                if self.focus_filter {
                                    r.request_focus();
                                    self.focus_filter = false;
                                }
                            });
                            ui.add_space(4.0);
                        }

                        let all_staged = !self.files.is_empty() && self.files.iter().all(|f| f.staged());
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new(format!("{staged_n} of {} selected", self.files.len())).size(12.5).color(p.muted));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(14.0);
                                if round_check(ui, &p, all_staged).clicked() && !self.files.is_empty() {
                                    actions.push(if all_staged { Action::UnstageAll } else { Action::StageAll });
                                }
                                if ui.add_enabled(!self.files.is_empty(), egui::Button::new(RichText::new("Stash").size(12.0).color(p.muted)).small().frame(false)).clicked() {
                                    do_stash = true;
                                }
                            });
                        });
                        ui.add_space(4.0);
                        self.stash_panel(ui, &p);

                        let q = self.filter.to_lowercase();
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let mut shown: Vec<&git::FileChange> =
                                self.files.iter().filter(|f| q.is_empty() || f.path.to_lowercase().contains(&q)).collect();
                            if self.tree_view {
                                shown.sort_by(|a, b| a.path.cmp(&b.path));
                            }
                            let mut last_dirs: Vec<String> = vec![];
                            for f in shown {
                                let mut depth = 0usize;
                                if self.tree_view {
                                    let parts: Vec<&str> = f.path.split('/').collect();
                                    let dirs = &parts[..parts.len() - 1];
                                    let mut hidden = false;
                                    for (d, name) in dirs.iter().enumerate() {
                                        let full = dirs[..=d].join("/");
                                        if last_dirs.get(d) != Some(&full) {
                                            last_dirs.truncate(d);
                                            last_dirs.push(full.clone());
                                            if hidden {
                                                continue;
                                            }
                                            let inside: Vec<&git::FileChange> = self
                                                .files
                                                .iter()
                                                .filter(|x| x.path.starts_with(&format!("{full}/")))
                                                .collect();
                                            let open = !self.collapsed.contains(&full);
                                            ui.horizontal(|ui| {
                                                ui.add_space(6.0 + d as f32 * 14.0);
                                                let mut all = inside.iter().all(|x| x.staged());
                                                if ui.checkbox(&mut all, "").changed() {
                                                    for x in &inside {
                                                        actions.push(if all { Action::Stage(x.path.clone()) } else { Action::Unstage(x.path.clone()) });
                                                    }
                                                }
                                                icon(ui, if open { "caretDown" } else { "caretRight" }, 12.0, p.muted);
                                                let r = ui.add(
                                                    egui::Label::new(RichText::new(*name).color(p.muted)).sense(Sense::click()),
                                                );
                                                ui.label(RichText::new(inside.len().to_string()).size(11.0).color(p.muted));
                                                if r.clicked() {
                                                    toggle_dir = Some(full.clone());
                                                }
                                            });
                                        }
                                        if self.collapsed.contains(&full) {
                                            hidden = true;
                                        }
                                    }
                                    if dirs.iter().enumerate().any(|(d, _)| self.collapsed.contains(&dirs[..=d].join("/"))) {
                                        continue;
                                    }
                                    depth = dirs.len();
                                }
                                let sel = self.selected_file.as_deref() == Some(f.path.as_str());
                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
                                if sel || resp.hovered() {
                                    ui.painter().rect_filled(rect.shrink2(egui::vec2(6.0, 1.0)), 6.0, if sel { p.hover } else { p.hover.gamma_multiply(0.5) });
                                }
                                let (dir, name) = match f.path.rsplit_once('/') {
                                    Some((_, n)) if self.tree_view => (String::new(), n.to_string()),
                                    Some((d, n)) => (d.to_string(), n.to_string()),
                                    None => (String::new(), f.path.clone()),
                                };
                                let x0 = rect.min.x + 16.0 + depth as f32 * 14.0;
                                let mut badge_ui = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(egui::pos2(x0, rect.center().y - 7.0), egui::vec2(14.0, 14.0))));
                                status_badge(&mut badge_ui, f.badge());
                                let check = egui::Rect::from_center_size(egui::pos2(rect.max.x - 22.0, rect.center().y), egui::vec2(18.0, 18.0));
                                let clip = ui.painter().with_clip_rect(egui::Rect::from_min_max(rect.min, egui::pos2(check.min.x - 6.0, rect.max.y)));
                                let name_g = clip.layout_no_wrap(name, FontId::proportional(13.5), p.text);
                                let tx = x0 + 24.0;
                                let nw = name_g.size().x;
                                clip.galley(egui::pos2(tx, rect.center().y - name_g.size().y / 2.0), name_g, p.text);
                                if !dir.is_empty() {
                                    clip.text(egui::pos2(tx + nw + 8.0, rect.center().y), Align2::LEFT_CENTER, dir, FontId::proportional(12.0), p.muted);
                                }
                                paint_round_check(ui, &p, check, f.staged(), resp.hovered());
                                if resp.clicked() {
                                    let on_check = resp.interact_pointer_pos().is_some_and(|pos| pos.x >= check.min.x - 8.0);
                                    if on_check {
                                        actions.push(if f.staged() { Action::Unstage(f.path.clone()) } else { Action::Stage(f.path.clone()) });
                                    } else {
                                        actions.push(Action::Select(f.path.clone()));
                                    }
                                }
                                resp.context_menu(|ui| {
                                    if ui.button("Discard changes...").clicked() {
                                        ask = Some(Confirm::Discard(f.path.clone()));
                                        ui.close_menu();
                                    }
                                    if ui.button("Discard all changes...").clicked() {
                                        ask = Some(Confirm::DiscardAll);
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("Ignore file").clicked() {
                                        ignore = Some(f.path.clone());
                                        ui.close_menu();
                                    }
                                    if let Some((dir, _)) = f.path.rsplit_once('/') {
                                        if ui.button(format!("Ignore folder ({dir}/)")).clicked() {
                                            ignore = Some(format!("{dir}/"));
                                            ui.close_menu();
                                        }
                                    }
                                    if let Some(ext) = Path::new(&f.path).extension() {
                                        let pat = format!("*.{}", ext.to_string_lossy());
                                        if ui.button(format!("Ignore all {pat} files")).clicked() {
                                            ignore = Some(pat);
                                            ui.close_menu();
                                        }
                                    }
                                    ui.separator();
                                    if ui.button("Copy file path").clicked() {
                                        ui.ctx().copy_text(f.path.clone());
                                        ui.close_menu();
                                    }
                                });
                            }
                            if self.files.is_empty() {
                                ui.add_space(30.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("No local changes").color(p.muted));
                                });
                                ui.horizontal(|ui| {
                                    ui.add_space(8.0);
                                    ui.vertical(|ui| tip_box(ui, &p, self.settings.tips, &mut self.tip));
                                });
                            }
                        });
                    }
                    Tab::History => self.history_tab(ui, &p),
                    Tab::Ignored => self.ignored_tab(ui, &p),
                    Tab::Graph => self.graph_sidebar(ui, &p),
                }
            });

        if let Some(d) = toggle_dir {
            if !self.collapsed.remove(&d) {
                self.collapsed.insert(d);
            }
        }
        if do_ai {
            self.generate_message(ctx);
        }
        if do_stash {
            self.stash_changes();
        }
        if let Some(pattern) = ignore {
            self.ignore(&pattern);
        }
        for a in actions {
            self.apply(a);
        }
        if std::mem::take(&mut self.commit_now) && !self.summary.trim().is_empty() && self.files.iter().any(|f| f.staged()) && !self.amend {
            do_commit = true;
        }
        if do_commit {
            if let Some(repo) = self.repo() {
                if self.settings.auto_include {
                    let _ = git::run(&repo, &["add", "-A"]);
                }
                let (subject, body) = self.commit_parts();
                let body = git::with_coauthors(&body, &self.chosen_coauthors());
                let r = if self.amend { git::amend(&repo, &subject, &body) } else { git::commit(&repo, &subject, &body) };
                if self.report(r) {
                    self.summary.clear();
                    self.amend = false;
                    self.status_msg.clear();
                }
                self.refresh();
            }
        }
        if do_undo {
            if let Some(repo) = self.repo() {
                let msg = self.last_commit.clone();
                let r = git::undo_commit(&repo);
                if self.report(r) {
                    if let Some((s, b)) = msg {
                        self.set_message(&s, &b);
                    }
                }
                self.refresh();
            }
        }
        if ask.is_some() {
            self.confirm = ask;
        }
    }
}

impl App {
    /// Ignored files with a snapshot to compare against.
    pub fn ignored_tab(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut snap = false;
        let mut pick: Option<String> = None;
        egui::TopBottomPanel::bottom("ignored_box").frame(egui::Frame::none().fill(p.sidebar)).show_inside(ui, |ui| {
            ui.separator();
            egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
                ui.label(RichText::new("Local history, never committed.").size(12.0).color(p.muted));
                ui.add_space(6.0);
                if ui.button("Save snapshot").clicked() {
                    snap = true;
                }
            });
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if self.ignored_files.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| ui.label(RichText::new("No ignored files").color(p.muted)));
            }
            for f in &self.ignored_files {
                let sel = self.ignored_sel.as_deref() == Some(f.as_str());
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());
                if sel || resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, p.hover);
                }
                let st = self.ignored_status(f);
                let clip = ui.painter().with_clip_rect(egui::Rect::from_min_max(rect.min, rect.max - egui::vec2(30.0, 0.0)));
                clip.text(rect.left_center() + egui::vec2(12.0, 0.0), Align2::LEFT_CENTER, f, FontId::proportional(13.0), if st == ' ' { p.muted } else { p.text });
                if st != ' ' {
                    ui.painter().text(rect.right_center() - egui::vec2(12.0, 0.0), Align2::RIGHT_CENTER, st, FontId::proportional(12.0), p.muted);
                }
                if resp.clicked() {
                    pick = Some(f.clone());
                }
            }
        });
        if let Some(f) = pick {
            self.ignored_sel = Some(f);
            self.ignored_diff_load();
        }
        if snap {
            self.ignored_snapshot();
        }
    }
}
