#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn sidebar(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut actions: Vec<Action> = vec![];
        let mut do_commit = false;
        let mut do_undo = false;
        let mut do_stash = false;
        let mut tag_here: Option<String> = None;
        let mut new_compare: Option<Option<String>> = None;
        let mut ignore: Option<String> = None;
        let mut ask: Option<Confirm> = None;
        let mut pick_commit: Option<String> = None;
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
                            ui.painter().rect_filled(line, 0.0, ACCENT);
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

                        // Commit box pinned at the bottom.
                        egui::TopBottomPanel::bottom("commit_box")
                            .frame(egui::Frame::none().fill(p.sidebar).inner_margin(egui::Margin::same(8.0)))
                            .show_inside(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.summary)
                                        .hint_text("Summary (required)")
                                        .desired_width(f32::INFINITY),
                                );
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
                                let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
                                    .fill(ACCENT)
                                    .min_size(egui::vec2(ui.available_width(), 28.0));
                                if ui.add_enabled(ok, btn).clicked() {
                                    do_commit = true;
                                }
                                if let Some((subject, body)) = self.last_commit.clone() {
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut self.amend, "Amend").changed() && self.amend {
                                            if self.summary.is_empty() {
                                                self.summary = subject.clone();
                                                self.desc = body;
                                            }
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("Undo").on_hover_text("Undo last commit, keep changes").clicked() {
                                                do_undo = true;
                                            }
                                            ui.add(
                                                egui::Label::new(RichText::new(format!("Latest: {subject}")).size(11.0).color(p.muted))
                                                    .truncate(),
                                            );
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
                            for f in self.files.iter().filter(|f| q.is_empty() || f.path.to_lowercase().contains(&q)) {
                                let sel = self.selected_file.as_deref() == Some(f.path.as_str());
                                let fill = if sel { ACCENT } else { Color32::TRANSPARENT };
                                egui::Frame::none().fill(fill).inner_margin(egui::Margin::symmetric(6.0, 1.0)).show(
                                    ui,
                                    |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            let mut staged = f.staged();
                                            if ui.checkbox(&mut staged, "").changed() {
                                                actions.push(if staged {
                                                    Action::Stage(f.path.clone())
                                                } else {
                                                    Action::Unstage(f.path.clone())
                                                });
                                            }
                                            let (dir, name) = match f.path.rsplit_once('/') {
                                                Some((d, n)) => (format!("{d}/"), n.to_string()),
                                                None => (String::new(), f.path.clone()),
                                            };
                                            let fg = if sel { Color32::WHITE } else { p.text };
                                            let dim = if sel { Color32::from_rgb(220, 235, 255) } else { p.muted };
                                            let mut job = LayoutJob::default();
                                            job.append(
                                                &dir,
                                                0.0,
                                                TextFormat { font_id: FontId::proportional(13.0), color: dim, ..Default::default() },
                                            );
                                            job.append(
                                                &name,
                                                0.0,
                                                TextFormat { font_id: FontId::proportional(13.0), color: fg, ..Default::default() },
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
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let c = match f.badge() {
                                                    'A' => Color32::from_rgb(63, 185, 80),
                                                    'D' => Color32::from_rgb(248, 81, 73),
                                                    _ => Color32::from_rgb(210, 153, 34),
                                                };
                                                let c = if sel { Color32::WHITE } else { c };
                                                ui.label(RichText::new(f.badge().to_string()).color(c).strong().size(12.0));
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
                            }
                        });
                    }
                    Tab::History => {
                        ui.horizontal(|ui| {
                            ui.add_space(6.0);
                            let label = match &self.compare {
                                Some(b) => format!("Compare: {b}"),
                                None => "Compare to branch...".to_string(),
                            };
                            ui.menu_button(label, |ui| {
                                ui.set_min_width(200.0);
                                if ui.button("None (show history)").clicked() {
                                    new_compare = Some(None);
                                    ui.close_menu();
                                }
                                ui.separator();
                                let others = self.branches.iter().filter(|b| **b != self.branch);
                                for b in others.chain(self.remote_branches.iter()) {
                                    if ui.button(b).clicked() {
                                        new_compare = Some(Some(b.clone()));
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                        if let Some(b) = &self.compare {
                            let (ours, theirs) = self.compare_counts;
                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("{theirs} behind, {ours} ahead of {b}"))
                                        .size(11.5)
                                        .color(p.muted),
                                );
                            });
                        }
                        ui.separator();
                        let list: &Vec<Commit> =
                            if self.compare.is_some() { &self.compare_commits } else { &self.commits };
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for c in list {
                                let sel = self.selected_commit.as_deref() == Some(c.hash.as_str());
                                let (rect, resp) =
                                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), Sense::click());
                                let fill = if sel {
                                    ACCENT
                                } else if resp.hovered() {
                                    p.hover
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(rect, 0.0, fill);
                                let fg = if sel { Color32::WHITE } else { p.text };
                                let dim = if sel { Color32::from_rgb(220, 235, 255) } else { p.muted };
                                let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)));
                                clip.text(
                                    rect.min + egui::vec2(10.0, 14.0),
                                    Align2::LEFT_CENTER,
                                    &c.subject,
                                    FontId::proportional(13.0),
                                    fg,
                                );
                                clip.text(
                                    rect.min + egui::vec2(10.0, 31.0),
                                    Align2::LEFT_CENTER,
                                    format!("{} · {}", c.author, c.when),
                                    FontId::proportional(11.0),
                                    dim,
                                );
                                if resp.clicked() {
                                    pick_commit = Some(c.hash.clone());
                                }
                                resp.context_menu(|ui| {
                                    if ui.button("Create tag here...").clicked() {
                                        tag_here = Some(c.hash.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Copy commit hash").clicked() {
                                        ui.ctx().copy_text(c.hash.clone());
                                        ui.close_menu();
                                    }
                                });
                            }
                            if list.is_empty() {
                                ui.add_space(30.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("No commits yet").color(p.muted));
                                });
                            }
                        });
                    }
                }
            });

        self.tab = new_tab;
        if do_stash {
            self.stash_changes();
        }
        if let Some(h) = tag_here {
            self.tag_target = h;
            self.dialog = Some(Dialog::Tags);
        }
        if let Some(c) = new_compare {
            self.set_compare(c);
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
        if let Some(h) = pick_commit {
            if let Some(repo) = self.repo() {
                self.commit_diff = git::parse_diff(&git::show(&repo, &h), true);
            }
            self.selected_commit = Some(h);
        }
    }
}
