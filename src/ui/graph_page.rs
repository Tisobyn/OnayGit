//! Graph tab: branches and tags on the left, the commit graph of the whole repository on the right.
use crate::prelude::*;
use crate::ui::graph::{graph_px, paint_graph_row};

const ROW_H: f32 = 34.0;

impl App {
    pub fn graph_sidebar(&mut self, ui: &mut egui::Ui, p: &Pal) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let w = (ui.available_width() - 10.0).max(40.0);
            framed(ui, p, egui::TextEdit::singleline(&mut self.graph_find).hint_text("Find branch or tag...").desired_width(w));
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            for (k, label) in ["All", "Local", "Remote", "Tags"].iter().enumerate() {
                if ui.selectable_label(self.graph_kind == k as u8, *label).clicked() {
                    self.graph_kind = k as u8;
                }
            }
        });
        ui.add_space(6.0);
        let q = self.graph_find.to_lowercase();
        let mut jump: Option<String> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (name, kind, hash) in &self.graph_list {
                if self.graph_kind != 0 && *kind != self.graph_kind {
                    continue;
                }
                if !q.is_empty() && !name.to_lowercase().contains(&q) {
                    continue;
                }
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), Sense::click());
                let current = *kind == 1 && *name == self.branch;
                if resp.hovered() {
                    ui.painter().rect_filled(rect, 0.0, p.hover);
                }
                let c = rect.left_center() + egui::vec2(24.0, 0.0);
                if current {
                    let st = egui::Stroke::new(1.6_f32, Color32::from_rgb(63, 185, 80));
                    ui.painter().line_segment([c + egui::vec2(-5.0, 0.0), c + egui::vec2(-1.5, 3.5)], st);
                    ui.painter().line_segment([c + egui::vec2(-1.5, 3.5), c + egui::vec2(5.5, -4.0)], st);
                } else {
                    let ic = match kind {
                        2 => "publish",
                        3 => "sp_bookmark",
                        _ => "branch",
                    };
                    paint_icon(ui, ic, egui::Rect::from_center_size(c, egui::vec2(13.0, 13.0)), p.muted);
                }
                let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(6.0, 0.0)));
                clip.text(rect.left_center() + egui::vec2(42.0, 0.0), Align2::LEFT_CENTER, name, FontId::proportional(13.5), if current { Color32::from_rgb(63, 185, 80) } else { p.text });
                if resp.clicked() {
                    jump = Some(hash.clone());
                }
            }
        });
        if let Some(h) = jump {
            self.graph_scroll = self.graph_commits.iter().position(|c| c.hash == h);
        }
    }

    pub fn graph_main(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut pick: Option<String> = None;
        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.label(RichText::new(format!("{} commits", self.graph_commits.len())).size(13.0).color(p.muted));
            });
            ui.add_space(8.0);
            ui.separator();
            if self.graph_commits.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| ui.label(RichText::new("No commits yet").size(15.0).color(p.muted)));
                return;
            }
            let head = self.commits.first().map(|c| c.hash.clone()).unwrap_or_default();
            let gw = graph_px(self.graph_lanes).max(22.0);
            let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
            if let Some(i) = self.graph_scroll.take() {
                area = area.vertical_scroll_offset((i as f32 * ROW_H - 3.0 * ROW_H).max(0.0));
            }
            area.show_rows(ui, ROW_H, self.graph_commits.len(), |ui, range| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for i in range {
                    let (Some(c), Some(row)) = (self.graph_commits.get(i), self.graph_rows.get(i)) else { continue };
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_H), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(rect, 6.0, p.hover);
                    }
                    paint_graph_row(ui.painter(), egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 0.0), egui::vec2(gw, ROW_H)), row);
                    let right = rect.right() - 18.0;
                    let (hash_x, author_x) = (right - 190.0, right - 340.0);
                    let clip_all = ui.painter().with_clip_rect(rect);
                    clip_all.text(egui::pos2(right, rect.center().y), Align2::RIGHT_CENTER, &c.when, FontId::proportional(12.5), p.muted);
                    clip_all.text(egui::pos2(hash_x + 100.0, rect.center().y), Align2::RIGHT_CENTER, c.hash.chars().take(7).collect::<String>(), FontId::monospace(12.0), p.muted);
                    let who = ui.painter().with_clip_rect(egui::Rect::from_min_max(egui::pos2(author_x, rect.top()), egui::pos2(hash_x - 8.0, rect.bottom())));
                    who.text(egui::pos2(author_x, rect.center().y), Align2::LEFT_CENTER, &c.author, FontId::proportional(12.5), p.muted);

                    // Branch and tag chips, then the subject.
                    let mut x = rect.left() + 8.0 + gw + 10.0;
                    let limit = author_x - 12.0;
                    let mut names: Vec<String> = vec![];
                    if c.hash == head {
                        names.push("HEAD".into());
                    }
                    if let Some(r) = self.graph_refs.get(&c.hash) {
                        names.extend(r.iter().cloned());
                    }
                    let shown = names.len().min(2);
                    for n in names.iter().take(shown) {
                        let g = ui.painter().layout_no_wrap(n.clone(), FontId::proportional(11.0), accent());
                        let w = g.size().x + 12.0;
                        if x + w > limit - 60.0 {
                            break;
                        }
                        let chip = egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 9.0), egui::vec2(w, 18.0));
                        ui.painter().rect_filled(chip, 5.0, accent().gamma_multiply(0.18));
                        ui.painter().galley(chip.center() - g.size() / 2.0, g, accent());
                        x += w + 5.0;
                    }
                    if names.len() > shown {
                        let t = format!("+{}", names.len() - shown);
                        let g = ui.painter().layout_no_wrap(t, FontId::proportional(11.0), p.muted);
                        let w = g.size().x + 12.0;
                        let chip = egui::Rect::from_min_size(egui::pos2(x, rect.center().y - 9.0), egui::vec2(w, 18.0));
                        ui.painter().rect_filled(chip, 5.0, p.hover);
                        ui.painter().galley(chip.center() - g.size() / 2.0, g, p.muted);
                        x += w + 5.0;
                    }
                    let subj = ui.painter().with_clip_rect(egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(limit, rect.bottom())));
                    subj.text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, &c.subject, FontId::proportional(13.5), p.text);
                    if resp.clicked() {
                        pick = Some(c.hash.clone());
                    }
                }
            });
        });
        if let Some(h) = pick {
            self.select_commit(h);
            self.tab = Tab::History;
        }
    }
}
