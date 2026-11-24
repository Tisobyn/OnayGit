#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn home(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open: Option<PathBuf> = None;
        let mut remove: Option<usize> = None;
        let mut scan: Option<PathBuf> = None;
        let mut goto_settings = false;
        let mut create = false;
        let mut open_disk = false;
        let mut del_space: Option<usize> = None;
        let mut new_space = false;
        let mut toggle: Option<(usize, PathBuf)> = None;

        egui::SidePanel::left("home_rail")
            .exact_width(260.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.home).inner_margin(egui::Margin::symmetric(14.0, 22.0)))
            .show(ctx, |ui| {
                rail_label(ui, "MACHINES", &p);
                if rail_item(ui, "home", "This Mac", self.selected_machine.is_none(), &p).clicked() {
                    self.selected_machine = None;
                }
                for i in 0..self.machines.len() {
                    let name = self.machines[i].name.clone();
                    if rail_item(ui, "network", &name, self.selected_machine == Some(i), &p).clicked() {
                        self.selected_machine = Some(i);
                    }
                }
                ui.add_space(6.0);
                if rail_item(ui, "add", "Add machine...", false, &p).clicked() {
                    self.status_msg.clear();
                    self.dialog = Some(Dialog::AddMachine);
                }
                ui.add_space(22.0);
                rail_label(ui, "SPACES", &p);
                if rail_item(ui, "layers", "All repositories", self.selected_space.is_none() && self.selected_machine.is_none(), &p).clicked() {
                    self.selected_space = None;
                    self.selected_machine = None;
                }
                for i in 0..self.spaces.len() {
                    let name = self.spaces[i].name.clone();
                    let r = rail_item(ui, "folder", &name, self.selected_space == Some(i) && self.selected_machine.is_none(), &p);
                    if r.clicked() {
                        self.selected_space = Some(i);
                        self.selected_machine = None;
                    }
                    r.context_menu(|ui| {
                        if ui.button("Delete space").clicked() {
                            del_space = Some(i);
                            ui.close_menu();
                        }
                    });
                }
                ui.horizontal(|ui| {
                    let w = ui.available_width() - 40.0;
                    let r = ui.add(egui::TextEdit::singleline(&mut self.space_name).hint_text("New space").desired_width(w));
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.small_button("Add").clicked() || enter {
                        new_space = true;
                    }
                });
                ui.add_space(22.0);
                rail_label(ui, "QUICK ACTIONS", &p);
                if rail_item(ui, "add", "Create repository", false, &p).clicked() {
                    create = true;
                }
                if rail_item(ui, "pull", "Clone repository", false, &p).clicked() {
                    self.clone_open = true;
                }
                if rail_item(ui, "folder", "Open from disk", false, &p).clicked() {
                    open_disk = true;
                }
                if rail_item(ui, "sync", "Scan repositories", false, &p).clicked() {
                    if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                        scan = Some(d);
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if rail_item(ui, "settings", "Settings", false, &p).clicked() {
                        goto_settings = true;
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.home).inner_margin(egui::Margin::symmetric(30.0, 22.0)))
            .show(ctx, |ui| {
                if let Some(mi) = self.selected_machine {
                    self.machine_view(ctx, ui, mi, &p, &mut open);
                    return;
                }
                ui.horizontal(|ui| {
                    let w = ui.available_width() - 130.0;
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text(format!("Search {} repositories", self.repos.len()))
                            .desired_width(w)
                            .margin(egui::vec2(10.0, 8.0)),
                    );
                    ui.selectable_value(&mut self.grid, true, "Grid");
                    ui.selectable_value(&mut self.grid, false, "List");
                });
                ui.add_space(14.0);
                if rail_item(ui, "add", "Add folder...", false, &p).clicked() {
                    if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                        scan = Some(d);
                    }
                }
                ui.add_space(14.0);

                let q = self.search.to_lowercase();
                let shown: Vec<usize> = (0..self.repos.len())
                    .filter(|&i| !git::is_remote(&self.repos[i]))
                    .filter(|&i| match self.selected_space.and_then(|s| self.spaces.get(s)) {
                        Some(s) => s.repos.contains(&self.repos[i]),
                        None => true,
                    })
                    .filter(|&i| q.is_empty() || repo_name(&self.repos[i]).to_lowercase().contains(&q))
                    .collect();

                if self.repos.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("No repositories yet").size(16.0).color(p.muted));
                        ui.label(
                            RichText::new("Use Open from disk, Clone, or Scan repositories.")
                                .color(p.muted),
                        );
                        ui.allocate_ui(egui::vec2(380.0, 0.0), |ui| tip_box(ui, &p, self.settings.tips, &mut self.tip));
                    });
                    return;
                }

                // Group header
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, p.hover);
                }
                let folder = egui::Rect::from_center_size(rect.min + egui::vec2(28.0, 26.0), egui::vec2(28.0, 28.0));
                paint_icon(ui, "folder", folder, accent());
                ui.painter().text(
                    rect.min + egui::vec2(58.0, 17.0),
                    Align2::LEFT_CENTER,
                    "Other repositories",
                    FontId::proportional(15.0),
                    p.text,
                );
                ui.painter().text(
                    rect.min + egui::vec2(58.0, 36.0),
                    Align2::LEFT_CENTER,
                    format!("{} repositories", shown.len()),
                    FontId::proportional(12.5),
                    p.muted,
                );
                if resp.clicked() {
                    self.group_open = !self.group_open;
                }

                if self.group_open {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        if self.grid {
                            ui.horizontal_wrapped(|ui| {
                                for &i in &shown {
                                    let r = repo_card(ui, &self.repos[i], &p, &self.style_of(&self.repos[i]));
                                    if r.clicked() {
                                        open = Some(self.repos[i].clone());
                                    }
                                    r.context_menu(|ui| {
                                        if ui.button("Remove from list").clicked() {
                                            remove = Some(i);
                                            ui.close_menu();
                                        }
                                        if !self.spaces.is_empty() {
                                            ui.menu_button("Spaces", |ui| {
                                                for (k, s) in self.spaces.iter().enumerate() {
                                                    let inside = s.repos.contains(&self.repos[i]);
                                                    if ui.selectable_label(inside, &s.name).clicked() {
                                                        toggle = Some((k, self.repos[i].clone()));
                                                        ui.close_menu();
                                                    }
                                                }
                                            });
                                        }
                                    });
                                }
                            });
                        } else {
                            for &i in &shown {
                                let r = repo_row(ui, &self.repos[i], &p, &self.style_of(&self.repos[i]));
                                if r.clicked() {
                                    open = Some(self.repos[i].clone());
                                }
                                r.context_menu(|ui| {
                                    if ui.button("Remove from list").clicked() {
                                        remove = Some(i);
                                        ui.close_menu();
                                    }
                                    if !self.spaces.is_empty() {
                                        ui.menu_button("Spaces", |ui| {
                                            for (k, s) in self.spaces.iter().enumerate() {
                                                let inside = s.repos.contains(&self.repos[i]);
                                                if ui.selectable_label(inside, &s.name).clicked() {
                                                    toggle = Some((k, self.repos[i].clone()));
                                                    ui.close_menu();
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                        }
                    });
                }
            });

        // Clone window
        if self.clone_open {
            let mut keep = true;
            let mut do_clone = false;
            egui::Window::new("Clone repository")
                .open(&mut keep)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Repository URL");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.clone_url)
                            .hint_text("https://github.com/user/repo.git")
                            .desired_width(380.0),
                    );
                    ui.add_space(8.0);
                    let ok = !self.clone_url.trim().is_empty() && self.busy.is_none();
                    if ui.add_enabled(ok, egui::Button::new("Choose folder and clone")).clicked() {
                        do_clone = true;
                    }
                });
            if !keep {
                self.clone_open = false;
            }
            if do_clone {
                if let Some(parent) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                    let url = self.clone_url.trim().to_string();
                    let name = url
                        .trim_end_matches('/')
                        .rsplit(['/', ':'])
                        .next()
                        .unwrap_or("repo")
                        .trim_end_matches(".git")
                        .to_string();
                    self.pending_clone = Some(parent.join(name));
                    self.spawn_in(ctx, parent, "Cloning", &["clone", &url]);
                    self.clone_open = false;
                }
            }
        }

        if let Some(b) = &self.busy {
            egui::Area::new(egui::Id::new("home_busy"))
                .anchor(Align2::RIGHT_BOTTOM, [-20.0, -16.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("{b}..."));
                    });
                });
        }

        if goto_settings {
            self.open_settings();
        }
        if create {
            if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                match git::init(&d) {
                    Ok(_) => open = Some(d),
                    Err(e) => self.status_msg = e,
                }
            }
        }
        if open_disk {
            if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                open = Some(d);
            }
        }
        if new_space {
            self.add_space();
        }
        if let Some(i) = del_space {
            self.remove_space(i);
        }
        if let Some((k, r)) = toggle {
            self.toggle_in_space(k, &r);
        }
        if let Some(d) = scan {
            self.scan_folder(&d);
        }
        if let Some(i) = remove {
            self.remove_repo(i);
        }
        if let Some(d) = open {
            self.open_repo(d);
        }
    }
}
