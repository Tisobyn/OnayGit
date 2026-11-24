//! Guided tour: small tips next to the real controls, shown once.
use crate::prelude::*;

const STEPS: [(&str, &str); 3] = [
    ("Find your current branch", "Open the Current Branch menu to see your branch, switch branches, or create one."),
    ("Create your own branch", "Name a local branch and choose Create branch to switch to it. Nothing changes until you do."),
    ("Switch spaces", "Use the Spaces button in the left rail to jump between groups of repositories."),
];

impl App {
    /// Mark a step as done.
    pub fn tour_done(&mut self, step: usize) {
        self.settings.tour |= 1 << step;
    }

    fn tour_step(&self) -> Option<usize> {
        let s = &self.settings;
        if !s.tips || s.tour_skipped || !s.onboarded || self.dialog.is_some() {
            return None;
        }
        (0..3).find(|i| s.tour & (1 << i) == 0).filter(|&i| match i {
            0 | 1 => self.screen == Screen::Repo,
            _ => self.spaces.len() > 1 && self.screen != Screen::Settings,
        })
    }

    pub fn tour_layer(&mut self, ctx: &egui::Context) {
        // Real actions finish steps.
        if self.tour_rects[1].is_some() {
            self.tour_done(0);
        }
        if self.tour_space != self.selected_space {
            if self.tour_space.is_some() || self.selected_space.is_some() {
                self.tour_done(2);
            }
            self.tour_space = self.selected_space;
        }
        let Some(step) = self.tour_step() else { return };
        let anchor = match step {
            1 => self.tour_rects[1].map(|r| r.right_top() + egui::vec2(14.0, -8.0)),
            0 if self.tour_rects[1].is_none() => self.tour_rects[0].map(|r| r.left_bottom() + egui::vec2(0.0, 10.0)),
            2 => self.tour_rects[2].map(|r| r.right_center() + egui::vec2(12.0, -24.0)),
            _ => None,
        };
        let Some(pos) = anchor else { return };
        let p = self.pal();
        // Inverted card: light on dark themes, dark on light themes.
        let dark = (p.bg.r() as u32 + p.bg.g() as u32 + p.bg.b() as u32) < 380;
        let (bg, fg, sub) = if dark {
            (Color32::from_rgb(0xf4, 0xf5, 0xf7), Color32::from_rgb(0x1a, 0x1c, 0x21), Color32::from_rgb(0x55, 0x5b, 0x66))
        } else {
            (Color32::from_rgb(0x22, 0x25, 0x2b), Color32::from_rgb(0xf2, 0xf3, 0xf5), Color32::from_rgb(0xa0, 0xa6, 0xb2))
        };
        let (title, body) = STEPS[step];
        let mut skip = false;
        egui::Area::new(egui::Id::new("tour_card")).order(egui::Order::Tooltip).fixed_pos(pos).show(ctx, |ui| {
            egui::Frame::none().fill(bg).rounding(10.0).inner_margin(egui::Margin::same(14.0)).shadow(egui::Shadow { offset: egui::vec2(0.0, 6.0), blur: 20.0, spread: 0.0, color: Color32::from_black_alpha(90) }).show(ui, |ui| {
                ui.set_width(260.0);
                ui.label(RichText::new(title).size(13.5).strong().color(fg));
                ui.add_space(4.0);
                ui.label(RichText::new(body).size(12.5).color(sub));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Label::new(RichText::new("Skip tour").size(12.5).color(sub)).sense(Sense::click())).clicked() {
                        skip = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} of 3", step + 1)).size(12.0).color(sub));
                    });
                });
            });
        });
        if skip || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.settings.tour_skipped = true;
        }
    }
}
