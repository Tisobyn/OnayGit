//! Dropdown panels under "Current Repository" and "Current Branch".
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum TbMenu {
    Repo,
    Branch,
}

/// Toolbar cell: two lines of text and an arrow that flips while the menu is open.
pub fn tb_cell(ui: &mut egui::Ui, p: &Pal, label: egui::text::LayoutJob, open: bool) -> bool {
    let b = ui.add(egui::Button::new(label));
    ui.add_space(4.0);
    let (r, a) = ui.allocate_exact_size(egui::vec2(12.0, 16.0), Sense::click());
    let c = r.center();
    let pts = if open {
        vec![c + egui::vec2(-4.0, 2.5), c + egui::vec2(4.0, 2.5), c + egui::vec2(0.0, -2.5)]
    } else {
        vec![c + egui::vec2(-4.0, -2.5), c + egui::vec2(4.0, -2.5), c + egui::vec2(0.0, 2.5)]
    };
    ui.painter().add(egui::Shape::convex_polygon(pts, p.text, egui::Stroke::NONE));
    b.clicked() || a.clicked()
}

fn branch_dates(repo: &Path) -> HashMap<String, String> {
    git::run_any(repo, &["for-each-ref", "refs/heads", "--format=%(refname:short)\t%(committerdate:relative)"])
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

impl App {
    pub fn toggle_tb_menu(&mut self, m: TbMenu) {
        if self.tb_menu == Some(m) {
            self.tb_menu = None;
            return;
        }
        self.tb_menu = Some(m);
        if m == TbMenu::Branch {
            self.branch_dates = self.repo().map(|r| branch_dates(&r)).unwrap_or_default();
            self.branch_tab = 0;
        }
        self.repo_filter.clear();
        self.branch_filter.clear();
    }

    /// Draw the open dropdown. `cells` are the toolbar cells, so clicks on them do not count as outside clicks.
    pub fn tb_menus(&mut self, ctx: &egui::Context, cells: [egui::Rect; 2]) {
        let Some(kind) = self.tb_menu else { return };
        let p = self.pal();
        let cell = if kind == TbMenu::Repo { cells[0] } else { cells[1] };
        let top = cell.bottom() + 1.0;
        let screen = ctx.screen_rect();
        let h = (screen.bottom() - top - 26.0).max(200.0);
        let w = if kind == TbMenu::Repo { cell.width().max(360.0) } else { cell.width().max(330.0) };
        let mut switch_repo: Option<usize> = None;
        let mut switch_branch: Option<String> = None;
        let mut merge_into: Option<String> = None;
        let mut new_branch = false;
        let mut open_prs = false;
        let mut add: Option<u8> = None;
        let mut close = false;
        let mut ask: Option<Confirm> = None;

        let area = egui::Area::new(egui::Id::new("tb_menu_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(cell.left() - 4.0, top))
            .show(ctx, |ui| {
                egui::Frame::none().fill(p.sidebar).stroke(egui::Stroke::new(1.0_f32, p.border)).show(ui, |ui| {
                    ui.set_width(w);
                    ui.set_height(h);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    match kind {
                        TbMenu::Repo => {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let fw = ui.available_width() - 90.0;
                                let r = framed(
                                    ui,
                                    &p,
                                    egui::TextEdit::singleline(&mut self.repo_filter).hint_text("Filter").desired_width(fw),
                                );
                                if !r.has_focus() && self.repo_filter.is_empty() && ui.memory(|m| m.focused().is_none()) {
                                    r.request_focus();
                                }
                                ui.menu_button("Add  \u{25be}", |ui| {
                                    if ui.button("Create repository...").clicked() {
                                        add = Some(0);
                                        ui.close_menu();
                                    }
                                    if ui.button("Clone repository...").clicked() {
                                        add = Some(1);
                                        ui.close_menu();
                                    }
                                    if ui.button("Open from disk...").clicked() {
                                        add = Some(2);
                                        ui.close_menu();
                                    }
                                });
                            });
                            ui.add_space(8.0);
                            let q = self.repo_filter.to_lowercase();
                            let shown: Vec<usize> = (0..self.repos.len())
                                .filter(|&i| !git::is_remote(&self.repos[i]))
                                .filter(|&i| q.is_empty() || repo_name(&self.repos[i]).to_lowercase().contains(&q))
                                .collect();
                            // Recent: the repositories open in tabs. Then one group per owner.
                            let recent: Vec<usize> = shown.iter().copied().filter(|&i| self.tabs.contains(&self.repos[i])).take(5).collect();
                            let mut groups: Vec<(String, Vec<usize>)> = vec![];
                            if !recent.is_empty() && recent.len() < shown.len() {
                                groups.push(("Recent".into(), recent));
                            }
                            let mut owners: Vec<(String, Vec<usize>)> = vec![];
                            for &i in &shown {
                                let o = self.repo_info.get(&self.repos[i]).map(|r| r.owner.clone()).filter(|o| !o.is_empty()).unwrap_or_else(|| "Local".into());
                                match owners.iter_mut().find(|g| g.0 == o) {
                                    Some(g) => g.1.push(i),
                                    None => owners.push((o, vec![i])),
                                }
                            }
                            owners.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
                            groups.extend(owners);
                            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                                for (title, list) in &groups {
                                    ui.add_space(12.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(14.0);
                                        ui.label(RichText::new(title).size(13.5).strong().color(p.text));
                                    });
                                    ui.add_space(4.0);
                                    for &i in list {
                                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::click());
                                        if Some(i) == self.current || resp.hovered() {
                                            ui.painter().rect_filled(rect, 0.0, p.hover);
                                        }
                                        let remote = self.repo_info.get(&self.repos[i]).is_some_and(|r| !r.owner.is_empty());
                                        let ic = egui::Rect::from_center_size(rect.left_center() + egui::vec2(24.0, 0.0), egui::vec2(16.0, 16.0));
                                        paint_icon(ui, if remote { "lock" } else { "folder" }, ic, p.text);
                                        ui.painter().text(rect.left_center() + egui::vec2(44.0, 0.0), Align2::LEFT_CENTER, repo_name(&self.repos[i]), FontId::proportional(14.0), p.text);
                                        if self.repo_info.get(&self.repos[i]).is_some_and(|r| r.dirty > 0) {
                                            ui.painter().circle_filled(rect.right_center() - egui::vec2(24.0, 0.0), 4.0, accent());
                                        }
                                        if resp.clicked() {
                                            switch_repo = Some(i);
                                        }
                                    }
                                }
                                if groups.is_empty() {
                                    ui.add_space(20.0);
                                    ui.vertical_centered(|ui| ui.label(RichText::new("No repositories").color(p.muted)));
                                }
                            });
                        }
                        TbMenu::Branch => {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                let tw = w / 2.0;
                                for (k, name) in ["Branches", "Pull Requests"].iter().enumerate() {
                                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(tw, 36.0), Sense::click());
                                    let on = self.branch_tab == k as u8;
                                    if resp.hovered() {
                                        ui.painter().rect_filled(rect, 0.0, p.hover.gamma_multiply(0.5));
                                    }
                                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, name, FontId::proportional(13.5), if on { p.text } else { p.muted });
                                    if on {
                                        ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - 2.0), egui::vec2(tw, 2.0)), 0.0, accent());
                                    }
                                    if resp.clicked() {
                                        self.branch_tab = k as u8;
                                    }
                                }
                            });
                            ui.separator();
                            if self.branch_tab == 1 {
                                ui.add_space(30.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("Pull requests of this repository").color(p.muted));
                                    ui.add_space(8.0);
                                    if ui.button("Open pull requests").clicked() {
                                        open_prs = true;
                                    }
                                });
                                return;
                            }
                            if self.branch_tab == 2 {
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(14.0);
                                    ui.label(RichText::new(format!("Merge into {}", self.branch)).size(13.5).strong().color(p.text));
                                });
                                ui.add_space(4.0);
                                let list_h = ui.available_height() - 56.0;
                                egui::ScrollArea::vertical().max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
                                    for b in self.branches.iter().filter(|b| **b != self.branch) {
                                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
                                        if resp.hovered() {
                                            ui.painter().rect_filled(rect, 0.0, p.hover);
                                        }
                                        ui.painter().text(rect.left_center() + egui::vec2(40.0, 0.0), Align2::LEFT_CENTER, b, FontId::proportional(13.5), p.text);
                                        if resp.clicked() {
                                            merge_into = Some(b.clone());
                                        }
                                    }
                                });
                                ui.separator();
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(14.0);
                                    let bw = ui.available_width() - 14.0;
                                    if ui.add(egui::Button::new(RichText::new("Cancel").strong()).min_size(egui::vec2(bw, 30.0))).clicked() {
                                        self.branch_tab = 0;
                                    }
                                });
                                return;
                            }
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let fw = ui.available_width() - 120.0;
                                let r = framed(
                                    ui,
                                    &p,
                                    egui::TextEdit::singleline(&mut self.branch_filter).hint_text("Filter").desired_width(fw),
                                );
                                if !r.has_focus() && self.branch_filter.is_empty() && ui.memory(|m| m.focused().is_none()) {
                                    r.request_focus();
                                }
                                let can = !self.branch_filter.trim().is_empty();
                                if ui.add_enabled(can, egui::Button::new("New Branch")).clicked() {
                                    new_branch = true;
                                }
                            });
                            ui.add_space(6.0);
                            let q = self.branch_filter.to_lowercase();
                            let def = ["main", "master"].iter().find(|d| self.branches.iter().any(|b| b == *d)).map(|d| d.to_string());
                            let list_h = ui.available_height() - 56.0;
                            egui::ScrollArea::vertical().max_height(list_h).auto_shrink([false, false]).show(ui, |ui| {
                                let row = |ui: &mut egui::Ui, name: &str, when: &str, cur: bool, local: bool, ask: &mut Option<Confirm>| -> bool {
                                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
                                    if cur || resp.hovered() {
                                        ui.painter().rect_filled(rect, 0.0, p.hover);
                                    }
                                    if cur {
                                        let c = rect.left_center() + egui::vec2(22.0, 0.0);
                                        let st = egui::Stroke::new(1.6_f32, p.text);
                                        ui.painter().line_segment([c + egui::vec2(-5.0, 0.0), c + egui::vec2(-1.5, 3.5)], st);
                                        ui.painter().line_segment([c + egui::vec2(-1.5, 3.5), c + egui::vec2(5.5, -4.0)], st);
                                    }
                                    ui.painter().text(rect.left_center() + egui::vec2(40.0, 0.0), Align2::LEFT_CENTER, name, FontId::proportional(13.5), p.text);
                                    ui.painter().text(rect.right_center() - egui::vec2(14.0, 0.0), Align2::RIGHT_CENTER, when, FontId::proportional(12.5), p.muted);
                                    if local && !cur {
                                        resp.context_menu(|ui| {
                                            if ui.button("Delete branch...").clicked() {
                                                *ask = Some(Confirm::DeleteBranch(name.to_string()));
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                    resp.clicked()
                                };
                                let title = |ui: &mut egui::Ui, t: &str| {
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(14.0);
                                        ui.label(RichText::new(t).size(13.5).strong().color(p.text));
                                    });
                                    ui.add_space(4.0);
                                };
                                if let Some(d) = def.as_ref().filter(|d| q.is_empty() || d.contains(&q)) {
                                    title(ui, "Default Branch");
                                    if row(ui, d, self.branch_dates.get(d).map(|s| s.as_str()).unwrap_or(""), *d == self.branch, true, &mut ask) {
                                        switch_branch = Some(d.clone());
                                    }
                                }
                                let others: Vec<&String> = self.branches.iter().filter(|b| Some(*b) != def.as_ref()).filter(|b| q.is_empty() || b.to_lowercase().contains(&q)).collect();
                                if !others.is_empty() {
                                    title(ui, "Other Branches");
                                    for b in others {
                                        if row(ui, b, self.branch_dates.get(b).map(|s| s.as_str()).unwrap_or(""), *b == self.branch, true, &mut ask) {
                                            switch_branch = Some(b.clone());
                                        }
                                    }
                                }
                                let remote: Vec<&String> = self.remote_branches.iter().filter(|b| q.is_empty() || b.to_lowercase().contains(&q)).collect();
                                if !remote.is_empty() {
                                    title(ui, "Remote Branches");
                                    for b in remote {
                                        if row(ui, b, "", false, false, &mut ask) {
                                            switch_branch = Some(b.split_once('/').map(|x| x.1).unwrap_or(b).to_string());
                                        }
                                    }
                                }
                            });
                            ui.separator();
                            ui.add_space(10.0);
                            let others: Vec<String> = self.branches.iter().filter(|b| **b != self.branch).cloned().collect();
                            let merging = self.branch_tab == 2;
                            let label = if merging { "Cancel".to_string() } else { format!("Choose a branch to merge into {}", self.branch) };
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                let bw = ui.available_width() - 14.0;
                                if ui.add_enabled(!others.is_empty(), egui::Button::new(RichText::new(label).strong()).min_size(egui::vec2(bw, 30.0))).clicked() {
                                    self.branch_tab = if merging { 0 } else { 2 };
                                }
                            });
                        }
                    }
                });
            });

        // Close on Escape or a click outside.
        let pressed = ctx.input(|i| i.pointer.any_pressed());
        let pos = ctx.input(|i| i.pointer.interact_pos());
        let in_popup = ctx.memory(|m| m.any_popup_open());
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if pressed && !in_popup {
            if let Some(pt) = pos {
                if !area.response.rect.contains(pt) && !cells.iter().any(|c| c.expand(2.0).contains(pt)) {
                    close = true;
                }
            }
        }

        if ask.is_some() {
            self.confirm = ask;
            close = true;
        }
        if let Some(i) = switch_repo {
            self.current = Some(i);
            self.selected_file = None;
            self.selected_commit = None;
            self.commit_diff.clear();
            self.refresh();
            close = true;
        }
        if let Some(b) = switch_branch {
            if let Some(repo) = self.repo() {
                let r = git::checkout(&repo, &b);
                self.report(r);
                self.refresh();
            }
            close = true;
        }
        if let Some(b) = merge_into {
            if let Some(repo) = self.repo() {
                let r = git::merge(&repo, &b);
                self.report(r);
                self.refresh();
            }
            close = true;
        }
        if new_branch {
            self.tour_done(1);
            if let Some(repo) = self.repo() {
                let name = self.branch_filter.trim().replace(' ', "-");
                let r = git::create_branch(&repo, &name);
                self.report(r);
                self.refresh();
            }
            close = true;
        }
        if open_prs {
            self.open_pull_requests(ctx);
            close = true;
        }
        match add {
            Some(0) => {
                self.screen = Screen::Home;
                self.create_open = true;
                self.create_anchor = egui::pos2(20.0, 400.0);
                self.create_dir = None;
                self.create_branch = "main".into();
                close = true;
            }
            Some(1) => {
                self.open_clone();
                close = true;
            }
            Some(2) => {
                if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                    self.open_repo(d);
                }
                close = true;
            }
            _ => {}
        }
        if close {
            self.tb_menu = None;
        }
    }
}
