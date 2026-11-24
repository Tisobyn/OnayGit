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
        TextFormat { font_id: FontId::proportional(10.5), color: p.muted, ..Default::default() },
    );
    j.append(
        value,
        0.0,
        TextFormat { font_id: FontId::proportional(13.0), color: p.text, ..Default::default() },
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

pub fn repo_row(ui: &mut egui::Ui, path: &Path, p: &Pal, style: &RepoStyle) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 46.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 6.0, p.hover);
    }
    let ic = egui::Rect::from_center_size(rect.min + egui::vec2(16.0, 23.0), egui::vec2(18.0, 18.0));
    paint_icon(ui, &style.icon, ic, ACCENTS[style.color % ACCENTS.len()].1);
    ui.painter().text(
        rect.min + egui::vec2(36.0, 15.0),
        Align2::LEFT_CENTER,
        repo_name(path),
        FontId::proportional(14.0),
        p.text,
    );
    ui.painter().text(
        rect.min + egui::vec2(36.0, 33.0),
        Align2::LEFT_CENTER,
        path.display().to_string(),
        FontId::proportional(11.5),
        p.muted,
    );
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
    ui.label(RichText::new(text).size(22.0).color(p.text));
    ui.add_space(14.0);
}

pub fn setting_row(
    ui: &mut egui::Ui,
    title: &str,
    desc: &str,
    p: &Pal,
    add: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(300.0);
            ui.label(RichText::new(title).size(14.0));
            ui.label(RichText::new(desc).size(12.0).color(p.muted));
        });
        add(ui);
    });
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);
}
