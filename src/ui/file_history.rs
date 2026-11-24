//! File history window: the commits that changed one file, and who wrote each line.
use crate::prelude::*;

impl App {
    pub fn file_history_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut pick: Option<String> = None;
        let title = format!("History of {}", self.fh_path);
        let close = modal(ctx, &p, "file_history", 760.0, Place::Center, &title, "", |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.fh_tab, 0, format!("Commits ({})", self.fh_commits.len()));
                ui.selectable_value(&mut self.fh_tab, 1, "Blame");
            });
            ui.add_space(6.0);
            if self.fh_tab == 0 {
                egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if self.fh_commits.is_empty() {
                        ui.label(RichText::new("No commits found for this file.").color(p.muted));
                    }
                    for c in &self.fh_commits {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), Sense::click());
                        if resp.hovered() {
                            ui.painter().rect_filled(rect, 6.0, p.hover);
                        }
                        let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)));
                        clip.text(rect.min + egui::vec2(10.0, 13.0), Align2::LEFT_CENTER, &c.subject, FontId::proportional(13.5), p.text);
                        let short: String = c.hash.chars().take(7).collect();
                        clip.text(rect.min + egui::vec2(10.0, 29.0), Align2::LEFT_CENTER, format!("{} \u{b7} {} \u{b7} {short}", c.author, c.when), FontId::proportional(11.5), p.muted);
                        if resp.clicked() {
                            pick = Some(c.hash.clone());
                        }
                    }
                });
            } else {
                egui::ScrollArea::both().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if self.fh_blame.is_empty() {
                        ui.label(RichText::new("No blame for this file (new, binary or untracked).").color(p.muted));
                    }
                    let mut last = String::new();
                    for (n, (hash, author, text)) in self.fh_blame.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            let same = *hash == last;
                            let (r, _) = ui.allocate_exact_size(egui::vec2(170.0, 16.0), Sense::hover());
                            if !same {
                                ui.painter().with_clip_rect(r).text(r.left_center(), Align2::LEFT_CENTER, format!("{hash}  {author}"), FontId::proportional(11.5), p.muted);
                            }
                            ui.label(RichText::new(format!("{:>4}", n + 1)).monospace().size(12.0).color(p.muted));
                            ui.label(RichText::new(text).monospace().size(12.5).color(p.text));
                            last = hash.clone();
                        });
                    }
                });
            }
        });
        if let Some(h) = pick {
            self.select_commit(h);
            self.tab = Tab::History;
            self.dialog = None;
        }
        if close {
            self.dialog = None;
        }
    }
}
