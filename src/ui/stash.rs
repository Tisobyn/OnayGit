//! Stashed changes list in the Changes tab.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn stash_panel(&mut self, ui: &mut egui::Ui, p: &Pal) {
        if self.stashes.is_empty() {
            return;
        }
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, 0.0, p.hover);
        }
        let arrow = if self.stash_open { "caretDown" } else { "caretRight" };
        let a = egui::Rect::from_center_size(rect.min + egui::vec2(16.0, 13.0), egui::vec2(12.0, 12.0));
        paint_icon(ui, arrow, a, p.muted);
        ui.painter().text(
            rect.min + egui::vec2(30.0, 13.0),
            Align2::LEFT_CENTER,
            format!("Stashed changes ({})", self.stashes.len()),
            FontId::proportional(13.0),
            p.text,
        );
        if resp.clicked() {
            self.stash_open = !self.stash_open;
        }
        if self.stash_open {
            let mut select: Option<String> = None;
            let mut op: Option<(StashOp, String)> = None;
            for s in &self.stashes {
                let sel = self.selected_stash.as_deref() == Some(s.name.as_str());
                let fill = if sel { ACCENT } else { Color32::TRANSPARENT };
                egui::Frame::none().fill(fill).inner_margin(egui::Margin::symmetric(8.0, 1.0)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let fg = if sel { Color32::WHITE } else { p.text };
                        let label = egui::Label::new(RichText::new(&s.message).size(12.5).color(fg))
                            .truncate()
                            .sense(Sense::click());
                        if ui.add(label).clicked() {
                            select = Some(s.name.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Drop").clicked() {
                                op = Some((StashOp::Drop, s.name.clone()));
                            }
                            if ui.small_button("Pop").clicked() {
                                op = Some((StashOp::Pop, s.name.clone()));
                            }
                            if ui.small_button("Apply").clicked() {
                                op = Some((StashOp::Apply, s.name.clone()));
                            }
                        });
                    });
                });
            }
            if let Some((o, name)) = op {
                self.stash_op(o, &name);
            } else if let Some(name) = select {
                self.select_stash(Some(name));
            }
        }
        ui.separator();
    }
}
