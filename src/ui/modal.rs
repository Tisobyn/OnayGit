//! Dialog look shared by all dialogs: dimmed backdrop, bold title, close button, bordered fields.
use crate::prelude::*;

/// Where a dialog sits: centered, or with a corner pinned to a point (a popover).
pub enum Place {
    Center,
    At(egui::Pos2, Align2),
}

#[allow(clippy::too_many_arguments)]
/// Show a dialog. Returns true when the user closed it (close button or Escape).
pub fn modal(ctx: &egui::Context, p: &Pal, id: &str, width: f32, place: Place, title: &str, desc: &str, content: impl FnOnce(&mut egui::Ui)) -> bool {
    let mut close = false;
    if matches!(place, Place::Center) {
        egui::Area::new(egui::Id::new((id, "dim"))).order(egui::Order::Middle).fixed_pos(egui::Pos2::ZERO).show(ctx, |ui| {
            let r = ctx.screen_rect();
            ui.painter().rect_filled(r, 0.0, Color32::from_black_alpha(120));
            ui.allocate_rect(r, Sense::click());
        });
    }
    let frame = egui::Frame::window(&ctx.style())
        .inner_margin(egui::Margin::same(if matches!(place, Place::Center) { 24.0 } else { 14.0 }))
        .rounding(12.0)
        .fill(p.bg);
    let mut w = egui::Window::new(id).title_bar(false).collapsible(false).resizable(false).order(egui::Order::Foreground).frame(frame);
    w = match place {
        Place::Center => w.anchor(Align2::CENTER_CENTER, [0.0, 0.0]),
        Place::At(pos, pivot) => w.pivot(pivot).fixed_pos(pos),
    };
    let popover = matches!(place, Place::At(..));
    w.show(ctx, |ui| {
        ui.set_width(width);
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).size(if popover { 14.0 } else { 16.0 }).strong().color(p.text));
            if !popover {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(rect, 6.0, p.hover);
                    }
                    paint_icon(ui, "close", rect.shrink(5.0), p.muted);
                    if resp.clicked() {
                        close = true;
                    }
                });
            }
        });
        if !desc.is_empty() {
            ui.label(RichText::new(desc).size(13.0).color(p.muted));
        }
        ui.add_space(if popover { 4.0 } else { 12.0 });
        content(ui);
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
    }
    close
}

/// Label above a bordered text field.
pub fn dialog_field(ui: &mut egui::Ui, p: &Pal, label: &str, text: &mut String, hint: &str) -> egui::Response {
    ui.add_space(6.0);
    ui.label(RichText::new(label).size(13.0).strong().color(p.text));
    ui.add(egui::TextEdit::singleline(text).hint_text(hint).desired_width(f32::INFINITY).margin(egui::vec2(12.0, 10.0)))
}

/// Label above a button-like field that opens a folder picker.
pub fn folder_field(ui: &mut egui::Ui, p: &Pal, label: &str, value: Option<&Path>) -> egui::Response {
    ui.add_space(6.0);
    ui.label(RichText::new(label).size(13.0).strong().color(p.text));
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), Sense::click());
    ui.painter().rect_filled(rect, 6.0, if resp.hovered() { p.hover } else { p.input });
    ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, p.border));
    paint_icon(ui, "folder", egui::Rect::from_center_size(rect.left_center() + egui::vec2(20.0, 0.0), egui::vec2(15.0, 15.0)), p.muted);
    let text = match value {
        Some(d) => d.display().to_string(),
        None => "Choose folder...".to_string(),
    };
    ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0))).text(rect.left_center() + egui::vec2(38.0, 0.0), Align2::LEFT_CENTER, text, FontId::proportional(13.5), p.text);
    resp
}

/// Primary action button in 1git's blue.
pub fn primary_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let blue = Color32::from_rgb(128, 167, 255);
    let (fill, fg) = if enabled { (Color32::from_rgb(48, 62, 83), blue) } else { (Color32::from_rgb(43, 53, 66), blue.gamma_multiply(0.7)) };
    ui.add(egui::Button::new(RichText::new(text).color(fg)).fill(fill).min_size(egui::vec2(0.0, 30.0)).sense(if enabled { Sense::click() } else { Sense::hover() }))
}

/// Secondary button (Cancel).
pub fn plain_button(ui: &mut egui::Ui, p: &Pal, text: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(text).color(p.text)).min_size(egui::vec2(0.0, 30.0)))
}

/// Destructive confirm button.
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(Color32::from_rgb(200, 50, 50)).min_size(egui::vec2(0.0, 30.0)))
}

/// Row of dialog buttons aligned to the right. Add the main button first.
pub fn button_row(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(14.0);
    let w = ui.available_width();
    ui.allocate_ui_with_layout(egui::vec2(w, 32.0), egui::Layout::right_to_left(egui::Align::Center), add);
}

/// Bordered multi-line field.
pub fn dialog_area(ui: &mut egui::Ui, text: &mut String, hint: &str, rows: usize) -> egui::Response {
    ui.add(egui::TextEdit::multiline(text).hint_text(hint).desired_rows(rows).desired_width(f32::INFINITY).margin(egui::vec2(12.0, 10.0)))
}

/// Small section title inside a dialog.
pub fn dialog_section(ui: &mut egui::Ui, p: &Pal, text: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(text).size(13.0).strong().color(p.text));
}
