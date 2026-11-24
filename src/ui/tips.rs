//! Welcome window and guided tips.
#[allow(unused_imports)]
use crate::prelude::*;

const TIPS: [&str; 9] = [
    "Click the checkbox next to a file to include it in the commit.",
    "Click the gutter of a changed line in the diff to stage just that line.",
    "Right-click a commit in History to cherry-pick, revert, reset or reword it.",
    "Use Stash to put your changes aside, then Pop to bring them back.",
    "Open several repositories as tabs. Cmd+T opens the list, Cmd+W closes a tab.",
    "Right-click a file to ignore it, its folder or its extension.",
    "Compare to branch in History shows what is ahead or behind.",
    "Settings > Repository > Appearance gives each repository a color and icon.",
    "Amend lets you fix the last commit. Undo puts its changes back.",
];

/// A small tip with a "Next" button. Does nothing when tips are off.
pub fn tip_box(ui: &mut egui::Ui, p: &Pal, enabled: bool, tip: &mut usize) {
    if !enabled {
        return;
    }
    ui.add_space(14.0);
    egui::Frame::none()
        .fill(p.hover)
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, "sparkle", 16.0, accent());
                ui.add(egui::Label::new(RichText::new(TIPS[*tip % TIPS.len()]).size(12.5).color(p.text)).wrap());
            });
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                if ui.small_button("Next tip").clicked() {
                    *tip = (*tip + 1) % TIPS.len();
                }
            });
        });
}

impl App {
    /// First-run window. Closing it marks the welcome as seen.
    pub fn onboarding_window(&mut self, ctx: &egui::Context) {
        if self.settings.onboarded {
            return;
        }
        let p = self.pal();
        let mut done = false;
        egui::Window::new("Welcome to OnayGit")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.label(RichText::new("A simple Git client. Here is how it works:").color(p.muted));
                ui.add_space(10.0);
                for (ic, title, text) in [
                    ("folder", "Add a repository", "Open from disk, clone, or scan a folder."),
                    ("check", "Review and commit", "Tick files, write a summary, press Commit."),
                    ("push", "Sync", "Fetch, pull and push from the top bar."),
                    ("history", "Explore history", "Graph, compare branches, cherry-pick and more."),
                ] {
                    ui.horizontal(|ui| {
                        icon(ui, ic, 22.0, accent());
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).strong());
                            ui.label(RichText::new(text).size(12.0).color(p.muted));
                        });
                    });
                    ui.add_space(6.0);
                }
                ui.checkbox(&mut self.settings.tips, "Show guided tips");
                ui.add_space(8.0);
                let btn = egui::Button::new(RichText::new("Get started").color(Color32::WHITE).strong()).fill(accent());
                if ui.add(btn).clicked() {
                    done = true;
                }
            });
        if done {
            self.settings.onboarded = true;
        }
    }
}
