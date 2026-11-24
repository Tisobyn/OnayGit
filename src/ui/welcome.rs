//! First-run welcome window.
use crate::prelude::*;

impl App {
    /// First-run window. Closing it marks the welcome as seen.
    pub fn onboarding_window(&mut self, ctx: &egui::Context) {
        if self.settings.onboarded {
            return;
        }
        let p = self.pal();
        let mut done = false;
        let close = modal(ctx, &p, "welcome", 480.0, Place::Center, "Welcome to OnayGit", "A simple Git client. Here is how it works:", |ui| {
            for (ic, title, text) in [
                ("folder", "Add a repository", "Open from disk, clone, or scan a folder."),
                ("check", "Review and commit", "Tick files, write a summary, press Commit."),
                ("push", "Sync", "Fetch, pull and push with the top button in the left rail."),
                ("history", "Explore history", "Graph, compare branches, cherry-pick and more."),
            ] {
                ui.horizontal(|ui| {
                    icon(ui, ic, 20.0, p.muted);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(RichText::new(title).size(13.5).color(p.text));
                        ui.label(RichText::new(text).size(12.5).color(p.muted));
                    });
                });
                ui.add_space(8.0);
            }
            button_row(ui, |ui| {
                if primary_button(ui, "Get started", true).clicked() {
                    done = true;
                }
            });
        });
        done |= close;
        if done {
            self.settings.onboarded = true;
        }
    }
}
