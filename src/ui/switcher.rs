//! Quick repository switcher (Cmd+P).
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn switcher_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut pick: Option<PathBuf> = None;
        let mut close = false;
        egui::Window::new("Switch repository")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([420.0, 0.0])
            .anchor(Align2::CENTER_TOP, [0.0, 90.0])
            .show(ctx, |ui| {
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.switcher_text)
                        .hint_text("Go to repository")
                        .desired_width(f32::INFINITY),
                );
                r.request_focus();
                let q = self.switcher_text.to_lowercase();
                let hits: Vec<PathBuf> = self
                    .repos
                    .iter()
                    .filter(|r| q.is_empty() || repo_name(r).to_lowercase().contains(&q))
                    .take(8)
                    .cloned()
                    .collect();
                ui.add_space(4.0);
                for (i, r) in hits.iter().enumerate() {
                    let current = Some(r) == self.repo().as_ref();
                    let label = if current { format!("{} (current)", repo_name(r)) } else { repo_name(r) };
                    if rail_item(ui, "folder", &label, i == 0, &p).clicked() {
                        pick = Some(r.clone());
                    }
                }
                if hits.is_empty() {
                    ui.label(RichText::new("No match").color(p.muted));
                }
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    pick = hits.first().cloned();
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    close = true;
                }
            });
        if let Some(r) = pick {
            self.dialog = None;
            self.open_repo(r);
        } else if close {
            self.dialog = None;
        }
    }
}
