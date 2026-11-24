//! Publish repository and pull request windows.
#[allow(unused_imports)]
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
        let mut open = true;
        let mut go = false;
        egui::Window::new("Publish repository")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if self.accounts.is_empty() {
                    ui.label("Add a GitHub or GitLab account first (Settings > Accounts).");
                    return;
                }
                if self.publish_account >= self.accounts.len() {
                    self.publish_account = 0;
                }
                ui.label("Account");
                egui::ComboBox::from_id_salt("publish_acc")
                    .selected_text(format!("{} ({})", self.accounts[self.publish_account].user, self.accounts[self.publish_account].host))
                    .show_ui(ui, |ui| {
                        for (i, a) in self.accounts.iter().enumerate() {
                            ui.selectable_value(&mut self.publish_account, i, format!("{} ({})", a.user, a.host));
                        }
                    });
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(&mut self.publish_name).desired_width(340.0));
                ui.label("Description");
                ui.add(egui::TextEdit::singleline(&mut self.publish_desc).desired_width(340.0));
                ui.checkbox(&mut self.publish_private, "Keep this code private");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let busy = self.task_running(T_PUBLISH);
                    if ui.add_enabled(!busy && !self.publish_name.trim().is_empty(), egui::Button::new("Publish repository")).clicked() {
                        go = true;
                    }
                    if busy {
                        ui.spinner();
                    }
                });
                if !self.status_msg.is_empty() {
                    ui.label(RichText::new(&self.status_msg).color(p.muted));
                }
            });
        if go {
            self.publish(ctx);
        }
        if !open {
            self.dialog = None;
        }
    }

    pub fn pull_requests_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open = true;
        let mut create = false;
        let mut reload = false;
        let mut url: Option<String> = None;
        egui::Window::new("Pull requests")
            .open(&mut open)
            .collapsible(false)
            .default_width(520.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Open").size(15.0));
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
                        ui.horizontal(|ui| {
                            ui.label(format!("#{} {}", pr.number, pr.title));
                            ui.label(RichText::new(format!("{} > {} by {}", pr.head, pr.base, pr.author)).size(11.5).color(p.muted));
                            if ui.small_button("Open in browser").clicked() {
                                url = Some(pr.url.clone());
                            }
                        });
                    }
                });
                ui.separator();
                ui.label(RichText::new(format!("New pull request from {}", self.branch)).size(15.0));
                ui.horizontal(|ui| {
                    ui.label("Base");
                    ui.add(egui::TextEdit::singleline(&mut self.pr_base).desired_width(120.0));
                });
                ui.add(egui::TextEdit::singleline(&mut self.pr_title).hint_text("Title").desired_width(f32::INFINITY));
                ui.add(egui::TextEdit::multiline(&mut self.pr_body).hint_text("Description").desired_rows(3).desired_width(f32::INFINITY));
                ui.horizontal(|ui| {
                    let busy = self.task_running(T_NEW_PR);
                    let ok = !busy && !self.pr_title.trim().is_empty() && !self.pr_base.trim().is_empty();
                    if ui.add_enabled(ok, egui::Button::new("Push and create")).clicked() {
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
        if !open {
            self.dialog = None;
        }
    }

    pub fn machine_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut add = false;
        egui::Window::new("Add machine")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Name (optional)");
                ui.add(egui::TextEdit::singleline(&mut self.machine_name).desired_width(320.0));
                ui.label("SSH host");
                ui.add(egui::TextEdit::singleline(&mut self.machine_host).hint_text("me@server").desired_width(320.0));
                ui.label(RichText::new("Needs key login (no password prompt).").size(11.5));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let busy = self.pending_machine.is_some();
                    if ui.add_enabled(!busy && !self.machine_host.trim().is_empty(), egui::Button::new("Test and add")).clicked() {
                        add = true;
                    }
                    if busy {
                        ui.spinner();
                    }
                });
                if !self.status_msg.is_empty() {
                    ui.label(&self.status_msg);
                }
            });
        if add {
            self.status_msg.clear();
            self.add_machine(ctx);
        }
        if !open {
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
