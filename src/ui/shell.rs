//! Window frame in the style of 1git: tabs in the title bar, an icon rail on the left
//! and a rounded content card.
use crate::prelude::*;
use crate::app::spaces::space_icon_name;
use crate::ui::settings_page::space_color32;

const TITLE_H: f32 = 40.0;
const RAIL_W: f32 = 52.0;
const GAP: f32 = 8.0;
const RADIUS: f32 = 12.0;

/// Small pill at the bottom right of a rail button: arrow icons with counts. Empty icon name means a plain count.
fn rail_badge(ui: &egui::Ui, rect: egui::Rect, p: &Pal, parts: &[(&str, usize)]) {
    let parts: Vec<&(&str, usize)> = parts.iter().filter(|x| x.1 > 0).collect();
    if parts.is_empty() {
        return;
    }
    let font = FontId::proportional(10.0);
    let widths: Vec<f32> = parts.iter().map(|(ic, n)| {
        let t = ui.painter().layout_no_wrap(n.min(&99).to_string(), font.clone(), p.text).size().x;
        t + if ic.is_empty() { 0.0 } else { 8.0 }
    }).collect();
    let w = widths.iter().sum::<f32>() + 8.0 + 3.0 * (parts.len() as f32 - 1.0);
    let pill = egui::Rect::from_min_size(egui::pos2(rect.right() - w + 6.0, rect.bottom() - 11.0), egui::vec2(w, 14.0));
    ui.painter().rect_filled(pill, 7.0, p.bg);
    ui.painter().rect_stroke(pill, 7.0, egui::Stroke::new(1.0_f32, p.border));
    let mut x = pill.left() + 4.0;
    for ((ic, n), w) in parts.iter().zip(widths) {
        if !ic.is_empty() {
            paint_icon(ui, ic, egui::Rect::from_min_size(egui::pos2(x, pill.center().y - 3.5), egui::vec2(7.0, 7.0)), p.text);
            x += 8.0;
        }
        ui.painter().text(egui::pos2(x, pill.center().y), Align2::LEFT_CENTER, n.min(&99).to_string(), font.clone(), p.text);
        x += w - if ic.is_empty() { 0.0 } else { 8.0 } + 3.0;
    }
}

fn icon_button(ui: &mut egui::Ui, name: &str, tip: &str, selected: bool, p: &Pal) -> egui::Response {
    icon_button_tint(ui, name, tip, selected, p, if selected { p.text } else { p.muted })
}

fn icon_button_tint(ui: &mut egui::Ui, name: &str, tip: &str, selected: bool, p: &Pal, tint: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::click());
    if selected || resp.hovered() {
        let a = if selected { 8 } else { 4 };
        let fill = if p.bg.r() > 128 {
            Color32::from_rgba_unmultiplied(0, 0, 0, a)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, a)
        };
        ui.painter().rect_filled(rect, 8.0, fill);
    }
    paint_icon(ui, name, rect.shrink(9.0), tint);
    resp.on_hover_text(tip)
}

impl App {
    /// Title bar with tabs, left rail and margins. Leaves the card area for the caller.
    pub fn shell(&mut self, ctx: &egui::Context) {
        let p = self.pal();
        self.titlebar(ctx, &p);
        self.rail(ctx, &p);
        let none = egui::Frame::none().fill(p.chrome);
        egui::SidePanel::right("shell_right")
            .exact_width(GAP)
            .resizable(false)
            .show_separator_line(false)
            .frame(none)
            .show(ctx, |_| {});
        egui::TopBottomPanel::bottom("shell_bottom")
            .exact_height(GAP)
            .resizable(false)
            .show_separator_line(false)
            .frame(none)
            .show(ctx, |_| {});
        self.card_rect = ctx.available_rect();
    }

    fn titlebar(&mut self, ctx: &egui::Context, p: &Pal) {
        let mut activate: Option<usize> = None;
        let mut close: Option<usize> = None;
        let mut go_home = false;
        let mut leave_settings = false;
        let mut new_tab = false;
        let mut reorder: Option<(usize, usize)> = None;
        let mut rects: Vec<egui::Rect> = vec![];
        let mut dragged: Option<usize> = None;
        let current = self.repo();

        egui::TopBottomPanel::top("titlebar")
            .exact_height(TITLE_H)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(p.chrome))
            .show(ctx, |ui| {
                // Dragging the empty bar moves the window.
                let bar = ui.max_rect();
                let drag = ui.interact(bar, egui::Id::new("titlebar_drag"), Sense::click_and_drag());
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }

                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.add_space(82.0);
                    // Home
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if icon_button(ui, "home", "Repositories", self.screen == Screen::Home, p).clicked() {
                            go_home = true;
                        }
                    });
                    ui.add_space(4.0);

                    for (i, path) in self.tabs.iter().enumerate() {
                        let name = repo_name(path);
                        let style = self.style_of(path);
                        let active = current.as_ref() == Some(path) && self.screen == Screen::Repo;
                        let label = if active && !self.branch.is_empty() {
                            format!("{name} / {}", self.branch)
                        } else {
                            name.clone()
                        };
                        let font = FontId::proportional(12.5);
                        let tw = ui.fonts(|f| f.layout_no_wrap(label.clone(), font.clone(), p.text).size().x);
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(tw + 70.0, 34.0), Sense::hover());
                        // A stable id per repository keeps the drag alive while tabs swap places.
                        let resp = ui.interact(rect, egui::Id::new(("tab", path.clone())), Sense::click_and_drag());
                        rects.push(rect);
                        if resp.dragged() {
                            dragged = Some(i);
                            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                        }
                        if active {
                            let r = egui::Rounding { nw: 10.0, ne: 10.0, sw: 0.0, se: 0.0 };
                            ui.painter().rect_filled(rect, r, p.bg);
                        } else if resp.hovered() {
                            let r = egui::Rounding { nw: 10.0, ne: 10.0, sw: 0.0, se: 0.0 };
                            ui.painter().rect_filled(rect, r, if p.bg.r() > 128 { Color32::from_rgba_unmultiplied(0, 0, 0, 4) } else { Color32::from_rgba_unmultiplied(255, 255, 255, 4) });
                        }
                        // Letter avatar
                        let av = egui::Rect::from_center_size(rect.left_center() + egui::vec2(20.0, -1.0), egui::vec2(18.0, 18.0));
                        ui.painter().rect_filled(av, 5.0, p.hover);
                        ui.painter().text(
                            av.center(),
                            Align2::CENTER_CENTER,
                            name.chars().next().unwrap_or('?').to_uppercase().to_string(),
                            FontId::proportional(10.5),
                            if active && self.repo_styles.contains_key(path) { ACCENTS[style.color % ACCENTS.len()].1 } else { p.muted },
                        );
                        ui.painter().text(
                            rect.left_center() + egui::vec2(36.0, -1.0),
                            Align2::LEFT_CENTER,
                            &label,
                            font,
                            if active { p.text } else { p.muted },
                        );
                        let x_rect = egui::Rect::from_center_size(rect.right_center() - egui::vec2(16.0, 1.0), egui::vec2(12.0, 12.0));
                        let over_x = resp.hover_pos().is_some_and(|pos| x_rect.expand(4.0).contains(pos));
                        if active || resp.hovered() {
                            paint_icon(ui, "close", x_rect, if over_x { p.text } else { p.muted });
                        }
                        if resp.clicked() {
                            if over_x {
                                close = Some(i);
                            } else {
                                activate = Some(i);
                            }
                        }
                        if resp.middle_clicked() {
                            close = Some(i);
                        }
                    }

                    if let (Some(from), Some(pos)) = (dragged, ctx.pointer_latest_pos()) {
                        if let Some(to) = rects.iter().position(|r| r.x_range().contains(pos.x)) {
                            if to != from {
                                reorder = Some((from, to));
                            }
                        }
                    }

                    if self.screen == Screen::Settings {
                        let font = FontId::proportional(12.5);
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(120.0, 34.0), Sense::click());
                        let r = egui::Rounding { nw: 10.0, ne: 10.0, sw: 0.0, se: 0.0 };
                        ui.painter().rect_filled(rect, r, p.bg);
                        paint_icon(ui, "settings", egui::Rect::from_center_size(rect.left_center() + egui::vec2(20.0, -1.0), egui::vec2(14.0, 14.0)), p.muted);
                        ui.painter().text(rect.left_center() + egui::vec2(36.0, -1.0), Align2::LEFT_CENTER, "Settings", font, p.text);
                        let x_rect = egui::Rect::from_center_size(rect.right_center() - egui::vec2(16.0, 1.0), egui::vec2(12.0, 12.0));
                        paint_icon(ui, "close", x_rect, p.muted);
                        if resp.clicked() {
                            leave_settings = true;
                        }
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if icon_button(ui, "add", "Open another repository (Cmd+T)", false, p).clicked() {
                            new_tab = true;
                        }
                    });
                });
            });

        if go_home {
            self.screen = Screen::Home;
        }
        if leave_settings {
            self.screen = if self.current.is_some() { Screen::Repo } else { Screen::Home };
        }
        if let Some((from, to)) = reorder {
            self.move_tab(from, to);
            ctx.request_repaint();
        }
        if let Some(i) = activate {
            self.activate_tab(i);
        }
        if let Some(i) = close {
            self.close_tab(i);
        }
        if new_tab {
            self.screen = Screen::Home;
        }
    }

    fn rail(&mut self, ctx: &egui::Context, p: &Pal) {
        let mut new_tab: Option<Tab> = None;
        let mut settings = false;
        let mut home = false;
        let accounts = false;
        let mut prs = false;
        let mut sync: Option<(&str, Vec<String>)> = None;
        let mut publish = false;
        let mut menu: Option<(u8, egui::Rect)> = None;
        egui::SidePanel::left("rail")
            .exact_width(RAIL_W)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(p.chrome).inner_margin(egui::Margin::symmetric(8.0, 6.0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                if self.screen == Screen::Repo {
                    // Sync: one button for fetch, pull, push and publish, with a count badge.
                    let (sicon, title, sub, args) = self.sync_info();
                    let busy = self.busy.is_some();
                    let r = icon_button(ui, sicon, &format!("{title}\n{sub}"), busy, p);
                    let (ahead, behind) = self.ahead_behind.unwrap_or((0, 0));
                    rail_badge(ui, r.rect, p, &[("arrowUp", ahead as usize), ("arrowDown", behind as usize)]);
                    if r.clicked() && !busy {
                        match args {
                            Some((label, a)) => sync = Some((label, a)),
                            None => publish = true,
                        }
                    }
                    if icon_button(ui, "branch", "Pull requests", false, p).clicked() {
                        prs = true;
                    }
                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);
                    if icon_button(ui, "tree", "Changes", self.tab == Tab::Changes, p).clicked() {
                        new_tab = Some(Tab::Changes);
                    }
                    if icon_button(ui, "history", "History", self.tab == Tab::History, p).clicked() {
                        new_tab = Some(Tab::History);
                    }
                    let r = icon_button(ui, "archive", "Stashes", false, p);
                    rail_badge(ui, r.rect, p, &[("", self.stashes.len())]);
                    if r.clicked() {
                        new_tab = Some(Tab::Changes);
                        self.stash_open = true;
                    }
                    if icon_button(ui, "network", "Graph", self.tab == Tab::Graph, p).clicked() {
                        new_tab = Some(Tab::Graph);
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    let r = icon_button(ui, "github", "Accounts", false, p);
                    if r.clicked() {
                        menu = Some((2, r.rect));
                    }
                    let in_repo = self.screen == Screen::Repo;
                    let r = icon_button(ui, "settings", if in_repo { "Workspace settings (Cmd+, opens Settings)" } else { "Settings (Cmd+,)" }, self.screen == Screen::Settings, p);
                    if r.clicked() {
                        if in_repo {
                            menu = Some((0, r.rect));
                        } else {
                            settings = true;
                        }
                    }
                    let (sicon, scol) = match self.selected_space.and_then(|i| self.spaces.get(i)) {
                        Some(s) => (space_icon_name(&s.icon), if s.color.is_some() { space_color32(s.color) } else { p.muted }),
                        None => ("layers".to_string(), p.muted),
                    };
                    let r = icon_button_tint(ui, &sicon, "Spaces", false, p, scol);
                    if r.clicked() {
                        menu = Some((1, r.rect));
                    }
                    if icon_button(ui, "home", "Repositories", self.screen == Screen::Home, p).clicked() {
                        home = true;
                    }
                });
            });
        if let Some((k, r)) = menu {
            let open = matches!(self.rail_menu, Some((kind, _, _)) if kind == k);
            self.rail_menu = if open { None } else { Some((k, r.right_bottom() + egui::vec2(6.0, 0.0), ctx.input(|i| i.time))) };
        }
        self.rail_menu_ui(ctx, p);
        if let Some(t) = new_tab {
            self.tab = t;
            match t {
                Tab::Ignored => self.load_ignored(),
                Tab::Graph => self.load_graph(),
                _ => {}
            }
        }
        if publish {
            self.open_publish();
        }
        if prs {
            self.open_pull_requests(ctx);
        }
        if let Some((label, args)) = sync {
            if let Some(repo) = self.repo() {
                let a: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                self.spawn_in(ctx, repo, label, &a);
            }
        }
        if home {
            self.screen = Screen::Home;
        }
        if settings {
            self.open_settings();
        }
        if accounts {
            self.open_settings();
            self.section = Section::Accounts;
        }
    }

    /// Paint the card background so gaps between panels never show the window frame.
    pub fn fill_card(&self, ctx: &egui::Context) {
        ctx.layer_painter(egui::LayerId::background()).rect_filled(self.card_rect, 0.0, self.pal().bg);
    }

    /// Fine noise over the window chrome (title bar, rail and margins), never over content.
    pub fn paint_grain(&mut self, ctx: &egui::Context) {
        if !self.settings.grain || !self.card_rect.is_positive() {
            return;
        }
        let tex = self
            .grain_tex
            .get_or_insert_with(|| {
                let n = 96usize;
                let mut seed = 0x2545F491u32;
                let mut pixels = Vec::with_capacity(n * n);
                for _ in 0..n * n {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    let v = if seed & 1 == 0 { 0u8 } else { 255u8 };
                    pixels.push(Color32::from_rgba_unmultiplied(v, v, v, ((seed >> 8) & 0x07) as u8));
                }
                let img = egui::ColorImage { size: [n, n], pixels };
                ctx.load_texture("grain", img, egui::TextureOptions { wrap_mode: egui::TextureWrapMode::Repeat, ..egui::TextureOptions::NEAREST })
            })
            .clone();
        // Tile size in points: finer patterns repeat the texture more often.
        let tile = [48.0_f32, 96.0, 192.0][self.settings.grain_pattern.min(2)];
        let op = [0.5_f32, 1.0, 1.5][self.settings.grain_strength.min(2)];
        let tint = Color32::from_white_alpha((40.0 * op).min(255.0) as u8);
        let screen = ctx.screen_rect();
        let card = self.card_rect;
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("grain")));
        let strips = [
            egui::Rect::from_min_max(screen.min, egui::pos2(screen.max.x, card.min.y)),
            egui::Rect::from_min_max(egui::pos2(screen.min.x, card.min.y), egui::pos2(card.min.x, card.max.y)),
            egui::Rect::from_min_max(egui::pos2(card.max.x, card.min.y), egui::pos2(screen.max.x, card.max.y)),
            egui::Rect::from_min_max(egui::pos2(screen.min.x, card.max.y), screen.max),
        ];
        for r in strips {
            if r.is_positive() {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(r.width() / tile, r.height() / tile));
                painter.image(tex.id(), r, uv, tint);
            }
        }
    }

    /// Round the corners of the content card by painting the outside corners in the frame color.
    pub fn paint_corners(&self, ctx: &egui::Context) {
        let r = self.card_rect;
        if !r.is_positive() {
            return;
        }
        let color = self.pal().chrome;
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("card_corners")));
        // (corner point, arc centre, start angle in quarter turns)
        let corners = [
            (r.left_top(), r.left_top() + egui::vec2(RADIUS, RADIUS), 2.0_f32),
            (r.right_top(), r.right_top() + egui::vec2(-RADIUS, RADIUS), 3.0),
            (r.right_bottom(), r.right_bottom() + egui::vec2(-RADIUS, -RADIUS), 0.0),
            (r.left_bottom(), r.left_bottom() + egui::vec2(RADIUS, -RADIUS), 1.0),
        ];
        for (corner, centre, quarter) in corners {
            let mut mesh = egui::epaint::Mesh::default();
            mesh.colored_vertex(corner, color);
            let steps = 10;
            for i in 0..=steps {
                let a = (quarter + i as f32 / steps as f32) * std::f32::consts::FRAC_PI_2;
                mesh.colored_vertex(centre + RADIUS * egui::vec2(a.cos(), a.sin()), color);
            }
            for i in 1..=steps {
                mesh.add_triangle(0, i as u32, i as u32 + 1);
            }
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

impl App {
    /// Menu that opens beside a rail button: settings shortcuts, spaces or accounts.
    fn rail_menu_ui(&mut self, ctx: &egui::Context, p: &Pal) {
        let Some((kind, pos, opened)) = self.rail_menu else { return };
        // (label, checked, enabled); an empty label is a separator.
        let mut items: Vec<(String, bool, bool)> = vec![];
        let sep = || (String::new(), false, false);
        match kind {
            0 => {
                for l in ["Appearance...", "Accounts...", "Spaces...", "AI...", "Help..."] {
                    items.push((l.into(), false, true));
                }
                items.push(sep());
                for l in ["Remotes...", "Ignored file history..."] {
                    items.push((l.into(), false, true));
                }
                items.push(sep());
                items.push(("All Settings...".into(), false, true));
            }
            1 => {
                items.push(("All repositories".into(), self.selected_space.is_none(), true));
                for (i, s) in self.spaces.iter().enumerate() {
                    items.push((s.name.clone(), self.selected_space == Some(i), true));
                }
                items.push(sep());
                items.push(("New space...".into(), false, true));
                items.push(("Manage spaces...".into(), false, true));
            }
            _ => {
                if self.accounts.is_empty() {
                    items.push(("No accounts connected".into(), false, false));
                }
                for (i, a) in self.accounts.iter().enumerate() {
                    let prov = if a.host.contains("gitlab") { "GitLab" } else { "GitHub" };
                    items.push((format!("{} \u{b7} {prov} ({})", a.user, a.host), i == self.publish_account, true));
                }
                items.push(sep());
                items.push(("Add account...".into(), false, true));
                items.push(("Manage accounts...".into(), false, true));
            }
        }
        let mut chosen: Option<usize> = None;
        let area = egui::Area::new(egui::Id::new("rail_menu"))
            .order(egui::Order::Foreground)
            .pivot(Align2::LEFT_BOTTOM)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(p.bg)
                    .stroke(egui::Stroke::new(1.0_f32, p.border))
                    .rounding(10.0)
                    .shadow(egui::epaint::Shadow { offset: egui::vec2(0.0, 6.0), blur: 18.0, spread: 0.0, color: Color32::from_black_alpha(70) })
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.set_width(220.0);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (i, (label, checked, enabled)) in items.iter().enumerate() {
                            if label.is_empty() {
                                ui.add_space(4.0);
                                let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
                                ui.painter().rect_filled(r, 0.0, p.border);
                                ui.add_space(4.0);
                                continue;
                            }
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), Sense::click());
                            if *enabled && resp.hovered() {
                                ui.painter().rect_filled(rect, 6.0, p.hover);
                            }
                            let col = if *enabled { p.text } else { p.muted };
                            if *checked {
                                paint_icon(ui, "check", egui::Rect::from_center_size(rect.left_center() + egui::vec2(14.0, 0.0), egui::vec2(14.0, 14.0)), col);
                            }
                            let mut tx = 28.0;
                            if kind == 1 {
                                let sp = self.spaces.iter().find(|s| s.name == *label);
                                let ico = match (sp, label.as_str()) {
                                    (Some(s), _) => Some((space_icon_name(&s.icon), if s.color.is_some() { space_color32(s.color) } else { p.muted })),
                                    (None, "All repositories") => Some(("layers".to_string(), p.muted)),
                                    (None, "New space...") => Some(("add".to_string(), p.muted)),
                                    _ => None,
                                };
                                if let Some((n, c)) = ico {
                                    paint_icon(ui, &n, egui::Rect::from_center_size(rect.left_center() + egui::vec2(38.0, 0.0), egui::vec2(14.0, 14.0)), c);
                                }
                                tx = 54.0;
                            }
                            ui.painter().text(rect.left_center() + egui::vec2(tx, 0.0), Align2::LEFT_CENTER, label, FontId::proportional(13.5), col);
                            if *enabled && resp.clicked() {
                                chosen = Some(i);
                            }
                        }
                    });
            });
        let clicked_out = ctx.input(|i| i.pointer.any_click()) && ctx.input(|i| i.time) - opened > 0.15 && !area.response.hovered() && !ctx.is_pointer_over_area();
        let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if chosen.is_some() || clicked_out || esc {
            self.rail_menu = None;
        }
        let Some(i) = chosen else { return };
        let label = items[i].0.clone();
        let open = |app: &mut App, s: Section| {
            app.open_settings();
            app.section = s;
        };
        match (kind, label.as_str()) {
            (0, "Appearance...") => open(self, Section::Appearance),
            (0, "Accounts...") => open(self, Section::Accounts),
            (0, "Spaces...") => open(self, Section::Spaces),
            (0, "AI...") => open(self, Section::Ai),
            (0, "Help...") => open(self, Section::Help),
            (0, "Remotes...") => open(self, Section::Remotes),
            (0, "Ignored file history...") => {
                open(self, Section::Ignore);
                self.ignore_text = self.repo().map(|r| git::read_ignore(&r)).unwrap_or_default();
            }
            (0, _) => self.open_settings(),
            (1, "All repositories") => {
                self.set_space(None);
                self.screen = Screen::Home;
            }
            (1, "New space...") => {
                open(self, Section::Spaces);
                self.space_name.clear();
                self.space_icon = "layers".into();
                self.space_color = None;
                self.space_dialog = Some(None);
            }
            (1, "Manage spaces...") => open(self, Section::Spaces),
            (1, _) => {
                if let Some(k) = self.spaces.iter().position(|s| s.name == label) {
                    self.set_space(Some(k));
                    self.screen = Screen::Home;
                }
            }
            (_, "Add account...") => {
                open(self, Section::Accounts);
                self.acct_adding = true;
            }
            (_, "Manage accounts...") => open(self, Section::Accounts),
            _ => {
                // An account row: remember it as the default for publishing.
                self.publish_account = i.min(self.accounts.len().saturating_sub(1));
            }
        }
    }
}
