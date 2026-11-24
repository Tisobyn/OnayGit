//! Command palette (Cmd+P): actions, settings pages, repositories and machines.
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone)]
enum Hit {
    Settings(Option<Section>),
    Create,
    Clone,
    Disk,
    Repo(PathBuf),
    Machine(usize),
}

struct Row {
    icon: &'static str,
    title: String,
    sub: String,
    hit: Hit,
}

impl App {
    fn palette_rows(&self) -> Vec<Row> {
        let mut rows = vec![Row { icon: "settings", title: "Open Settings".into(), sub: "\u{2318}, \u{b7} Command".into(), hit: Hit::Settings(None) }];
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
            (Section::About, "info", "About"),
        ] {
            rows.push(Row { icon, title: format!("Settings \u{b7} {name}"), sub: "Settings".into(), hit: Hit::Settings(Some(s)) });
        }
        rows.push(Row { icon: "add", title: "Create repository".into(), sub: "Command".into(), hit: Hit::Create });
        rows.push(Row { icon: "branch", title: "Clone repository".into(), sub: "Command".into(), hit: Hit::Clone });
        rows.push(Row { icon: "folder", title: "Open from disk".into(), sub: "Command".into(), hit: Hit::Disk });
        let current = self.repo();
        for r in &self.repos {
            let name = repo_name(r);
            let sub = if Some(r) == current.as_ref() { format!("{} \u{b7} Current", r.display()) } else { r.display().to_string() };
            rows.push(Row { icon: "folder", title: name, sub, hit: Hit::Repo(r.clone()) });
        }
        for (i, m) in self.machines.iter().enumerate() {
            rows.push(Row { icon: "desktop", title: m.name.clone(), sub: format!("Machine \u{b7} {}", m.host), hit: Hit::Machine(i) });
        }
        rows
    }

    pub fn switcher_window(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        let q = self.switcher_text.to_lowercase();
        let rows: Vec<Row> = self
            .palette_rows()
            .into_iter()
            .filter(|r| q.is_empty() || r.title.to_lowercase().contains(&q) || r.sub.to_lowercase().contains(&q))
            .collect();
        if !rows.is_empty() {
            self.switcher_sel = self.switcher_sel.min(rows.len() - 1);
        }
        let (down, up, enter) = ctx.input(|i| (i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::Enter)));
        if down && self.switcher_sel + 1 < rows.len() {
            self.switcher_sel += 1;
        }
        if up {
            self.switcher_sel = self.switcher_sel.saturating_sub(1);
        }
        let mut pick: Option<Hit> = enter.then(|| rows.get(self.switcher_sel).map(|r| r.hit.clone())).flatten();
        let mut close = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        egui::Area::new(egui::Id::new("palette_dim")).order(egui::Order::Middle).fixed_pos(egui::Pos2::ZERO).show(ctx, |ui| {
            let r = ctx.screen_rect();
            ui.painter().rect_filled(r, 0.0, Color32::from_black_alpha(110));
            if ui.allocate_rect(r, Sense::click()).clicked() {
                close = true;
            }
        });
        let frame = egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(10.0)).rounding(12.0).fill(p.bg);
        egui::Window::new("palette")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .frame(frame)
            .anchor(Align2::CENTER_TOP, [0.0, 110.0])
            .show(ctx, |ui| {
                ui.set_width(640.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                egui::Frame::none().stroke(egui::Stroke::new(1.0_f32, p.border)).rounding(8.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            ui.add_space(13.0);
                            let (r, _) = ui.allocate_exact_size(egui::vec2(15.0, 15.0), Sense::hover());
                            paint_icon(ui, "search", r, p.muted);
                        });
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.switcher_text)
                                .hint_text("Search actions, repositories, paths, or machines...")
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .margin(egui::vec2(6.0, 13.0)),
                        );
                        r.request_focus();
                    });
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(520.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for (i, row) in rows.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), Sense::click());
                        let sel = i == self.switcher_sel;
                        if sel || resp.hovered() {
                            ui.painter().rect_filled(rect, 8.0, if sel { p.hover } else { p.hover.gamma_multiply(0.5) });
                        }
                        if sel && (down || up) {
                            ui.scroll_to_rect(rect, None);
                        }
                        paint_icon(ui, row.icon, egui::Rect::from_center_size(rect.left_center() + egui::vec2(24.0, 0.0), egui::vec2(16.0, 16.0)), p.muted);
                        let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(10.0, 0.0)));
                        clip.text(rect.left_center() + egui::vec2(52.0, -9.0), Align2::LEFT_CENTER, &row.title, FontId::proportional(13.5), p.text);
                        clip.text(rect.left_center() + egui::vec2(52.0, 10.0), Align2::LEFT_CENTER, &row.sub, FontId::proportional(12.0), p.muted);
                        if resp.clicked() {
                            pick = Some(row.hit.clone());
                        }
                    }
                    if rows.is_empty() {
                        ui.add_space(12.0);
                        ui.vertical_centered(|ui| ui.label(RichText::new("No match").color(p.muted)));
                        ui.add_space(12.0);
                    }
                });
            });

        if let Some(hit) = pick {
            self.dialog = None;
            match hit {
                Hit::Settings(s) => {
                    self.open_settings();
                    if let Some(s) = s {
                        self.section = s;
                    }
                }
                Hit::Create => {
                    self.screen = Screen::Home;
                    self.create_open = true;
                    self.create_dir = None;
                    self.create_branch = "main".into();
                    self.create_anchor = egui::pos2(230.0, 420.0);
                }
                Hit::Clone => {
                    self.open_clone();
                }
                Hit::Disk => {
                    if let Some(d) = rfd::FileDialog::new().set_directory(home_dir()).pick_folder() {
                        self.open_repo(d);
                    }
                }
                Hit::Repo(r) => self.open_repo(r),
                Hit::Machine(i) => {
                    self.screen = Screen::Home;
                    self.selected_machine = Some(i);
                }
            }
        } else if close {
            self.dialog = None;
        }
    }
}
