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

/// The line that pairs with `idx` inside a block of removed then added lines.
fn partner(lines: &[DiffLine], idx: usize) -> Option<usize> {
    match lines[idx].kind {
        Kind::Del => {
            let mut s = idx;
            while s > 0 && lines[s - 1].kind == Kind::Del {
                s -= 1;
            }
            let mut e = idx;
            while e + 1 < lines.len() && lines[e + 1].kind == Kind::Del {
                e += 1;
            }
            let p = e + 1 + (idx - s);
            (p < lines.len() && lines[p].kind == Kind::Add).then_some(p)
        }
        Kind::Add => {
            let mut a = idx;
            while a > 0 && lines[a - 1].kind == Kind::Add {
                a -= 1;
            }
            let mut s = a;
            while s > 0 && lines[s - 1].kind == Kind::Del {
                s -= 1;
            }
            let p = s + (idx - a);
            (p < a && lines[p].kind == Kind::Del).then_some(p)
        }
        _ => None,
    }
}

/// Character range of `a` that differs from `b` (common start and end removed).
fn changed_range(a: &str, b: &str) -> Option<(usize, usize)> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let pre = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..].iter().rev().zip(b[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let end = a.len() - suf;
    (end > pre && (pre > 0 || suf > 0)).then_some((pre, end))
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
    let row_h = size + 8.0;
    let cw = ui.fonts(|f| f.layout_no_wrap("0".repeat(100), font.clone(), Color32::WHITE).size().x) / 100.0;
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
                        let mut text_start = rect.min.x + off + num_w * 2.0 + cw;
                        if selection.is_some() {
                            let b = egui::Rect::from_min_size(
                                rect.min + egui::vec2(cw, 2.0),
                                egui::vec2(btn_w, row_h - 4.0),
                            );
                            painter.rect_filled(b, 4.0, accent());
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
                                } else if resp.hovered() {
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
                        let mark = match l.kind {
                            Kind::Add => Color32::from_rgb(63, 185, 80),
                            Kind::Del => Color32::from_rgb(248, 81, 73),
                            _ => p.muted,
                        };
                        if l.kind != Kind::Ctx {
                            let bar = egui::Rect::from_min_size(egui::pos2(rect.min.x + off + num_w * 2.0 + cw * 0.8, rect.min.y), egui::vec2(2.0, row_h));
                            painter.rect_filled(bar, 0.0, mark);
                        }
                        painter.text(
                            egui::pos2(rect.min.x + off + num_w * 2.0 + cw * 1.5, cy),
                            Align2::LEFT_CENTER,
                            sign,
                            font.clone(),
                            mark,
                        );
                        if let Some((a, b)) = partner(lines, idx).and_then(|q| changed_range(&l.text, &lines[q].text)) {
                            let c = if l.kind == Kind::Add {
                                Color32::from_rgba_unmultiplied(63, 185, 80, 80)
                            } else {
                                Color32::from_rgba_unmultiplied(248, 81, 73, 80)
                            };
                            let x0 = rect.min.x + text_x + cw * a as f32;
                            let hl = egui::Rect::from_min_max(egui::pos2(x0, rect.min.y + 1.0), egui::pos2(x0 + cw * (b - a) as f32, rect.max.y - 1.0));
                            painter.rect_filled(hl, 2.0, c);
                        }
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
