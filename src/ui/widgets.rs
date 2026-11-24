#[allow(unused_imports)]
use crate::prelude::*;

pub fn repo_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn two_line(label: &str, value: &str, p: &Pal) -> LayoutJob {
    let mut j = LayoutJob::default();
    j.append(
        &format!("{label}\n"),
        0.0,
        TextFormat { font_id: FontId::proportional(10.0), color: p.muted, ..Default::default() },
    );
    j.append(
        value,
        0.0,
        TextFormat { font_id: FontId::proportional(12.5), color: p.text, ..Default::default() },
    );
    j
}

pub fn ago(t: Option<Instant>) -> String {
    match t {
        None => "Never fetched".into(),
        Some(t) => {
            let s = t.elapsed().as_secs();
            if s < 60 {
                "Last fetched just now".into()
            } else {
                format!("Last fetched {} min ago", s / 60)
            }
        }
    }
}


pub fn rail_label(ui: &mut egui::Ui, text: &str, p: &Pal) {
    ui.add_space(2.0);
    ui.label(RichText::new(text).size(11.5).color(p.muted));
    ui.add_space(6.0);
}

pub fn rail_item(ui: &mut egui::Ui, icon: &str, text: &str, selected: bool, p: &Pal) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::click());
    if selected || resp.hovered() {
        ui.painter().rect_filled(rect, 6.0, p.hover);
    }
    let icon_rect = egui::Rect::from_center_size(rect.min + egui::vec2(20.0, 17.0), egui::vec2(18.0, 18.0));
    paint_icon(ui, icon, icon_rect, p.muted);
    ui.painter().text(
        rect.min + egui::vec2(42.0, 17.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(14.0),
        p.text,
    );
    resp
}

pub fn repo_row(ui: &mut egui::Ui, path: &Path, p: &Pal, style: &RepoStyle, info: Option<&RepoInfo>) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 56.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 8.0, p.hover);
    }
    let av = egui::Rect::from_center_size(rect.left_center() + egui::vec2(24.0, 0.0), egui::vec2(34.0, 34.0));
    ui.painter().rect_filled(av, 8.0, if resp.hovered() { p.border } else { p.hover });
    ui.painter().text(
        av.center(),
        Align2::CENTER_CENTER,
        repo_name(path).chars().next().unwrap_or('?').to_uppercase().to_string(),
        FontId::proportional(14.0),
        ACCENTS[style.color % ACCENTS.len()].1,
    );
    ui.painter().text(rect.min + egui::vec2(56.0, 19.0), Align2::LEFT_CENTER, repo_name(path), FontId::proportional(14.0), p.text);
    ui.painter().with_clip_rect(egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x - 230.0, rect.max.y))).text(
        rect.min + egui::vec2(56.0, 38.0),
        Align2::LEFT_CENTER,
        path.display().to_string(),
        FontId::proportional(12.0),
        p.muted,
    );
    let x = rect.max.x - 210.0;
    match info {
        None => {
            ui.painter().text(egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, "Status unavailable", FontId::proportional(12.0), p.muted);
        }
        Some(i) => {
            paint_icon(ui, "branch", egui::Rect::from_center_size(egui::pos2(x + 7.0, rect.min.y + 19.0), egui::vec2(13.0, 13.0)), p.muted);
            ui.painter().text(egui::pos2(x + 20.0, rect.min.y + 19.0), Align2::LEFT_CENTER, &i.branch, FontId::proportional(12.0), p.muted);
            let (txt, col) = if i.dirty == 0 {
                ("Clean".to_string(), Color32::from_rgb(105, 186, 134))
            } else {
                (format!("{} change{}", i.dirty, if i.dirty == 1 { "" } else { "s" }), Color32::from_rgb(230, 170, 70))
            };
            let w = ui.fonts(|f| f.layout_no_wrap(txt.clone(), FontId::proportional(12.0), col).size().x);
            ui.painter().text(egui::pos2(x, rect.min.y + 38.0), Align2::LEFT_CENTER, txt, FontId::proportional(12.0), col);
            let sync = match i.sync {
                None => "No upstream".to_string(),
                Some((0, 0)) => "Up to date".to_string(),
                Some((a, b)) => format!("{}{}", if a > 0 { format!("\u{2191}{a} ") } else { String::new() }, if b > 0 { format!("\u{2193}{b}") } else { String::new() }),
            };
            ui.painter().text(egui::pos2(x + w + 10.0, rect.min.y + 38.0), Align2::LEFT_CENTER, sync, FontId::proportional(12.0), p.muted);
        }
    }
    resp
}

pub fn repo_card(ui: &mut egui::Ui, path: &Path, p: &Pal, style: &RepoStyle) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(200.0, 70.0), Sense::click());
    ui.painter().rect_filled(rect, 8.0, if resp.hovered() { p.hover } else { p.input });
    ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0_f32, p.border));
    let ic = egui::Rect::from_center_size(rect.min + egui::vec2(20.0, 24.0), egui::vec2(16.0, 16.0));
    paint_icon(ui, &style.icon, ic, ACCENTS[style.color % ACCENTS.len()].1);
    ui.painter().text(
        rect.min + egui::vec2(36.0, 24.0),
        Align2::LEFT_CENTER,
        repo_name(path),
        FontId::proportional(14.0),
        p.text,
    );
    let parent = path.parent().map(|d| d.display().to_string()).unwrap_or_default();
    ui.painter().with_clip_rect(rect).text(
        rect.min + egui::vec2(12.0, 46.0),
        Align2::LEFT_CENTER,
        parent,
        FontId::proportional(11.0),
        p.muted,
    );
    resp
}

pub fn page_title(ui: &mut egui::Ui, text: &str, p: &Pal) {
    ui.label(RichText::new(text).size(19.0).color(p.text));
    ui.add_space(12.0);
}

pub fn setting_row(
    ui: &mut egui::Ui,
    title: &str,
    desc: &str,
    p: &Pal,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::none()
        .fill(p.input)
        .stroke(egui::Stroke::new(1.0_f32, p.border))
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    ui.label(RichText::new(title).size(13.5).color(p.text));
                    ui.label(RichText::new(desc).size(12.0).color(p.muted));
                });
                add(ui);
            });
        });
    ui.add_space(8.0);
}

/// Small colored square for a change type: modified, added, deleted, renamed, conflict.
pub fn status_badge(ui: &mut egui::Ui, badge: char) {
    let (color, mark) = match badge {
        'A' => (Color32::from_rgb(63, 185, 80), "+"),
        'D' => (Color32::from_rgb(248, 81, 73), "\u{2212}"),
        '!' => (Color32::from_rgb(248, 81, 73), "!"),
        'R' => (Color32::from_rgb(163, 113, 247), "\u{2192}"),
        _ => (Color32::from_rgb(77, 150, 255), "\u{2022}"),
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::hover());
    ui.painter().rect_stroke(rect.shrink(0.5), 3.0, egui::Stroke::new(1.2_f32, color));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, mark, FontId::proportional(11.0), color);
}
