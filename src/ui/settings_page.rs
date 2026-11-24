#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn open_settings(&mut self) {
        self.git_name = git::config_get("user.name");
        self.git_email = git::config_get("user.email");
        self.ai_key_set = git::secret_get(git::AI_SECRET).is_some();
        self.status_msg.clear();
        self.screen = Screen::Settings;
    }

    pub fn settings_screen(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        egui::SidePanel::left("settings_rail")
            .exact_width(190.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 16.0, right: 12.0, top: 24.0, bottom: 16.0 }))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                rail_label(ui, "Application", &p);
                for (s, icon, name) in [
                    (Section::Application, "settings", "Application"),
                    (Section::Spaces, "layers", "Spaces"),
                    (Section::Appearance, "contrast", "Appearance"),
                    (Section::Keybinds, "keyboard", "Keybinds"),
                    (Section::Privacy, "shield", "Privacy"),
                    (Section::Accounts, "userCircle", "Accounts"),
                    (Section::Git, "branch", "Git"),
                    (Section::Editor, "pencil", "External editor"),
                    (Section::Ai, "sparkle", "AI"),
                    (Section::Help, "help", "Help"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                    }
                }
                ui.add_space(14.0);
                rail_label(ui, "Repository", &p);
                for (s, icon, name) in [
                    (Section::Remotes, "network", "Remotes"),
                    (Section::RepoStyle, "palette", "Appearance"),
                    (Section::Identity, "idCard", "Commit identity"),
                    (Section::Ignore, "stop", "Ignored files"),
                ] {
                    if rail_item(ui, icon, name, self.section == s, &p).clicked() {
                        self.section = s;
                        if s == Section::Ignore {
                            self.ignore_text = self.repo().map(|r| git::read_ignore(&r)).unwrap_or_default();
                        }
                        if s == Section::Identity {
                            self.load_repo_identity();
                        }
                    }
                }
            });

        let adding = self.section == Section::Accounts && self.acct_adding;
        if adding {
            egui::SidePanel::right("account_add")
                .exact_width(330.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(p.bg).stroke(egui::Stroke::new(1.0_f32, p.border)).inner_margin(egui::Margin::same(18.0)))
                .show(ctx, |ui| self.account_add_panel(ctx, ui, &p));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin { left: 24.0, right: 32.0, top: 20.0, bottom: 20.0 }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_max_width(580.0);
                    match self.section {
                        Section::Application => self.application_page(ui, &p),
                        Section::Spaces => self.spaces_page(ui, &p),
                        Section::Appearance => self.appearance_page(ctx, ui, &p),
                        Section::Keybinds => self.keybinds_page(ui, &p),
                        Section::Privacy => privacy_page(ui, &p),
                        Section::Git => self.git_page(ui, &p),
                        Section::Editor => self.editor_page(ui, &p),
                        Section::Ignore => self.ignore_page(ui, &p),
                        Section::Accounts => self.accounts_page(ui, &p),
                        Section::Ai => self.ai_page(ui, &p),
                        Section::Remotes => self.remotes_page(ui, &p),
                        Section::RepoStyle => self.repo_style_page(ui, &p),
                        Section::Identity => self.identity_page(ui, &p),
                        Section::Help => self.help_page(ui, &p),
                    }
                });
            });
    }

    fn application_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Application", "Install the onaygit command so local paths open in this app.", |_| {});
        let user_bin = home_dir().join(".local/bin/onaygit");
        let sys_bin = PathBuf::from("/usr/local/bin/onaygit");
        let mut act: Option<(bool, bool)> = None;
        settings_group(ui, p, "Command-line launcher", "", |_| {}, |ui| {
            for (system, name, desc, bin) in [
                (true, "All users", "Install in /usr/local/bin. macOS may ask for administrator approval.", &sys_bin),
                (false, "Only me", "Install in ~/.local/bin without administrator access.", &user_bin),
            ] {
                let installed = bin.exists();
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(name).size(13.5).color(p.text));
                            ui.label(RichText::new(desc).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(if installed { "Uninstall" } else { "Install" }).clicked() {
                                act = Some((system, installed));
                            }
                            ui.label(RichText::new(if installed { "Installed" } else { "Not installed" }).size(12.0).color(p.muted));
                        });
                    });
                });
            }
        });
        if !self.launcher_msg.is_empty() {
            ui.label(RichText::new(&self.launcher_msg).size(12.0).color(p.muted));
        } else {
            ui.label(
                RichText::new(format!("The per-user command will be installed at {}. Add ~/.local/bin to PATH if your shell cannot find it.", user_bin.parent().unwrap().display()))
                    .size(12.0)
                    .color(p.muted),
            );
        }
        ui.add_space(14.0);
        if let Some((system, installed)) = act {
            let bin = if system { sys_bin } else { user_bin };
            self.launcher_msg = match toggle_launcher(&bin, installed, system) {
                Ok(m) => m,
                Err(e) => e,
            };
        }
        settings_group(ui, p, "General", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Auto refresh").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.refresh_secs, 1..=30).suffix(" s"));
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Guided tips").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.settings.tips, "");
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Welcome window").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Show again").clicked() {
                            self.settings.onboarded = false;
                        }
                    });
                });
            });
        });
    }

    fn spaces_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Spaces", "Named groups of repositories shown on the home screen.", |_| {});
        let mut remove: Option<usize> = None;
        let count = format!("{}", self.spaces.len());
        settings_group(ui, p, "Spaces", &count, |_| {}, |ui| {
            if self.spaces.is_empty() {
                group_row(ui, p, |ui| {
                    ui.label(RichText::new("No spaces yet").color(p.muted));
                });
            }
            for (i, s) in self.spaces.iter().enumerate() {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&s.name).color(p.text));
                        ui.label(RichText::new(format!("{} repositories", s.repos.len())).size(12.0).color(p.muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                remove = Some(i);
                            }
                        });
                    });
                });
            }
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.space_name).hint_text("New space name").desired_width(240.0));
                    if ui.button("Add space").clicked() {
                        self.add_space();
                    }
                });
            });
        });
        if let Some(i) = remove {
            self.remove_space(i);
        }
    }

    fn appearance_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Appearance", "Theme, accent color and text size.", |_| {});
        settings_group(ui, p, "Theme", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    for t in Theme::ALL {
                        let icon = if matches!(t, Theme::Light | Theme::Sand) { "contrast" } else { "palette" };
                        if choice_tile(ui, p, icon, t.label(), self.settings.theme == t, 96.0).clicked() {
                            self.settings.theme = t;
                        }
                    }
                });
            });
        });
        settings_group(ui, p, "Accent color", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    for (i, (name, color)) in ACCENTS.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                        ui.painter().circle_filled(rect.center(), 11.0, *color);
                        if self.settings.accent == i {
                            paint_icon(ui, "check", rect.shrink(6.0), Color32::WHITE);
                        }
                        if resp.on_hover_text(*name).clicked() {
                            self.settings.accent = i;
                        }
                    }
                });
            });
        });
        settings_group(ui, p, "Text and size", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Diff font size").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Slider::new(&mut self.settings.diff_font, 10.0..=18.0));
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Interface zoom").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = ui.add(egui::Slider::new(&mut self.settings.zoom, 0.8..=1.5));
                        if r.drag_stopped() || (r.changed() && !r.dragged()) {
                            ctx.set_zoom_factor(self.settings.zoom);
                        }
                    });
                });
            });
        });
    }

    fn keybinds_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Keybinds", "Keyboard shortcuts.", |_| {});
        ui.add(egui::TextEdit::singleline(&mut self.keybind_search).hint_text("Search shortcuts").desired_width(f32::INFINITY));
        ui.add_space(12.0);
        let q = self.keybind_search.to_lowercase();
        let all = [
            ("Switch repository", "Cmd+P"),
            ("New tab", "Cmd+T"),
            ("Close tab", "Cmd+W"),
            ("Changes", "Cmd+1"),
            ("History", "Cmd+2"),
            ("Settings", "Cmd+,"),
        ];
        settings_group(ui, p, "General", "", |_| {}, |ui| {
            for (name, key) in all.iter().filter(|(n, k)| q.is_empty() || n.to_lowercase().contains(&q) || k.to_lowercase().contains(&q)) {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(*name).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(*key).monospace().color(p.muted));
                        });
                    });
                });
            }
        });
    }

    fn git_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Git", "Your global identity, used for new commits.", |_| {});
        settings_group(ui, p, "Identity", "global git config", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                field_label(ui, p, "Name");
                ui.add(egui::TextEdit::singleline(&mut self.git_name).desired_width(f32::INFINITY));
                field_label(ui, p, "Email");
                ui.add(egui::TextEdit::singleline(&mut self.git_email).desired_width(f32::INFINITY));
                ui.add_space(6.0);
                if ui.button("Save identity").clicked() {
                    let a = git::config_set("user.name", self.git_name.trim());
                    let b = git::config_set("user.email", self.git_email.trim());
                    if let Err(e) = a.and(b) {
                        self.status_msg = e;
                    }
                }
            });
        });
    }

    fn editor_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "External editor", "Used by Open in editor.", |_| {});
        settings_group(ui, p, "Default editor", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (label, cmd) in [("VS Code", "code"), ("Cursor", "cursor"), ("Zed", "zed"), ("Sublime", "subl"), ("Xcode", "open -a Xcode")] {
                        if ui.selectable_label(self.settings.editor == cmd, label).clicked() {
                            self.settings.editor = cmd.to_string();
                        }
                    }
                });
                field_label(ui, p, "Command");
                ui.add(egui::TextEdit::singleline(&mut self.settings.editor).desired_width(f32::INFINITY));
            });
        });
    }

    fn ignore_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Ignored files", "The .gitignore of this repository.", |_| {});
        let Some(repo) = self.repo() else {
            ui.label(RichText::new("Open a repository first.").color(p.muted));
            return;
        };
        settings_group(ui, p, ".gitignore", &repo_name(&repo), |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.ignore_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(14)
                        .desired_width(f32::INFINITY),
                );
                if ui.button("Save .gitignore").clicked() {
                    let r = git::write_ignore(&repo, &self.ignore_text);
                    self.report(r);
                    self.refresh();
                }
            });
        });
    }

    fn load_repo_identity(&mut self) {
        let Some(repo) = self.repo() else { return };
        let get = |k: &str| git::run(&repo, &["config", "--local", k]).map(|s| s.trim().to_string()).unwrap_or_default();
        self.repo_name_cfg = get("user.name");
        self.repo_email_cfg = get("user.email");
    }

    fn identity_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Commit identity", "Override your name and email for this repository only.", |_| {});
        let Some(repo) = self.repo() else {
            ui.label(RichText::new("Open a repository first.").color(p.muted));
            return;
        };
        settings_group(ui, p, "Identity", &repo_name(&repo), |_| {}, |ui| {
            group_row(ui, p, |ui| {
                field_label(ui, p, "Name");
                ui.add(egui::TextEdit::singleline(&mut self.repo_name_cfg).hint_text(&self.git_name).desired_width(f32::INFINITY));
                field_label(ui, p, "Email");
                ui.add(egui::TextEdit::singleline(&mut self.repo_email_cfg).hint_text(&self.git_email).desired_width(f32::INFINITY));
                ui.add_space(6.0);
                if ui.button("Save").clicked() {
                    for (k, v) in [("user.name", self.repo_name_cfg.trim()), ("user.email", self.repo_email_cfg.trim())] {
                        let args: Vec<&str> = if v.is_empty() { vec!["config", "--local", "--unset", k] } else { vec!["config", "--local", k, v] };
                        let _ = git::run(&repo, &args);
                    }
                    self.status_msg = "Saved".into();
                }
            });
        });
    }

    fn accounts_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut add = false;
        page_header(ui, p, "Accounts", "Connected hosting accounts.", |ui| {
            if ui.button("Add account").clicked() {
                add = true;
            }
        });
        if add {
            self.acct_adding = true;
        }
        let mut out: Option<usize> = None;
        let count = format!("{} account{}", self.accounts.len(), if self.accounts.len() == 1 { "" } else { "s" });
        settings_group(ui, p, "Connected accounts", &count, |_| {}, |ui| {
            if self.accounts.is_empty() {
                group_row(ui, p, |ui| {
                    ui.label(RichText::new("No accounts connected").color(p.muted));
                });
            }
            for (i, a) in self.accounts.iter().enumerate() {
                group_row(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        icon(ui, if a.provider == git::Provider::GitHub { "github" } else { "gitlab" }, 18.0, p.text);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&a.user).color(p.text));
                            ui.label(RichText::new(format!("{} \u{b7} Connected", a.host)).size(12.0).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::click());
                            paint_icon(ui, "remove", rect.shrink(3.0), Color32::from_rgb(220, 90, 90));
                            if resp.on_hover_text("Sign out").clicked() {
                                out = Some(i);
                            }
                        });
                    });
                });
            }
        });
        if let Some(i) = out {
            self.sign_out(i);
        }
    }

    fn account_add_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: &Pal) {
        ui.label(RichText::new("Add account").size(16.0).color(p.text));
        ui.label(RichText::new("Tokens are kept in the macOS Keychain.").size(12.5).color(p.muted));
        ui.add_space(12.0);
        field_label(ui, p, "Provider");
        ui.horizontal(|ui| {
            for pr in [git::Provider::GitHub, git::Provider::GitLab] {
                let icon = if pr == git::Provider::GitHub { "github" } else { "gitlab" };
                if choice_tile(ui, p, icon, pr.label(), self.acct_provider == pr, 140.0).clicked() {
                    self.acct_provider = pr;
                    self.acct_host.clear();
                    if pr != git::Provider::GitHub {
                        self.acct_cli = false;
                    }
                }
            }
        });
        ui.add_space(8.0);
        if self.acct_provider == git::Provider::GitHub {
            field_label(ui, p, "Connect with");
            ui.horizontal(|ui| {
                if choice_tile(ui, p, "key", "Access token", !self.acct_cli, 140.0).clicked() {
                    self.acct_cli = false;
                }
                if choice_tile(ui, p, "terminal", "GitHub CLI", self.acct_cli, 140.0).clicked() {
                    self.acct_cli = true;
                }
            });
            ui.add_space(8.0);
        }
        field_label(ui, p, "Host");
        ui.add(egui::TextEdit::singleline(&mut self.acct_host).hint_text(self.acct_provider.default_host()).desired_width(f32::INFINITY));
        if !self.acct_cli {
            field_label(ui, p, "Access token");
            ui.add(egui::TextEdit::singleline(&mut self.acct_token).password(true).desired_width(f32::INFINITY));
        } else {
            ui.add_space(6.0);
            ui.label(RichText::new("Uses the token from gh auth login.").size(12.0).color(p.muted));
        }
        if !self.status_msg.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(&self.status_msg).size(12.0).color(p.muted));
        }
        ui.add_space(12.0);
        let busy = self.task_running(T_SIGN_IN);
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.acct_adding = false;
                self.acct_token.clear();
            }
            let ready = self.acct_cli || !self.acct_token.trim().is_empty();
            if ui.add_enabled(!busy && ready, egui::Button::new("Connect")).clicked() {
                self.sign_in(ctx);
            }
            if busy {
                ui.spinner();
            }
        });
    }

    fn help_page(&mut self, ui: &mut egui::Ui, p: &Pal) {
        page_header(ui, p, "Help", "Getting started and support.", |_| {});
        settings_group(ui, p, "Getting started", "", |_| {}, |ui| {
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Guided tips").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.settings.tips, "");
                    });
                });
            });
            group_row(ui, p, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("First-run setup").color(p.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Run again").clicked() {
                            self.settings.onboarded = false;
                        }
                    });
                });
            });
        });
        settings_group(ui, p, "Support", &format!("OnayGit {}", env!("CARGO_PKG_VERSION")), |_| {}, |ui| {
            for (label, url) in [
                ("Source code", "https://github.com/Tisobyn/OnayGit"),
                ("Report an issue", "https://github.com/Tisobyn/OnayGit/issues"),
            ] {
                group_row(ui, p, |ui| {
                    ui.hyperlink_to(label, url);
                });
            }
            group_row(ui, p, |ui| {
                ui.label(RichText::new(format!("Repository list: {}", repos_path().display())).size(12.0).color(p.muted));
                ui.label(RichText::new(format!("Settings file: {}", settings_path().display())).size(12.0).color(p.muted));
            });
        });
    }
}

fn privacy_page(ui: &mut egui::Ui, p: &Pal) {
    page_header(ui, p, "Privacy", "What OnayGit does with your data.", |_| {});
    settings_group(ui, p, "Data", "", |_| {}, |ui| {
        for t in [
            "OnayGit sends nothing to any server of its own. There is no telemetry.",
            "Git commands run on this Mac, or on your machines over SSH.",
            "Account tokens and API keys are stored in the macOS Keychain.",
            "Network calls happen only when you use GitHub, GitLab or an AI provider.",
        ] {
            group_row(ui, p, |ui| {
                ui.label(RichText::new(t).color(p.text));
            });
        }
    });
}

/// Link or remove the `onaygit` command. The system folder asks macOS for admin approval.
fn toggle_launcher(bin: &Path, installed: bool, system: bool) -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if system {
        let cmd = if installed {
            format!("rm -f '{}'", bin.display())
        } else {
            format!("mkdir -p /usr/local/bin && ln -sf '{}' '{}'", exe.display(), bin.display())
        };
        let script = format!("do shell script \"{}\" with administrator privileges", cmd.replace('\\', "\\\\").replace('"', "\\\""));
        let out = std::process::Command::new("osascript").arg("-e").arg(script).output().map_err(|e| e.to_string())?;
        return if out.status.success() {
            Ok(if installed { "Removed".into() } else { format!("Installed at {}", bin.display()) })
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        };
    }
    if installed {
        std::fs::remove_file(bin).map_err(|e| e.to_string())?;
        return Ok("Removed".into());
    }
    if let Some(dir) = bin.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::os::unix::fs::symlink(exe, bin).map_err(|e| e.to_string())?;
    Ok(format!("Installed at {}", bin.display()))
}
