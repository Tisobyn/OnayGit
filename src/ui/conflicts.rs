//! Banner shown while a merge, rebase, cherry-pick or revert is waiting for you.
use crate::prelude::*;

impl App {
    pub fn conflict_banner(&mut self, ui: &mut egui::Ui) {
        let Some(op) = self.repo_op else { return };
        let conflicts = self.files.iter().filter(|f| f.conflicted()).count();
        let mut cont = false;
        let mut abort = false;
        egui::Frame::none()
            .fill(Color32::from_rgb(110, 76, 20))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let text = if conflicts > 0 {
                    format!("{} in progress: {conflicts} conflicted file{}", op.label(), if conflicts == 1 { "" } else { "s" })
                } else {
                    format!("{} in progress: all conflicts resolved", op.label())
                };
                ui.label(RichText::new(text).color(Color32::WHITE).size(12.5));
                ui.horizontal(|ui| {
                    if ui.add_enabled(conflicts == 0, egui::Button::new("Continue").small()).clicked() {
                        cont = true;
                    }
                    if ui.add(egui::Button::new("Abort").small()).clicked() {
                        abort = true;
                    }
                });
            });
        if cont {
            self.continue_op();
        }
        if abort {
            self.confirm = Some(Confirm::AbortOp);
        }
    }
}
