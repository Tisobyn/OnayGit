use crate::prelude::*;
use crate::ui::toolbar_menu::{tb_cell, TbMenu};

/// (icon, title, subtitle, optional (task label, git args)).
pub type SyncInfo = (&'static str, String, String, Option<(&'static str, Vec<String>)>);

impl App {
    pub fn toolbar(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut goto_settings = false;
        let rstyle = self.repo().map(|r| self.style_of(&r));
        let mut open_wt = false;
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
                    toolbar_divider(ui);

                    // Worktrees
                    icon(ui, "layers", 18.0, p.muted);
                    let n = self.worktrees.len().max(1);
                    let sub = if n == 1 { "1 checkout".to_string() } else { format!("{n} checkouts") };
                    if ui.add(egui::Button::new(two_line("Worktrees", &sub, &p))).clicked() {
                        open_wt = true;
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

        if goto_settings {
            self.open_settings();
        }
        if open_wt {
            self.dialog = Some(Dialog::Worktrees);
        }
    }

    /// What the sync button does now: (icon, title, subtitle, git args). No args and a remote means busy or nothing to do.
    pub fn sync_info(&self) -> SyncInfo {
        let remote = self.remote_name.clone();
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        if !self.has_remote {
            return ("publish", "Publish repository".into(), "Create it on GitHub or GitLab".into(), None);
        }
        match self.ahead_behind {
            None => ("publish", "Publish branch".into(), format!("Push to {remote}"), Some(("Publishing", vec!["push".into(), "-u".into(), remote, "HEAD".into()]))),
            Some((_, b)) if b > 0 => ("pull", format!("Pull {remote}"), format!("{b} commits behind"), Some(("Pulling", args(&["pull", "--no-rebase", "--no-edit"])))),
            Some((a, _)) if a > 0 => ("push", format!("Push {remote}"), format!("{a} commits ahead"), Some(("Pushing", args(&["push"])))),
            _ => ("sync", format!("Fetch {remote}"), ago(self.last_fetched), Some(("Fetching", args(&["fetch", "--all", "--prune"])))),
        }
    }
}

/// Thin vertical line between toolbar cells, with equal room on both sides.
fn toolbar_divider(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);
}
