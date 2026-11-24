//! Right side: diff of the selected file or commit.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn main_panel(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open_editor = false;
        let mut reveal = false;
        let mut stage_selected = false;
        let mut resolve: Option<ConflictAct> = None;
        let mut mode: Option<bool> = None;
        let mut events: Vec<DiffEvent> = Vec::new();

        egui::CentralPanel::default().frame(egui::Frame::none().fill(p.bg)).show(ctx, |ui| {
            let changes = self.tab == Tab::Changes;
            let stash_view = changes && self.selected_stash.is_some();
            let conflicted = self
                .selected_file
                .as_ref()
                .and_then(|p| self.files.iter().find(|f| &f.path == p))
                .is_some_and(|f| f.conflicted());
            let lines: &[DiffLine] = if stash_view {
                &self.stash_diff
            } else if changes {
                &self.diff
            } else {
                &self.commit_diff
            };

            if stash_view {
                egui::Frame::none().inner_margin(egui::Margin::symmetric(10.0, 6.0)).show(ui, |ui| {
                    ui.label(RichText::new("Stashed changes (read only)").size(13.0).color(p.muted));
                });
                ui.separator();
            }

            if changes && !stash_view {
                let file = self
                    .selected_file
                    .as_ref()
                    .and_then(|path| self.files.iter().find(|f| &f.path == path));
                if let Some(f) = file {
                    egui::Frame::none().inner_margin(egui::Margin::symmetric(10.0, 6.0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&f.path).size(13.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Show in Finder").clicked() {
                                    reveal = true;
                                }
                                if ui.small_button("Open in editor").clicked() {
                                    open_editor = true;
                                }
                                if f.conflicted() {
                                    if ui.small_button("Mark resolved").clicked() {
                                        resolve = Some(ConflictAct::Resolved);
                                    }
                                    if ui.small_button("Use theirs").clicked() {
                                        resolve = Some(ConflictAct::Theirs);
                                    }
                                    if ui.small_button("Use ours").clicked() {
                                        resolve = Some(ConflictAct::Ours);
                                    }
                                    ui.label(RichText::new("Conflict").size(12.0).color(Color32::from_rgb(248, 81, 73)));
                                    return;
                                }
                                ui.separator();
                                let n = self.sel_lines.len();
                                if n > 0 {
                                    let verb = if self.diff_staged { "Unstage" } else { "Stage" };
                                    let s = if n == 1 { "" } else { "s" };
                                    if ui.small_button(format!("{verb} {n} line{s}")).clicked() {
                                        stage_selected = true;
                                    }
                                }
                                if f.staged() && f.work != ' ' {
                                    let mut st = self.diff_staged;
                                    ui.selectable_value(&mut st, true, "Staged");
                                    ui.selectable_value(&mut st, false, "Unstaged");
                                    if st != self.diff_staged {
                                        mode = Some(st);
                                    }
                                } else {
                                    let label = if self.diff_staged { "Staged changes" } else { "Unstaged changes" };
                                    ui.label(RichText::new(label).size(12.0).color(p.muted));
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }

            if lines.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    let msg = match self.tab {
                        Tab::Changes if self.files.is_empty() => "No local changes",
                        Tab::Changes => "No diff to show",
                        Tab::History => "Select a commit",
                    };
                    ui.label(RichText::new(msg).size(17.0).color(p.muted));
                    ui.allocate_ui(egui::vec2(380.0, 0.0), |ui| tip_box(ui, &p, self.settings.tips, &mut self.tip));
                });
            } else if changes && !stash_view && !conflicted {
                let label = if self.diff_staged { "Unstage hunk" } else { "Stage hunk" };
                events = draw_diff(ui, lines, &p, self.settings.diff_font, Some(&self.sel_lines), label);
            } else {
                draw_diff(ui, lines, &p, self.settings.diff_font, None, "");
            }
        });

        if let Some(a) = resolve {
            if let Some(path) = self.selected_file.clone() {
                self.resolve_conflict(&path, a);
            }
        }
        let mut hunk: Option<usize> = None;
        for e in events {
            match e {
                DiffEvent::ToggleLine(i) => {
                    if !self.sel_lines.remove(&i) {
                        self.sel_lines.insert(i);
                    }
                }
                DiffEvent::Hunk(h) => hunk = Some(h),
            }
        }
        if let Some(m) = mode {
            self.diff_staged = m;
            self.sel_lines.clear();
            self.refresh();
        }
        if stage_selected {
            self.apply_lines(None);
        }
        if let Some(h) = hunk {
            self.apply_lines(Some(h));
        }

        if open_editor || reveal {
            if let (Some(repo), Some(file)) = (self.repo(), self.selected_file.clone()) {
                let path = repo.join(file);
                if git::is_remote(&repo) {
                    if reveal {
                        self.status_msg = "Show in Finder is not available on other machines".into();
                    } else if let Some((host, rp)) = git::split_remote(&path) {
                        let cmd = format!("{} --remote ssh-remote+{} {}", self.settings.editor, host, git::quote(&rp));
                        let _ = std::process::Command::new("sh").arg("-c").arg(cmd).spawn();
                    }
                } else if reveal {
                    git::reveal(&path);
                } else if let Err(e) = git::open_editor(&self.settings.editor, &path) {
                    self.status_msg = e;
                }
            }
        }
    }
}
