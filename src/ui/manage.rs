//! Tags and worktrees windows.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn manage_windows(&mut self, ctx: &egui::Context) {
        match self.dialog {
            Some(Dialog::Tags) => self.tags_window(ctx),
            Some(Dialog::Worktrees) => self.worktrees_window(ctx),
            Some(Dialog::Reword) => self.reword_window(ctx),
            Some(Dialog::Publish) => self.publish_window(ctx),
            Some(Dialog::Switcher) => self.switcher_window(ctx),
            Some(Dialog::PullRequests) => self.pull_requests_window(ctx),
            Some(Dialog::AddMachine) => self.machine_window(ctx),
            None => {}
        }
    }

    fn reword_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut ok = false;
        egui::Window::new("Reword commit")
            .open(&mut open)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.reword_text).desired_rows(4).desired_width(380.0));
                ui.add_space(6.0);
                let enabled = !self.reword_text.trim().is_empty();
                if ui.add_enabled(enabled, egui::Button::new("Reword commit")).clicked() {
                    ok = true;
                }
            });
        if ok {
            self.do_reword();
            self.dialog = None;
        }
        if !open {
            self.dialog = None;
        }
    }

    fn tags_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open = true;
        let mut delete: Option<String> = None;
        let mut push: Option<String> = None;
        let mut create = false;
        egui::Window::new("Tags")
            .open(&mut open)
            .collapsible(false)
            .default_width(420.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("New tag").color(p.muted));
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_tag)
                        .hint_text("Tag name, for example v1.0")
                        .desired_width(f32::INFINITY),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_tag_msg)
                        .hint_text("Message (optional, makes an annotated tag)")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    let target = if self.tag_target.is_empty() {
                        "HEAD".to_string()
                    } else {
                        self.tag_target.chars().take(7).collect()
                    };
                    ui.label(RichText::new(format!("At {target}")).color(p.muted));
                    if !self.tag_target.is_empty() && ui.small_button("Use HEAD").clicked() {
                        self.tag_target.clear();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ok = !self.new_tag.trim().is_empty();
                        if ui.add_enabled(ok, egui::Button::new("Create tag")).clicked() {
                            create = true;
                        }
                    });
                });
                ui.separator();
                if self.tags.is_empty() {
                    ui.label(RichText::new("No tags yet").color(p.muted));
                }
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    for t in &self.tags {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&t.name).strong());
                            ui.label(RichText::new(&t.hash).monospace().size(11.0).color(p.muted));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Delete").clicked() {
                                    delete = Some(t.name.clone());
                                }
                                if ui.add_enabled(self.has_remote, egui::Button::new("Push").small()).clicked() {
                                    push = Some(t.name.clone());
                                }
                                ui.add(egui::Label::new(RichText::new(&t.subject).size(11.5).color(p.muted)).truncate());
                            });
                        });
                    }
                });
            });
        if create {
            self.create_tag();
        }
        if let Some(n) = delete {
            self.delete_tag(&n);
        }
        if let Some(n) = push {
            self.push_tag(ctx, &n);
        }
        if !open {
            self.dialog = None;
        }
    }

    fn worktrees_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open = true;
        let mut switch: Option<PathBuf> = None;
        let mut remove: Option<PathBuf> = None;
        let mut add = false;
        egui::Window::new("Worktrees")
            .open(&mut open)
            .collapsible(false)
            .default_width(460.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                for w in &self.worktrees {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let name = if w.branch.is_empty() { format!("(detached {})", w.head) } else { w.branch.clone() };
                            let tag = if w.main { "  main worktree" } else { "" };
                            ui.label(RichText::new(format!("{name}{tag}")).strong());
                            ui.label(RichText::new(w.path.display().to_string()).size(11.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !w.main && ui.small_button("Remove").clicked() {
                                remove = Some(w.path.clone());
                            }
                            let here = self.repo().as_deref() == Some(w.path.as_path());
                            if ui.add_enabled(!here, egui::Button::new("Open").small()).clicked() {
                                switch = Some(w.path.clone());
                            }
                        });
                    });
                    ui.separator();
                }
                ui.label(RichText::new("Add worktree").color(p.muted));
                ui.add(
                    egui::TextEdit::singleline(&mut self.wt_name)
                        .hint_text("Branch name")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.wt_new_branch, "Create new branch");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ok = !self.wt_name.trim().is_empty();
                        if ui.add_enabled(ok, egui::Button::new("Choose folder and add...")).clicked() {
                            add = true;
                        }
                    });
                });
            });
        if add {
            self.add_worktree();
        }
        if let Some(path) = remove {
            self.remove_worktree(&path);
        }
        if let Some(path) = switch {
            self.open_repo(path);
        }
        if !open {
            self.dialog = None;
        }
    }
}
