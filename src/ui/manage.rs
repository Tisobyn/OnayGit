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
        let p = self.pal();
        let mut ok = false;
        let mut cancel = false;
        let close = modal(ctx, &p, "reword", 480.0, Place::Center, "Reword commit", "Edit the message. This rewrites history.", |ui| {
            dialog_area(ui, &mut self.reword_text, "Commit message", 5);
            button_row(ui, |ui| {
                if primary_button(ui, "Reword commit", !self.reword_text.trim().is_empty()).clicked() {
                    ok = true;
                }
                if plain_button(ui, &p, "Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if ok {
            self.do_reword();
            self.dialog = None;
        }
        if close || cancel {
            self.dialog = None;
        }
    }

    fn tags_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut delete: Option<String> = None;
        let mut push: Option<String> = None;
        let mut create = false;
        let close = modal(ctx, &p, "tags", 540.0, Place::Center, "Tags", "", |ui| {
            dialog_field(ui, &p, "New tag", &mut self.new_tag, "Tag name, for example v1.0");
            ui.add_space(4.0);
            framed(ui, &p, egui::TextEdit::singleline(&mut self.new_tag_msg).hint_text("Message (optional, makes an annotated tag)").desired_width(f32::INFINITY).margin(egui::vec2(12.0, 10.0)));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let target = if self.tag_target.is_empty() { "HEAD".to_string() } else { self.tag_target.chars().take(7).collect() };
                ui.label(RichText::new(format!("At {target}")).size(12.5).color(p.muted));
                if !self.tag_target.is_empty() && ui.small_button("Use HEAD").clicked() {
                    self.tag_target.clear();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "Create tag", !self.new_tag.trim().is_empty()).clicked() {
                        create = true;
                    }
                });
            });
            dialog_section(ui, &p, "All tags");
            if self.tags.is_empty() {
                ui.label(RichText::new("No tags yet").color(p.muted));
            }
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                for t in &self.tags {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&t.name).size(13.5).color(p.text));
                        ui.label(RichText::new(&t.hash).monospace().size(11.5).color(p.muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Delete").clicked() {
                                delete = Some(t.name.clone());
                            }
                            if ui.add_enabled(self.has_remote, egui::Button::new("Push").small()).clicked() {
                                push = Some(t.name.clone());
                            }
                            ui.add(egui::Label::new(RichText::new(&t.subject).size(12.0).color(p.muted)).truncate());
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
        if close {
            self.dialog = None;
        }
    }

    fn worktrees_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut switch: Option<PathBuf> = None;
        let mut remove: Option<PathBuf> = None;
        let mut add = false;
        let close = modal(ctx, &p, "worktrees", 540.0, Place::Center, "Worktrees", "Several branches checked out at the same time.", |ui| {
            for (i, w) in self.worktrees.iter().enumerate() {
                if i > 0 {
                    ui.separator();
                }
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_max_width(ui.available_width() - 120.0);
                        let name = if w.branch.is_empty() { format!("(detached {})", w.head) } else { w.branch.clone() };
                        let tag = if w.main { "  main worktree" } else { "" };
                        ui.label(RichText::new(format!("{name}{tag}")).size(13.5).color(p.text));
                        ui.label(RichText::new(w.path.display().to_string()).size(12.0).color(p.muted));
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
            }
            dialog_section(ui, &p, "Add worktree");
            framed(ui, &p, egui::TextEdit::singleline(&mut self.wt_name).hint_text("Branch name").desired_width(f32::INFINITY).margin(egui::vec2(12.0, 10.0)));
            ui.add_space(4.0);
            ui.checkbox(&mut self.wt_new_branch, "Create new branch");
            button_row(ui, |ui| {
                if primary_button(ui, "Choose folder and add...", !self.wt_name.trim().is_empty()).clicked() {
                    add = true;
                }
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
        if close {
            self.dialog = None;
        }
    }

}
