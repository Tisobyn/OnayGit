#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn home(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let mut open: Option<PathBuf> = None;
        let mut remove: Option<usize> = None;
        let mut scan: Option<PathBuf> = None;
        let goto_settings = false;
        let mut open_disk = false;
        let mut del_space: Option<usize> = None;
        let mut new_space = false;
        let mut toggle: Option<(usize, PathBuf)> = None;

        // Content is centered, about 980 wide: a 220 wide column on the left and the list on the right.
        let total = ctx.available_rect().width();
        let side = ((total - 980.0) / 2.0).max(24.0);
        egui::SidePanel::left("home_rail")
            .exact_width(side + 220.0 + 20.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: side, right: 20.0, top: 40.0, bottom: 20.0 }))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                rail_label(ui, "MACHINES", &p);
                if rail_item_h(ui, "desktop", "This Mac", self.selected_machine.is_none(), &p, 37.0).clicked() {
                    self.selected_machine = None;
                }
                for i in 0..self.machines.len() {
                    let name = self.machines[i].name.clone();
                    if rail_item_h(ui, "network", &name, self.selected_machine == Some(i), &p, 37.0).clicked() {
                        self.selected_machine = Some(i);
                    }
                }
                if rail_item_h(ui, "add", "Add machine...", false, &p, 37.0).clicked() {
                    self.status_msg.clear();
                    self.dialog = Some(Dialog::AddMachine);
                }
                ui.add_space(20.0);
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
                    let w = ui.available_width() - 70.0;
                    let r = ui.add(egui::TextEdit::singleline(&mut self.space_name).hint_text("New space").desired_width(w));
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.small_button("Add").clicked() || enter {
                        new_space = true;
                    }
                });
                ui.add_space(20.0);
                rail_label(ui, "QUICK ACTIONS", &p);
                let r = rail_item_h(ui, "add", "Create repository", false, &p, 28.0);
                if r.clicked() {
                    self.create_open = !self.create_open;
                    self.create_anchor = r.rect.left_top() - egui::vec2(0.0, 8.0);
                    self.create_dir = None;
                    self.create_branch = "main".into();
                }
                if rail_item_h(ui, "branch", "Clone repository", false, &p, 28.0).clicked() {
                    self.clone_open = true;
                    self.clone_dir = None;
                }
                if rail_item_h(ui, "folder", "Open from disk", false, &p, 28.0).clicked() {
                    open_disk = true;
                }
                if rail_item_h(ui, "sync", "Scan repositories", false, &p, 28.0).clicked() {
                    if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                        scan = Some(d);
                    }
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 0.0, right: side, top: 40.0, bottom: 20.0 }))
            .show(ctx, |ui| {
                if let Some(mi) = self.selected_machine {
                    self.machine_view(ctx, ui, mi, &p, &mut open);
                    return;
                }
                self.load_repo_info(ctx);
                ctx.request_repaint_after(Duration::from_secs(2));
                ui.horizontal(|ui| {
                    let w = ui.available_width() - 90.0;
                    egui::Frame::none().stroke(egui::Stroke::new(1.0_f32, p.border)).rounding(8.0).show(ui, |ui| {
                        ui.set_width(w - 2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.add_space(11.0);
                                let (r, _) = ui.allocate_exact_size(egui::vec2(15.0, 15.0), Sense::hover());
                                paint_icon(ui, "search", r, p.muted);
                            });
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search)
                                    .hint_text(format!("Search {} repositories", self.repos.len()))
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .margin(egui::vec2(6.0, 11.0)),
                            );
                        });
                    });
                    for (name, grid) in [("grid", true), ("list", false)] {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::click());
                        if self.grid == grid || resp.hovered() {
                            ui.painter().rect_filled(rect, 8.0, p.hover);
                        }
                        paint_icon(ui, name, rect.shrink(9.0), if self.grid == grid { accent() } else { p.muted });
                        if resp.clicked() {
                            self.grid = grid;
                        }
                    }
                });
                ui.add_space(14.0);
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

                {
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
                                let r = repo_row(ui, &self.repos[i], &p, &self.style_of(&self.repos[i]), self.repo_styles.contains_key(&self.repos[i]), self.repo_info.get(&self.repos[i]));
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

        // Clone dialog
        if self.clone_open {
            let mut do_clone = false;
            let mut pick = false;
            let mut cancel = false;
            let close = modal(ctx, &p, "clone_repo", 540.0, Place::Center, "Clone repository", "", |ui| {
                dialog_field(ui, &p, "Repository URL", &mut self.clone_url, "https://github.com/user/repo.git");
                if folder_field(ui, &p, "Location", self.clone_dir.as_deref()).clicked() {
                    pick = true;
                }
                ui.add_space(16.0);
                let w = ui.available_width();
                ui.allocate_ui_with_layout(egui::vec2(w, 32.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ok = !self.clone_url.trim().is_empty() && self.clone_dir.is_some() && self.busy.is_none();
                    if primary_button(ui, "Clone", ok).clicked() {
                        do_clone = true;
                    }
                    if plain_button(ui, &p, "Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
            if pick {
                if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                    self.clone_dir = Some(d);
                }
            }
            if close || cancel {
                self.clone_open = false;
            }
            if do_clone {
                if let Some(parent) = self.clone_dir.clone() {
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

        // Create repository popover
        if self.create_open {
            let mut pick = false;
            let mut go = false;
            let mut cancel = false;
            let close = modal(ctx, &p, "create_repo", 340.0, Place::At(self.create_anchor, Align2::LEFT_BOTTOM), "Create repository", "", |ui| {
                if folder_field(ui, &p, "Location", self.create_dir.as_deref()).clicked() {
                    pick = true;
                }
                dialog_field(ui, &p, "Initial branch", &mut self.create_branch, "main");
                ui.add_space(10.0);
                let w = ui.available_width();
                ui.allocate_ui_with_layout(egui::vec2(w, 32.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "Create & open", self.create_dir.is_some()).clicked() {
                        go = true;
                    }
                    if plain_button(ui, &p, "Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
            if pick {
                if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                    self.create_dir = Some(d);
                }
            }
            if close || cancel {
                self.create_open = false;
            }
            if go {
                if let Some(d) = self.create_dir.clone() {
                    match git::init(&d, &self.create_branch) {
                        Ok(_) => {
                            self.create_open = false;
                            open = Some(d);
                        }
                        Err(e) => self.status_msg = e,
                    }
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
