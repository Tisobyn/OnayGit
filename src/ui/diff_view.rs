//! Compact diff renderer with line numbers and optional line selection.
#[allow(unused_imports)]
use crate::prelude::*;

/// Something the user did in the diff.
pub enum DiffEvent {
    /// Toggle the selection mark of the line at this index.
    ToggleLine(usize),
    /// Stage or unstage the whole hunk whose header is at this index.
    Hunk(usize),
}

/// Draw `lines`. With `selection` set, changed lines get a checkbox in the gutter
/// and every hunk header gets a button labelled `hunk_label`.
pub fn draw_diff(
    ui: &mut egui::Ui,
    lines: &[DiffLine],
    p: &Pal,
    size: f32,
    selection: Option<&HashSet<usize>>,
    hunk_label: &str,
) -> Vec<DiffEvent> {
    let mut events = Vec::new();
    let font = FontId::monospace(size);
    let row_h = size + 6.0;
    let cw = ui.fonts(|f| f.glyph_width(&font, '0'));
    let num_w = cw * 5.0;
    let off = if selection.is_some() { cw * 3.0 } else { 0.0 };
    let gutter_w = off + num_w * 2.0 + cw;
    let text_x = off + num_w * 2.0 + cw * 2.5;
    let max_chars = lines.iter().map(|l| l.text.chars().count()).max().unwrap_or(0);
    let content_w = (text_x + cw * (max_chars as f32 + 4.0)).max(ui.available_width());
    let btn_w = cw * (hunk_label.chars().count() as f32 + 3.0);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show_rows(ui, row_h, lines.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for idx in range {
                let l = &lines[idx];
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(content_w, row_h), Sense::click());
                let painter = ui.painter().clone();
                let cy = rect.center().y;
                let click_x = resp
                    .clicked()
                    .then(|| resp.interact_pointer_pos())
                    .flatten()
                    .map(|pos| pos.x - rect.min.x);

                match l.kind {
                    Kind::Hunk => {
                        painter.rect_filled(rect, 0.0, p.hunk_bg);
                        let mut text_start = rect.min.x + off + num_w * 2.0 + cw;
                        if selection.is_some() {
                            let b = egui::Rect::from_min_size(
                                rect.min + egui::vec2(cw, 2.0),
                                egui::vec2(btn_w, row_h - 4.0),
                            );
                            painter.rect_filled(b, 3.0, accent());
                            painter.text(
                                b.center(),
                                Align2::CENTER_CENTER,
                                hunk_label,
                                FontId::proportional(size - 1.0),
                                Color32::WHITE,
                            );
                            text_start = b.max.x + cw;
                            if click_x.is_some_and(|x| x >= cw && x <= cw + btn_w) {
                                events.push(DiffEvent::Hunk(idx));
                            }
                        }
                        painter.text(
                            egui::pos2(text_start, cy),
                            Align2::LEFT_CENTER,
                            &l.text,
                            font.clone(),
                            p.muted,
                        );
                    }
                    Kind::File => {
                        painter.rect_filled(rect, 0.0, p.hover);
                        painter.text(
                            egui::pos2(rect.min.x + 10.0, cy),
                            Align2::LEFT_CENTER,
                            &l.text,
                            FontId::proportional(size + 1.0),
                            p.text,
                        );
                    }
                    Kind::Meta => {
                        painter.text(
                            egui::pos2(rect.min.x + 10.0, cy),
                            Align2::LEFT_CENTER,
                            &l.text,
                            font.clone(),
                            p.muted,
                        );
                    }
                    Kind::Add | Kind::Del | Kind::Ctx => {
                        let (bg, gut, sign) = match l.kind {
                            Kind::Add => (Some(p.add_bg), p.add_gut, "+"),
                            Kind::Del => (Some(p.del_bg), p.del_gut, "-"),
                            _ => (None, p.sidebar, " "),
                        };
                        if let Some(bg) = bg {
                            painter.rect_filled(rect, 0.0, bg);
                        }
                        let gutter = egui::Rect::from_min_size(rect.min, egui::vec2(gutter_w, row_h));
                        painter.rect_filled(gutter, 0.0, gut);

                        if let Some(sel) = selection {
                            if l.kind != Kind::Ctx {
                                let side = size - 2.0;
                                let b = egui::Rect::from_center_size(
                                    egui::pos2(rect.min.x + off * 0.5, cy),
                                    egui::vec2(side, side),
                                );
                                if sel.contains(&idx) {
                                    painter.rect_filled(b, 2.0, accent());
                                    paint_icon(ui, "check", b.shrink(1.0), Color32::WHITE);
                                } else {
                                    painter.rect_stroke(b, 2.0, egui::Stroke::new(1.0_f32, p.muted));
                                }
                                if click_x.is_some_and(|x| x <= gutter_w) {
                                    events.push(DiffEvent::ToggleLine(idx));
                                }
                            }
                        }
                        if let Some(n) = l.old {
                            painter.text(
                                egui::pos2(rect.min.x + off + num_w, cy),
                                Align2::RIGHT_CENTER,
                                n.to_string(),
                                font.clone(),
                                p.muted,
                            );
                        }
                        if let Some(n) = l.new {
                            painter.text(
                                egui::pos2(rect.min.x + off + num_w * 2.0, cy),
                                Align2::RIGHT_CENTER,
                                n.to_string(),
                                font.clone(),
                                p.muted,
                            );
                        }
                        painter.text(
                            egui::pos2(rect.min.x + off + num_w * 2.0 + cw * 1.5, cy),
                            Align2::LEFT_CENTER,
                            sign,
                            font.clone(),
                            p.muted,
                        );
                        painter.text(
                            egui::pos2(rect.min.x + text_x, cy),
                            Align2::LEFT_CENTER,
                            &l.text,
                            font.clone(),
                            p.text,
                        );
                    }
                }
            }
        });
    events
}
