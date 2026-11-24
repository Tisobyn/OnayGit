//! Tab bar with the open repositories.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    /// Cmd+W closes the tab, Cmd+T opens the repository list.
    pub fn tab_shortcuts(&mut self, ctx: &egui::Context) {
        if self.recording.is_some() && (self.screen != Screen::Settings || self.section != Section::Keybinds) {
            self.recording = None;
        }
        if self.recording.is_some() {
            return;
        }
        use crate::keybinds::hit;
        let ov = self.keybinds.clone();
        let h = |id: &str| ctx.input(|i| hit(i, id, &ov));
        let (close, new_tab, quick, prefs) = (h("close-tab"), h("new-tab"), h("quick-open"), h("settings"));
        if quick {
            self.switcher_text.clear();
            self.switcher_sel = 0;
            self.dialog = Some(Dialog::Switcher);
        }
        if prefs {
            self.open_settings();
        }
        if h("open") {
            if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                self.open_repo(d);
            }
        }
        if h("refresh") && self.screen == Screen::Repo {
            if let Some(repo) = self.repo() {
                if self.has_remote && self.busy.is_none() {
                    self.spawn_in(ctx, repo, "Fetching", &["fetch", "--all", "--prune"]);
                }
            }
        }
        let n = self.tabs.len();
        if n > 0 {
            let cur = self.repo().and_then(|r| self.tabs.iter().position(|t| *t == r));
            let mut go: Option<usize> = None;
            let typing = ctx.wants_keyboard_input();
            if h("previous-tab") && !typing {
                go = Some(cur.map_or(n - 1, |c| (c + n - 1) % n));
            }
            if h("next-tab") && !typing {
                go = Some(cur.map_or(0, |c| (c + 1) % n));
            }
            for k in 0..9 {
                if h(&format!("tab-{}", k + 1)) && k < n {
                    go = Some(k);
                }
            }
            if let Some(i) = go {
                self.activate_tab(i);
            }
        }
        let m = self.spaces.len();
        if m > 0 {
            // Positions: 0 is all repositories, 1.. are the spaces.
            let cur = self.selected_space.map_or(0, |s| s + 1);
            let mut to = None;
            if h("previous-space") {
                to = Some((cur + m) % (m + 1));
            }
            if h("next-space") {
                to = Some((cur + 1) % (m + 1));
            }
            if let Some(t) = to {
                self.selected_space = t.checked_sub(1);
                self.screen = Screen::Home;
            }
        }
        if self.screen == Screen::Repo {
            if h("find") {
                self.filter_open = true;
            }
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
