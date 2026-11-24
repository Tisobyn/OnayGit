//! Tab bar with the open repositories.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    /// Cmd+W closes the tab, Cmd+T opens the repository list.
    pub fn tab_shortcuts(&mut self, ctx: &egui::Context) {
        let (close, new_tab, quick, prefs, tab1, tab2) = ctx.input(|i| {
            let c = i.modifiers.command;
            (
                c && i.key_pressed(egui::Key::W),
                c && i.key_pressed(egui::Key::T),
                c && i.key_pressed(egui::Key::P),
                c && i.key_pressed(egui::Key::Comma),
                c && i.key_pressed(egui::Key::Num1),
                c && i.key_pressed(egui::Key::Num2),
            )
        });
        if quick {
            self.switcher_text.clear();
            self.dialog = Some(Dialog::Switcher);
        }
        if prefs {
            self.open_settings();
        }
        if self.screen == Screen::Repo {
            if tab1 {
                self.tab = Tab::Changes;
            }
            if tab2 {
                self.tab = Tab::History;
            }
        }
        if new_tab {
            self.screen = Screen::Home;
        }
        if close && self.screen == Screen::Repo {
            if let Some(r) = self.repo() {
                if let Some(i) = self.tabs.iter().position(|t| *t == r) {
                    self.close_tab(i);
                }
            }
        }
    }
}
