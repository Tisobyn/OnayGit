//! Settings pages that belong to the open repository: Remotes and Appearance.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn remotes_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_title(ui, "Remotes", p);
        let Some(repo) = self.repo() else {
            ui.label(RichText::new("Open a repository first.").color(p.muted));
            return;
        };
        ui.label(RichText::new(format!("Remotes of {}", repo_name(&repo))).color(p.muted));
        ui.label(RichText::new("A remote is the online copy of this repository, for example on GitHub. Push and fetch use it. \"origin\" is the usual name of the main one.").size(12.5).color(p.muted));
        ui.add_space(8.0);

        let mut remove: Option<String> = None;
        let mut save_url: Option<(String, String)> = None;
        let mut rename: Option<(String, String)> = None;
        let list = self.remotes.clone();
        if list.is_empty() {
            ui.label(RichText::new("No remotes yet").color(p.muted));
        }
        for r in &list {
            ui.horizontal(|ui| {
                match &mut self.remote_rename {
                    Some((old, text)) if *old == r.name => {
                        framed(ui, &p, egui::TextEdit::singleline(text).desired_width(120.0));
                        if ui.small_button("OK").clicked() {
                            rename = Some((old.clone(), text.clone()));
                        }
                        if ui.small_button("Cancel").clicked() {
                            *old = String::new();
                        }
                    }
                    _ => {
                        ui.label(RichText::new(&r.name).strong());
                    }
                }
                match &mut self.remote_edit {
                    Some((name, url)) if *name == r.name => {
                        framed(ui, &p, egui::TextEdit::singleline(url).desired_width(260.0));
                        if ui.small_button("Save").clicked() {
                            save_url = Some((name.clone(), url.clone()));
                        }
                        if ui.small_button("Cancel").clicked() {
                            *name = String::new();
                        }
                    }
                    _ => {
                        ui.label(RichText::new(&r.url).color(p.muted));
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Remove").clicked() {
                        remove = Some(r.name.clone());
                    }
                    if ui.small_button("Rename").clicked() {
                        self.remote_rename = Some((r.name.clone(), r.name.clone()));
                    }
                    if ui.small_button("Edit URL").clicked() {
                        self.remote_edit = Some((r.name.clone(), r.url.clone()));
                    }
                });
            });
            ui.separator();
        }
        if let Some((n, u)) = save_url {
            self.set_remote_url(&n, &u);
            self.remote_edit = None;
        }
        if let Some((o, n)) = rename {
            self.rename_remote(&o, &n);
            self.remote_rename = None;
        }
        if let Some(n) = remove {
            self.remove_remote(&n);
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Add remote").color(p.muted));
        framed(ui, &p, egui::TextEdit::singleline(&mut self.new_remote_name).hint_text("Name, for example origin").desired_width(f32::INFINITY));
        framed(ui, &p, egui::TextEdit::singleline(&mut self.new_remote_url).hint_text("URL").desired_width(f32::INFINITY));
        ui.add_space(6.0);
        let ok = !self.new_remote_name.trim().is_empty() && !self.new_remote_url.trim().is_empty();
        if ui.add_enabled(ok, egui::Button::new("Add remote")).clicked() {
            self.add_remote();
        }
    }

}
