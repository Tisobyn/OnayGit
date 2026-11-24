#[allow(unused_imports)]
use crate::prelude::*;
use crate::ui::toolbar_menu::{tb_cell, TbMenu};

impl App {
    pub fn toolbar(&mut self, ctx: &egui::Context) {
        self.tour_rects[1] = None;
        let p = self.pal();
        let mut goto_settings = false;
        let mut open_pub = false;
        let remote = self.remote_name.clone();
        let rstyle = self.repo().map(|r| self.style_of(&r));
        let mut sync: Option<(&str, Vec<&str>)> = None;
        let mut toggle: Option<TbMenu> = None;
        let mut cells = [egui::Rect::NOTHING; 2];

        egui::TopBottomPanel::top("toolbar")
            .exact_height(44.0)
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
                    let r0 = ui.cursor().left_top();
                    if tb_cell(ui, &p, two_line("Current Repository", &name, &p), self.tb_menu == Some(TbMenu::Repo)) {
                        toggle = Some(TbMenu::Repo);
                    }
                    cells[0] = egui::Rect::from_min_max(egui::pos2(r0.x - 40.0, ui.max_rect().top()), egui::pos2(ui.cursor().left() + 160.0, ui.max_rect().bottom()));
                    toolbar_divider(ui);

                    // Branch block
                    icon(ui, "branch", 18.0, p.muted);
                    let b0 = ui.cursor().left_top();
                    if tb_cell(ui, &p, two_line("Current Branch", &self.branch, &p), self.tb_menu == Some(TbMenu::Branch)) {
                        toggle = Some(TbMenu::Branch);
                    }
                    cells[1] = egui::Rect::from_min_max(egui::pos2(b0.x - 30.0, ui.max_rect().top()), egui::pos2(ui.cursor().left() + 20.0, ui.max_rect().bottom()));
                    self.tour_rects[0] = Some(cells[1]);
                    toolbar_divider(ui);

                    // Sync block (Fetch / Pull / Push / Publish like GitHub Desktop)
                    let (title, sub, args): (String, String, Option<(&str, Vec<&str>)>) = if !self.has_remote {
                        ("Publish repository".to_string(), "Create it on GitHub or GitLab".into(), None)
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
                    let no_remote = !self.has_remote;
                    let enabled = (args.is_some() || no_remote) && self.busy.is_none();
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
                    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                        if no_remote {
                            open_pub = true;
                        } else {
                            sync = args;
                        }
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

        if let Some(m) = toggle {
            self.toggle_tb_menu(m);
        }
        self.tb_menus(ctx, cells);

        if open_pub {
            self.open_publish();
        }
        if goto_settings {
            self.open_settings();
        }
        if let Some((label, args)) = sync {
            if let Some(repo) = self.repo() {
                self.spawn_in(ctx, repo, label, &args);
            }
        }
    }
}

/// Thin vertical line between toolbar cells, with equal room on both sides.
fn toolbar_divider(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);
}
