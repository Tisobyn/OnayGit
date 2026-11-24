#[allow(unused_imports)]
use crate::prelude::*;

impl App {
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
        let mut new_tab = self.tab;

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(300.0)
            .width_range(240.0..=520.0)
            .frame(egui::Frame::none().fill(p.sidebar))
            .show(ctx, |ui| {
                // Tabs
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let w = ui.available_width() / 2.0;
                    for (t, label) in [
                        (Tab::Changes, format!("Changes  {}", self.files.len())),
                        (Tab::History, "History".to_string()),
                    ] {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), Sense::click());
                        let active = self.tab == t;
                        if resp.hovered() {
                            ui.painter().rect_filled(rect, 0.0, p.hover);
                        }
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            label,
                            FontId::proportional(13.0),
                            if active { p.text } else { p.muted },
                        );
                        if active {
                            let line = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x, rect.max.y - 2.0),
                                egui::vec2(rect.width(), 2.0),
                            );
                            ui.painter().rect_filled(line, 0.0, accent());
                        }
                        if resp.clicked() {
                            new_tab = t;
                        }
                    }
                });
                ui.separator();

                match self.tab {
                    Tab::Changes => {
                        let staged_n = self.files.iter().filter(|f| f.staged()).count();
                        self.conflict_banner(ui);

                        // Commit box pinned at the bottom.
                        egui::TopBottomPanel::bottom("commit_box")
                            .frame(egui::Frame::none().fill(p.sidebar).inner_margin(egui::Margin::same(8.0)))
                            .show_inside(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let w = ui.available_width() - 30.0;
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.summary)
                                            .hint_text("Summary (required)")
                                            .desired_width(w),
                                    );
                                    if self.task_running(T_AI) {
                                        ui.spinner();
                                    } else {
                                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
                                        if resp.hovered() {
                                            ui.painter().rect_filled(rect, 6.0, p.hover);
                                        }
                                        paint_icon(ui, "sparkle", rect.shrink(4.0), accent());
                                        if resp.on_hover_text("Write message with AI").clicked() {
                                            do_ai = true;
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.desc)
                                        .hint_text("Description")
                                        .desired_rows(3)
                                        .desired_width(f32::INFINITY),
                                );
                                ui.add_space(6.0);
                                let ok = if self.amend {
                                    self.last_commit.is_some()
                                } else {
                                    staged_n > 0 && !self.summary.trim().is_empty()
                                };
                                let label = if self.amend {
                                    "Amend last commit".to_string()
                                } else if staged_n == 1 {
                                    format!("Commit 1 file to {}", self.branch)
                                } else {
                                    format!("Commit {} files to {}", staged_n, self.branch)
                                };
                                let (fill, fg) = if ok { (accent(), Color32::WHITE) } else { (p.hover, p.muted) };
                                let btn = egui::Button::new(RichText::new(label).color(fg).strong())
                                    .fill(fill)
                                    .min_size(egui::vec2(ui.available_width(), 28.0));
                                if ui.add_enabled(ok, btn).clicked() {
                                    do_commit = true;
                                }
                                if let Some((subject, body)) = self.last_commit.clone() {
                                    ui.add_space(4.0);
                                    let w = ui.available_width();
                                    ui.allocate_ui_with_layout(egui::vec2(w, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.set_max_width(w);
                                        if ui.checkbox(&mut self.amend, "Amend").changed() && self.amend {
                                            if self.summary.is_empty() {
                                                self.summary = subject.clone();
                                                self.desc = body;
                                            }
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("Undo").on_hover_text(format!("Undo \"{subject}\" and keep the changes")).clicked() {
                                                do_undo = true;
                                            }
                                        });
                                    });
                                }
                            });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(6.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.filter)
                                    .hint_text("Filter")
                                    .desired_width(ui.available_width() - 6.0),
                            );
                        });
                        ui.add_space(2.0);

                        let all_staged = !self.files.is_empty() && self.files.iter().all(|f| f.staged());
                        ui.horizontal(|ui| {
                            ui.add_space(6.0);
                            let mut all = all_staged;
                            let label = format!(
                                "{} changed file{}",
                                self.files.len(),
                                if self.files.len() == 1 { "" } else { "s" }
                            );
                            if ui.checkbox(&mut all, label).changed() {
                                actions.push(if all { Action::StageAll } else { Action::UnstageAll });
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(6.0);
                                if ui.selectable_label(self.tree_view, "Tree").on_hover_text("Group files by folder").clicked() {
                                    self.tree_view = !self.tree_view;
                                }
                                let has = !self.files.is_empty();
                                if ui.add_enabled(has, egui::Button::new("Stash").small()).clicked() {
                                    do_stash = true;
                                }
                            });
                        });
                        ui.separator();
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
                                let fill = if sel { p.hover } else { Color32::TRANSPARENT };
                                egui::Frame::none().fill(fill).inner_margin(egui::Margin::symmetric(6.0, 1.0)).show(
                                    ui,
                                    |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.add_space(depth as f32 * 14.0);
                                            let mut staged = f.staged();
                                            if ui.checkbox(&mut staged, "").changed() {
                                                actions.push(if staged {
                                                    Action::Stage(f.path.clone())
                                                } else {
                                                    Action::Unstage(f.path.clone())
                                                });
                                            }
                                            let (dir, name) = match f.path.rsplit_once('/') {
                                                Some((_, n)) if self.tree_view => (String::new(), n.to_string()),
                                                Some((d, n)) => (format!("{d}/"), n.to_string()),
                                                None => (String::new(), f.path.clone()),
                                            };
                                            status_badge(ui, f.badge());
                                            let fg = p.text;
                                            let dim = p.muted;
                                            let mut job = LayoutJob::default();
                                            job.append(
                                                &dir,
                                                0.0,
                                                TextFormat { font_id: FontId::proportional(12.5), color: dim, ..Default::default() },
                                            );
                                            job.append(
                                                &name,
                                                0.0,
                                                TextFormat { font_id: FontId::proportional(12.5), color: fg, ..Default::default() },
                                            );
                                            let r = ui.add(egui::Label::new(job).truncate().sense(Sense::click()));
                                            if r.clicked() {
                                                actions.push(Action::Select(f.path.clone()));
                                            }
                                            r.context_menu(|ui| {
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

                                        });
                                    },
                                );
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
                }
            });

        self.tab = new_tab;
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
        if do_commit {
            if let Some(repo) = self.repo() {
                let r = if self.amend {
                    git::amend(&repo, self.summary.trim(), &self.desc)
                } else {
                    git::commit(&repo, self.summary.trim(), &self.desc)
                };
                if self.report(r) {
                    self.summary.clear();
                    self.desc.clear();
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
                        self.summary = s;
                        self.desc = b;
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
