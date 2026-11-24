#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn confirm_window(&mut self, ctx: &egui::Context) {
        let Some(c) = self.confirm.clone() else { return };
        let (title, text) = match &c {
            Confirm::Discard(p) => ("Discard changes", format!("Discard changes to {p}? This cannot be undone.")),
            Confirm::DiscardAll => ("Discard all changes", "Discard ALL local changes? This cannot be undone.".to_string()),
            Confirm::DeleteBranch(b) => ("Delete branch", format!("Delete branch {b}? Unmerged commits can be lost.")),
            Confirm::ResetHard(h) => (
                "Hard reset",
                format!("Reset the current branch to {} and discard all local changes?", &h[..7.min(h.len())]),
            ),
            Confirm::Rewrite(op, h) => {
                let what = match op {
                    git::RewriteOp::Drop => "Drop",
                    git::RewriteOp::Squash => "Squash",
                    git::RewriteOp::Reword => "Reword",
                    git::RewriteOp::MoveOlder => "Move",
                };
                (
                    "Rewrite history",
                    format!("{what} commit {}? This rewrites history. Pushed commits will need a force push.", &h[..7.min(h.len())]),
                )
            }
            Confirm::AbortOp => ("Abort", "Abort the operation in progress and go back to how things were?".to_string()),
        };
        let (mut yes, mut no) = (false, false);
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(text);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        no = true;
                    }
                    let red = egui::Button::new(RichText::new("Confirm").color(Color32::WHITE))
                        .fill(Color32::from_rgb(200, 50, 50));
                    if ui.add(red).clicked() {
                        yes = true;
                    }
                });
            });
        if yes {
            if let Some(repo) = self.repo() {
                let r = match &c {
                    Confirm::Discard(p) => match self.files.iter().find(|f| &f.path == p) {
                        Some(f) => git::discard(&repo, f),
                        None => Ok(()),
                    },
                    Confirm::DiscardAll => git::discard_all(&repo),
                    Confirm::DeleteBranch(b) => git::delete_branch(&repo, b).map(|_| ()),
                    Confirm::ResetHard(h) => git::reset_to(&repo, h, "hard").map(|_| ()),
                    Confirm::Rewrite(op, h) => git::rewrite(&repo, *op, h, "").map(|_| ()),
                    Confirm::AbortOp => match self.repo_op {
                        Some(op) => git::op_abort(&repo, op).map(|_| ()),
                        None => Ok(()),
                    },
                };
                self.report(r);
                self.refresh();
            }
        }
        if yes || no {
            self.confirm = None;
        }
    }

    pub fn status_bar(&mut self, ctx: &egui::Context) {
        if self.status_msg.is_empty() {
            return;
        }
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(90, 30, 34))
                    .inner_margin(egui::Margin::symmetric(10.0, 5.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_msg).color(Color32::WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Dismiss").clicked() {
                            self.status_msg.clear();
                        }
                    });
                });
            });
    }
}
