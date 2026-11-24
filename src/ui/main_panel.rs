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

            if !changes {
                if let Some(h) = self.selected_commit.as_deref() {
                    let subject = self.commits.iter().chain(self.compare_commits.iter()).find(|c| c.hash == h).map(|c| c.subject.clone()).unwrap_or_default();
                    let short: String = h.chars().take(7).collect();
                    egui::Frame::none().inner_margin(egui::Margin::symmetric(18.0, 12.0)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new(subject).size(14.5).color(p.text));
                        if let Some((author, date, body)) = &self.commit_info {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(author).size(12.5).color(p.text));
                                ui.label(RichText::new(format!("\u{b7} {date}")).size(12.5).color(p.muted));
                                ui.label(RichText::new(short).size(12.5).monospace().color(p.muted));
                            });
                            if !body.is_empty() {
                                ui.add_space(4.0);
                                ui.label(RichText::new(body).size(12.5).color(p.muted));
                            }
                        }
                    });
                    ui.separator();
                }
            }

            if changes && !stash_view {
                let file = self
                    .selected_file
                    .as_ref()
                    .and_then(|path| self.files.iter().find(|f| &f.path == path));
                if let Some(f) = file {
                    let adds = lines.iter().filter(|l| l.kind == Kind::Add).count();
                    let dels = lines.iter().filter(|l| l.kind == Kind::Del).count();
                    let (dir, name) = match f.path.rsplit_once('/') {
                        Some((d, n)) => (d.to_string(), n.to_string()),
                        None => (String::new(), f.path.clone()),
                    };
                    egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 6.0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name).size(13.5));
                            if !dir.is_empty() {
                                ui.label(RichText::new(dir).size(12.0).color(p.muted));
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("\u{2212}{dels}")).size(12.0).color(Color32::from_rgb(248, 81, 73)));
                                ui.label(RichText::new(format!("+{adds}")).size(12.0).color(Color32::from_rgb(63, 185, 80)));
                                ui.add_space(6.0);
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

            let image_file = changes
                .then(|| self.selected_file.clone())
                .flatten()
                .filter(|f| git::is_image(f))
                .filter(|_| self.repo().is_some_and(|r| !git::is_remote(&r)));
            if let (Some(file), Some(repo)) = (image_file, self.repo()) {
                let after = repo.join(&file);
                let removed = !after.exists();
                ui.columns(2, |cols| {
                    cols[0].label(RichText::new("Before").color(p.muted));
                    match &self.img_before {
                        Some(b) => {
                            cols[0].add(egui::Image::new(format!("file://{}", b.display())).max_width(cols[0].available_width()).max_height(420.0));
                        }
                        None => {
                            cols[0].label(RichText::new("New file").color(p.muted));
                        }
                    }
                    cols[1].label(RichText::new("After").color(p.muted));
                    if removed {
                        cols[1].label(RichText::new("Deleted").color(p.muted));
                    } else {
                        cols[1].add(egui::Image::new(format!("file://{}", after.display())).max_width(cols[1].available_width()).max_height(420.0));
                    }
                });
            } else if lines.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    let (msg, hint) = match self.tab {
                        Tab::Changes if self.files.is_empty() => ("No local changes", ""),
                        Tab::Changes if self.selected_file.is_none() => ("Select a file to inspect", "Choose a file to inspect its diff. Check the whole files you want to commit."),
                        Tab::Changes => ("No diff to show", ""),
                        Tab::History => ("Select a commit", ""),
                    };
                    ui.label(RichText::new(msg).size(15.0).color(p.text));
                    if !hint.is_empty() {
                        ui.label(RichText::new(hint).size(12.5).color(p.muted));
                    }
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
