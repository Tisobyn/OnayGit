//! Publish repository and pull request windows.
use crate::prelude::*;

impl App {
    pub fn open_publish(&mut self) {
        if let Some(r) = self.repo() {
            if self.publish_name.is_empty() {
                self.publish_name = repo_name(&r);
            }
        }
        self.dialog = Some(Dialog::Publish);
    }

    pub fn open_pull_requests(&mut self, ctx: &egui::Context) {
        self.prepare_pr_form();
        self.prs.clear();
        self.dialog = Some(Dialog::PullRequests);
        self.load_prs(ctx);
    }

    pub fn publish_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut go = false;
        let mut cancel = false;
        let close = modal(ctx, &p, "publish", 480.0, Place::Center, "Publish repository", "Create it on GitHub or GitLab and push.", |ui| {
            if self.accounts.is_empty() {
                ui.label(RichText::new("Add a GitHub or GitLab account first (Settings > Accounts).").color(p.muted));
                return;
            }
            if self.publish_account >= self.accounts.len() {
                self.publish_account = 0;
            }
            ui.label(RichText::new("Account").size(13.0).strong().color(p.text));
            egui::ComboBox::from_id_salt("publish_acc")
                .width(ui.available_width() - 8.0)
                .selected_text(format!("{} ({})", self.accounts[self.publish_account].user, self.accounts[self.publish_account].host))
                .show_ui(ui, |ui| {
                    for (i, a) in self.accounts.iter().enumerate() {
                        ui.selectable_value(&mut self.publish_account, i, format!("{} ({})", a.user, a.host));
                    }
                });
            dialog_field(ui, &p, "Name", &mut self.publish_name, "");
            dialog_field(ui, &p, "Description", &mut self.publish_desc, "");
            ui.add_space(8.0);
            ui.checkbox(&mut self.publish_private, "Keep this code private");
            if !self.status_msg.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new(&self.status_msg).size(12.5).color(p.muted));
            }
            let busy = self.task_running(T_PUBLISH);
            button_row(ui, |ui| {
                if primary_button(ui, "Publish repository", !busy && !self.publish_name.trim().is_empty()).clicked() {
                    go = true;
                }
                if plain_button(ui, &p, "Cancel").clicked() {
                    cancel = true;
                }
                if busy {
                    ui.spinner();
                }
            });
        });
        if go {
            self.publish(ctx);
        }
        if close || cancel {
            self.dialog = None;
        }
    }

    pub fn pull_requests_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut create = false;
        let mut reload = false;
        let mut url: Option<String> = None;
        let close = modal(ctx, &p, "pull_requests", 580.0, Place::Center, "Pull requests", "", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Open").size(13.0).strong().color(p.text));
                if ui.small_button("Reload").clicked() {
                    reload = true;
                }
                if self.task_running(T_PRS) {
                    ui.spinner();
                }
            });
            if !self.pr_error.is_empty() {
                ui.label(RichText::new(&self.pr_error).color(p.muted));
            } else if self.prs.is_empty() && !self.task_running(T_PRS) {
                ui.label(RichText::new("No open pull requests.").color(p.muted));
            }
            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                for pr in &self.prs {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(format!("#{} {}", pr.number, pr.title)).size(13.5).color(p.text));
                            ui.label(RichText::new(format!("{} > {} by {}", pr.head, pr.base, pr.author)).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Open in browser").clicked() {
                                url = Some(pr.url.clone());
                            }
                        });
                    });
                }
            });
            dialog_section(ui, &p, &format!("New pull request from {}", self.branch));
            dialog_field(ui, &p, "Base", &mut self.pr_base, "main");
            dialog_field(ui, &p, "Title", &mut self.pr_title, "");
            ui.add_space(6.0);
            dialog_area(ui, &mut self.pr_body, "Description", 3);
            let busy = self.task_running(T_NEW_PR);
            let ok = !busy && !self.pr_title.trim().is_empty() && !self.pr_base.trim().is_empty();
            button_row(ui, |ui| {
                if primary_button(ui, "Push and create", ok).clicked() {
                    create = true;
                }
                if busy {
                    ui.spinner();
                }
            });
        });
        if let Some(u) = url {
            open_url(&u);
        }
        if reload {
            self.load_prs(ctx);
        }
        if create {
            self.create_pr(ctx);
        }
        if close {
            self.dialog = None;
        }
    }

    pub fn machine_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut add = false;
        let busy = self.pending_machine.is_some();
        let close = modal(ctx, &p, "add_machine", 540.0, Place::Center, "Add SSH machine", "One connection for all your repositories.", |ui| {
            dialog_field(ui, &p, "Machine name", &mut self.machine_name, "Devbox");
            dialog_field(ui, &p, "SSH host", &mut self.machine_host, "devbox or host.example.com");
            ui.add_space(8.0);
            egui::CollapsingHeader::new(RichText::new("Authentication and connection options").size(13.0).color(p.text)).show(ui, |ui| {
                ui.label(RichText::new("Uses key login through your SSH config and agent. Password prompts are not supported.").size(12.5).color(p.muted));
            });
            ui.add_space(8.0);
            ui.label(RichText::new("Your SSH config, keys and agent work automatically. OnayGit needs key login and never asks for a password.").size(12.5).color(p.muted));
            if !self.status_msg.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new(&self.status_msg).size(12.5).color(Color32::from_rgb(248, 81, 73)));
            }
            ui.add_space(16.0);
            let w = ui.available_width();
            ui.allocate_ui_with_layout(egui::vec2(w, 32.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_button(ui, "Save and connect", !busy && !self.machine_host.trim().is_empty()).clicked() {
                    add = true;
                }
                if plain_button(ui, &p, "Cancel").clicked() {
                    self.dialog = None;
                }
                if busy {
                    ui.spinner();
                }
            });
        });
        if add {
            self.status_msg.clear();
            self.add_machine(ctx);
        }
        if close {
            self.dialog = None;
        }
    }
}

impl App {
    /// Main area of the home screen for one machine.
    pub fn machine_view(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, mi: usize, p: &Pal, open: &mut Option<PathBuf>) {
        let Some(m) = self.machines.get(mi).cloned() else {
            self.selected_machine = None;
            return;
        };
        let mut scan = false;
        let mut remove = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new(&m.name).size(20.0));
            ui.label(RichText::new(&m.host).color(p.muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Remove machine").clicked() {
                    remove = true;
                }
                let label = format!("{T_SCAN}{}", m.host);
                if self.task_running(&label) {
                    ui.spinner();
                } else if ui.button("Scan for repositories").clicked() {
                    scan = true;
                }
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.machine_path).hint_text("/home/me/project").desired_width(320.0));
            if ui.button("Open path").clicked() {
                let path = self.machine_path.trim();
                if path.starts_with('/') {
                    *open = Some(git::remote_path(&m.host, path));
                } else {
                    self.status_msg = "Type an absolute path, for example /home/me/project".into();
                }
            }
        });
        ui.add_space(10.0);
        let saved: Vec<PathBuf> = self
            .repos
            .iter()
            .filter(|r| git::split_remote(r).is_some_and(|(h, _)| h == m.host))
            .cloned()
            .collect();
        let found = self.machine_repos.get(&m.host).cloned().unwrap_or_default();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for r in &saved {
                if rail_item(ui, "folder", &repo_name(r), false, p).clicked() {
                    *open = Some(r.clone());
                }
            }
            if !found.is_empty() {
                ui.add_space(8.0);
                rail_label(ui, "FOUND", p);
                for path in &found {
                    if rail_item(ui, "folder", path, false, p).clicked() {
                        *open = Some(git::remote_path(&m.host, path));
                    }
                }
            }
        });
        if scan {
            self.scan_machine(ctx, &m.host);
        }
        if remove {
            self.remove_machine(mi);
        }
    }
}
