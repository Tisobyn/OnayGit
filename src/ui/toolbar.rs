#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn toolbar(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut goto_home = false;
        let mut open_dialog: Option<Dialog> = None;
        let mut goto_settings = false;
        let mut switch_repo: Option<usize> = None;
        let mut switch_branch: Option<String> = None;
        let mut create_branch = false;
        let mut merge_branch: Option<String> = None;
        let mut ask: Option<Confirm> = None;
        let remote = self.remote_name.clone();
        let rstyle = self.repo().map(|r| self.style_of(&r));
        let mut sync: Option<(&str, Vec<&str>)> = None;

        egui::TopBottomPanel::top("toolbar")
            .exact_height(46.0)
            .frame(
                egui::Frame::none()
                    .fill(p.toolbar)
                    .stroke(egui::Stroke::new(1.0_f32, p.border))
                    .inner_margin(egui::Margin::symmetric(4.0, 0.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;

                    // Repository block
                    let name = self.repo().map(|r| repo_name(&r)).unwrap_or_default();
                    ui.add_space(6.0);
                    match &rstyle {
                        Some(st) => icon(ui, &st.icon, 18.0, ACCENTS[st.color % ACCENTS.len()].1),
                        None => icon(ui, "folder", 18.0, p.muted),
                    }
                    ui.menu_button(two_line("Current Repository", &name, &p), |ui| {
                        ui.set_min_width(220.0);
                        for (i, r) in self.repos.iter().enumerate() {
                            if ui.selectable_label(Some(i) == self.current, repo_name(r)).clicked() {
                                switch_repo = Some(i);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("Tags...").clicked() {
                            open_dialog = Some(Dialog::Tags);
                            ui.close_menu();
                        }
                        if ui.button("Worktrees...").clicked() {
                            open_dialog = Some(Dialog::Worktrees);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("All repositories...").clicked() {
                            goto_home = true;
                            ui.close_menu();
                        }
                    });
                    ui.separator();

                    // Branch block
                    icon(ui, "branch", 18.0, p.muted);
                    ui.menu_button(two_line("Current Branch", &self.branch, &p), |ui| {
                        ui.set_min_width(240.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.branch_filter)
                                .hint_text("Filter branches")
                                .desired_width(f32::INFINITY),
                        );
                        let q = self.branch_filter.to_lowercase();
                        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                            ui.label(RichText::new("LOCAL").size(10.5).color(p.muted));
                            for b in self.branches.iter().filter(|b| q.is_empty() || b.to_lowercase().contains(&q)) {
                                if ui.selectable_label(*b == self.branch, b).clicked() {
                                    switch_branch = Some(b.clone());
                                    ui.close_menu();
                                }
                            }
                            if !self.remote_branches.is_empty() {
                                ui.add_space(4.0);
                                ui.label(RichText::new("REMOTE").size(10.5).color(p.muted));
                                for b in self.remote_branches.iter().filter(|b| q.is_empty() || b.to_lowercase().contains(&q)) {
                                    if ui.selectable_label(false, b).clicked() {
                                        let local = b.split_once('/').map(|x| x.1).unwrap_or(b);
                                        switch_branch = Some(local.to_string());
                                        ui.close_menu();
                                    }
                                }
                            }
                        });
                        ui.separator();
                        let others: Vec<String> =
                            self.branches.iter().filter(|b| **b != self.branch).cloned().collect();
                        ui.add_enabled_ui(!others.is_empty(), |ui| {
                            ui.menu_button(format!("Merge into {}...", self.branch), |ui| {
                                for b in &others {
                                    if ui.button(b).clicked() {
                                        merge_branch = Some(b.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                            ui.menu_button("Delete branch...", |ui| {
                                for b in &others {
                                    if ui.button(b).clicked() {
                                        ask = Some(Confirm::DeleteBranch(b.clone()));
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                        ui.separator();
                        ui.label("New branch");
                        ui.text_edit_singleline(&mut self.new_branch);
                        let ok = !self.new_branch.trim().is_empty();
                        if ui.add_enabled(ok, egui::Button::new("Create branch")).clicked() {
                            create_branch = true;
                            ui.close_menu();
                        }
                    });
                    ui.separator();

                    // Sync block (Fetch / Pull / Push / Publish like GitHub Desktop)
                    let (title, sub, args): (String, String, Option<(&str, Vec<&str>)>) = if !self.has_remote {
                        ("No remote".to_string(), "Add a remote to sync".into(), None)
                    } else {
                        match self.ahead_behind {
                            None => (
                                "Publish branch".to_string(),
                                format!("Push to {remote}"),
                                Some(("Publishing", vec!["push", "-u", remote.as_str(), "HEAD"])),
                            ),
                            Some((_, b)) if b > 0 => (
                                format!("Pull {remote}"),
                                format!("{b} commits behind"),
                                Some(("Pulling", vec!["pull", "--no-rebase", "--no-edit"])),
                            ),
                            Some((a, _)) if a > 0 => (
                                format!("Push {remote}"),
                                format!("{a} commits ahead"),
                                Some(("Pushing", vec!["push"])),
                            ),
                            _ => (
                                format!("Fetch {remote}"),
                                ago(self.last_fetched),
                                Some(("Fetching", vec!["fetch", "--all", "--prune"])),
                            ),
                        }
                    };
                    let enabled = args.is_some() && self.busy.is_none();
                    let label = if let Some(b) = &self.busy {
                        two_line(&format!("{b}..."), "Please wait", &p)
                    } else {
                        two_line(&title, &sub, &p)
                    };
                    let sync_icon = if title.starts_with("Publish") {
                        "publish"
                    } else if title.starts_with("Pull") {
                        "pull"
                    } else if title.starts_with("Push") {
                        "push"
                    } else {
                        "sync"
                    };
                    icon(ui, sync_icon, 18.0, p.muted);
                    if ui.add_enabled(enabled, egui::Button::new(label).frame(false)).clicked() {
                        sync = args;
                    }
                    if self.busy.is_some() {
                        ui.spinner();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::click());
                        if resp.hovered() {
                            ui.painter().rect_filled(rect, 6.0, p.hover);
                        }
                        paint_icon(ui, "settings", rect.shrink(5.0), p.muted);
                        if resp.on_hover_text("Settings").clicked() {
                            goto_settings = true;
                        }
                    });
                });
            });

        if open_dialog.is_some() {
            self.dialog = open_dialog;
        }
        if goto_home {
            self.screen = Screen::Home;
        }
        if goto_settings {
            self.open_settings();
        }
        if let Some(i) = switch_repo {
            self.current = Some(i);
            self.selected_file = None;
            self.selected_commit = None;
            self.commit_diff.clear();
            self.refresh();
        }
        if let Some(b) = switch_branch {
            if let Some(repo) = self.repo() {
                let r = git::checkout(&repo, &b);
                self.report(r);
                self.refresh();
            }
        }
        if let Some(b) = merge_branch {
            if let Some(repo) = self.repo() {
                let r = git::merge(&repo, &b);
                self.report(r);
                self.refresh();
            }
        }
        if ask.is_some() {
            self.confirm = ask;
        }
        if create_branch {
            if let Some(repo) = self.repo() {
                let name = self.new_branch.trim().replace(' ', "-");
                let r = git::create_branch(&repo, &name);
                if self.report(r) {
                    self.new_branch.clear();
                }
                self.refresh();
            }
        }
        if let Some((label, args)) = sync {
            if let Some(repo) = self.repo() {
                self.spawn_in(ctx, repo, label, &args);
            }
        }
    }
}
