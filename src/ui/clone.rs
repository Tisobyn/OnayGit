//! Full-page Clone: pick a repository from an account or paste a URL.
use crate::prelude::*;

pub const T_REPOS: &str = "Repos:";

/// Folder name git would use for a URL.
pub fn folder_name(url: &str) -> String {
    url.trim().trim_end_matches('/').rsplit(['/', ':']).next().unwrap_or("repo").trim_end_matches(".git").to_string()
}

impl App {
    pub fn open_clone(&mut self) {
        self.clone_dir = None;
        self.clone_pick = None;
        self.clone_error.clear();
        self.clone_search.clear();
        self.clone_source = if self.accounts.is_empty() { None } else { Some(0) };
        self.screen = Screen::Clone;
    }

    fn load_clone_repos(&mut self, ctx: &egui::Context, i: usize) {
        let Some(acc) = self.accounts.get(i).cloned() else { return };
        let key = acc.secret_name();
        if self.clone_repos.contains_key(&key) || self.task_running(&format!("{T_REPOS}{key}")) {
            return;
        }
        let Some(token) = acc.token() else {
            self.clone_error = "The token for this account is missing from the Keychain".into();
            return;
        };
        self.spawn_task(ctx, &format!("{T_REPOS}{key}"), move || {
            let list = git::list_repos(&acc, &token)?;
            Ok(list.iter().map(|r| r.to_line()).collect::<Vec<_>>().join("\n"))
        });
    }

    fn clone_target(&self) -> Option<(String, PathBuf)> {
        let url = match self.clone_source {
            None => self.clone_url.trim().to_string(),
            Some(_) => self.clone_pick.clone().unwrap_or_default(),
        };
        if url.is_empty() {
            return None;
        }
        // An empty chosen folder is the target itself. A folder with files gets a sub folder named after the repository.
        let path = match self.clone_dir.clone() {
            Some(d) if d.is_dir() && std::fs::read_dir(&d).is_ok_and(|mut r| r.next().is_some()) => d.join(folder_name(&url)),
            Some(d) => d,
            None => home_dir().join(folder_name(&url)),
        };
        Some((url, path))
    }

    pub fn clone_screen(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let cloning = self.busy.as_deref() == Some("Cloning");
        let mut back = false;
        let mut add_account = false;
        let mut source = self.clone_source;
        let mut changed = false;

        egui::SidePanel::left("clone_sources")
            .exact_width(200.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if rail_item(ui, "arrowLeft", "Back", false, &p).clicked() && !cloning {
                    back = true;
                }
                ui.add_space(14.0);
                rail_label(ui, "SOURCES", &p);
                for (i, a) in self.accounts.iter().enumerate() {
                    let icon = if a.provider == git::Provider::GitHub { "github" } else { "gitlab" };
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 46.0), Sense::click());
                    if source == Some(i) || resp.hovered() {
                        ui.painter().rect_filled(rect, 6.0, p.hover);
                    }
                    paint_icon(ui, icon, egui::Rect::from_center_size(rect.min + egui::vec2(18.0, 23.0), egui::vec2(16.0, 16.0)), p.muted);
                    let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(6.0, 0.0)));
                    clip.text(rect.min + egui::vec2(36.0, 15.0), Align2::LEFT_CENTER, &a.user, FontId::proportional(13.5), p.text);
                    clip.text(rect.min + egui::vec2(36.0, 32.0), Align2::LEFT_CENTER, &a.host, FontId::proportional(12.0), p.muted);
                    if resp.clicked() && !cloning {
                        source = Some(i);
                        changed = true;
                    }
                }
                if rail_item(ui, "branch", "Repository URL", source.is_none(), &p).clicked() && !cloning {
                    source = None;
                    changed = true;
                }
                ui.add_space(10.0);
                if rail_item(ui, "add", "Add account", false, &p).clicked() && !cloning {
                    add_account = true;
                }
            });
        if changed {
            self.clone_source = source;
            self.clone_pick = None;
            self.clone_error.clear();
        }

        let target = self.clone_target();
        let mut pick_dir = false;
        let mut cancel = false;
        let mut go = false;
        egui::TopBottomPanel::bottom("clone_footer")
            .exact_height(78.0)
            .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin { left: 24.0, right: 24.0, top: 10.0, bottom: 10.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width((ui.available_width() - 190.0).min(560.0));
                        ui.label(RichText::new("Location").size(12.0).color(p.muted));
                        let shown = target.as_ref().map(|t| t.1.clone()).or(self.clone_dir.clone());
                        if folder_field_plain(ui, &p, shown.as_deref()).clicked() && !cloning {
                            pick_dir = true;
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                        if primary_button(ui, if cloning { "Cloning..." } else { "Clone" }, target.is_some() && !cloning).clicked() {
                            go = true;
                        }
                        if plain_button(ui, &p, "Cancel").clicked() && !cloning {
                            cancel = true;
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin::same(24.0)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Clone repository").size(20.0).color(p.text));
                ui.add_space(14.0);
                ui.set_max_width(760.0);
                if cloning {
                    ui.add_space(80.0);
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(RichText::new("Cloning repository...").color(p.text));
                        if let Some((_, path)) = &target {
                            ui.label(RichText::new(folder_name(&path.display().to_string())).size(12.5).color(p.muted));
                        }
                    });
                    return;
                }
                match self.clone_source {
                    None => {
                        ui.label(RichText::new("Repository URL").size(12.0).color(p.muted));
                        ui.add(egui::TextEdit::singleline(&mut self.clone_url).hint_text("https://github.com/owner/project.git").desired_width(f32::INFINITY).margin(egui::vec2(12.0, 9.0)));
                        ui.add_space(6.0);
                        ui.label(RichText::new("Clone using your configured Git credentials.").size(12.0).color(p.muted));
                    }
                    Some(i) => self.clone_repo_list(ctx, ui, &p, i),
                }
                if !self.clone_error.is_empty() {
                    ui.add_space(8.0);
                    ui.label(RichText::new(&self.clone_error).size(12.5).color(Color32::from_rgb(248, 81, 73)));
                }
            });

        if back || cancel {
            self.screen = if self.current.is_some() { Screen::Repo } else { Screen::Home };
        }
        if add_account {
            self.open_settings();
            self.section = Section::Accounts;
            self.acct_adding = true;
        }
        if pick_dir {
            if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).set_title("Choose empty clone folder").pick_folder() {
                self.clone_dir = Some(d);
            }
        }
        if go {
            if let Some((url, path)) = target {
                self.pending_clone = Some(path.clone());
                let env = match self.clone_source.and_then(|i| self.accounts.get(i)) {
                    Some(a) => a.token().and_then(|t| git::auth_env(a, &t).ok()).unwrap_or_default(),
                    None => vec![],
                };
                self.spawn_clone(ctx, url, path, env);
            }
        }
    }

    fn clone_repo_list(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal, i: usize) {
        self.load_clone_repos(ctx, i);
        let Some(acc) = self.accounts.get(i) else { return };
        let key = acc.secret_name();
        ui.label(RichText::new("Search").size(12.0).color(p.muted));
        ui.add(egui::TextEdit::singleline(&mut self.clone_search).hint_text("Filter repositories").desired_width(f32::INFINITY).margin(egui::vec2(12.0, 9.0)));
        ui.add_space(10.0);
        let loading = self.task_running(&format!("{T_REPOS}{key}"));
        let Some(list) = self.clone_repos.get(&key) else {
            if loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Loading repositories...").color(p.muted));
                });
            }
            return;
        };
        let q = self.clone_search.to_lowercase();
        let shown: Vec<&git::RemoteRepo> = list.iter().filter(|r| q.is_empty() || r.name.to_lowercase().contains(&q) || r.description.to_lowercase().contains(&q)).collect();
        if shown.is_empty() {
            ui.label(RichText::new("No repositories found.").color(p.muted));
            return;
        }
        let mut pick: Option<String> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, 54.0, shown.len(), |ui, range| {
            for r in &shown[range] {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), Sense::click());
                ui.add_space(2.0);
                let sel = self.clone_pick.as_deref() == Some(r.url.as_str());
                if sel || resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, p.hover);
                }
                if sel {
                    ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, Color32::from_rgb(128, 167, 255)));
                }
                paint_icon(ui, "folder", egui::Rect::from_center_size(rect.min + egui::vec2(20.0, 26.0), egui::vec2(16.0, 16.0)), p.muted);
                let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)));
                clip.text(rect.min + egui::vec2(40.0, 17.0), Align2::LEFT_CENTER, &r.name, FontId::proportional(13.5), p.text);
                let sub: String = r.description.chars().take(90).collect();
                clip.text(rect.min + egui::vec2(40.0, 36.0), Align2::LEFT_CENTER, sub, FontId::proportional(12.0), p.muted);
                clip.text(rect.right_center() - egui::vec2(12.0, 0.0), Align2::RIGHT_CENTER, if r.private { "Private" } else { "Public" }, FontId::proportional(12.0), p.muted);
                if resp.clicked() {
                    pick = Some(r.url.clone());
                }
            }
        });
        if pick.is_some() {
            self.clone_pick = pick;
            self.clone_dir = None;
        }
    }

    pub fn handle_repos_result(&mut self, key: &str, res: &Result<String, String>) {
        match res {
            Ok(tsv) => {
                self.clone_repos.insert(key.to_string(), tsv.lines().filter_map(git::RemoteRepo::from_line).collect());
                self.clone_error.clear();
            }
            Err(e) => self.clone_error = e.clone(),
        }
    }

    fn spawn_clone(&mut self, ctx: &egui::Context, url: String, path: PathBuf, env: Vec<(String, String)>) {
        if self.busy.is_some() {
            return;
        }
        self.busy = Some("Cloning".to_string());
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let env_refs: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let dest = path.display().to_string();
            let res = git::run_env(&home_dir(), &["clone", &url, &dest], &env_refs);
            let _ = tx.send(("Cloning".to_string(), res));
            ctx.request_repaint();
        });
    }
}

/// Bordered, button-like path field.
fn folder_field_plain(ui: &mut egui::Ui, p: &Pal, value: Option<&Path>) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), Sense::click());
    ui.painter().rect_filled(rect, 6.0, if resp.hovered() { p.hover } else { p.input });
    ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, p.border));
    paint_icon(ui, "folder", egui::Rect::from_center_size(rect.left_center() + egui::vec2(18.0, 0.0), egui::vec2(14.0, 14.0)), p.muted);
    let text = value.map(|d| d.display().to_string()).unwrap_or_else(|| "Choose folder...".to_string());
    ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0))).text(rect.left_center() + egui::vec2(34.0, 0.0), Align2::LEFT_CENTER, text, FontId::proportional(13.5), p.text);
    resp
}
