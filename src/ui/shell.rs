//! Window frame in the style of 1git: tabs in the title bar, an icon rail on the left
//! and a rounded content card.
#[allow(unused_imports)]
use crate::prelude::*;

const TITLE_H: f32 = 40.0;
const RAIL_W: f32 = 52.0;
const GAP: f32 = 8.0;
const RADIUS: f32 = 12.0;

fn icon_button(ui: &mut egui::Ui, name: &str, tip: &str, selected: bool, p: &Pal) -> egui::Response {
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
    paint_icon(ui, name, rect.shrink(9.0), if selected { p.text } else { p.muted });
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
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(tw + 70.0, 34.0), Sense::click());
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
        let mut dialog: Option<Dialog> = None;
        let mut settings = false;
        let mut home = false;
        let mut accounts = false;
        let mut prs = false;
        egui::SidePanel::left("rail")
            .exact_width(RAIL_W)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(p.chrome).inner_margin(egui::Margin::symmetric(8.0, 6.0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                if self.screen == Screen::Repo {
                    if icon_button(ui, "branch", "Changes", self.tab == Tab::Changes, p).clicked() {
                        new_tab = Some(Tab::Changes);
                    }
                    if icon_button(ui, "history", "History", self.tab == Tab::History, p).clicked() {
                        new_tab = Some(Tab::History);
                    }
                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);
                    if icon_button(ui, "pull", "Pull requests", false, p).clicked() {
                        prs = true;
                    }
                    if icon_button(ui, "tree", "Worktrees", false, p).clicked() {
                        dialog = Some(Dialog::Worktrees);
                    }
                    if icon_button(ui, "archive", "Tags", false, p).clicked() {
                        dialog = Some(Dialog::Tags);
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    if icon_button(ui, "github", "Accounts", false, p).clicked() {
                        accounts = true;
                    }
                    if icon_button(ui, "settings", "Settings (Cmd+,)", self.screen == Screen::Settings, p).clicked() {
                        settings = true;
                    }
                    if icon_button(ui, "home", "Repositories", self.screen == Screen::Home, p).clicked() {
                        home = true;
                    }
                });
            });
        if let Some(t) = new_tab {
            self.tab = t;
        }
        if dialog.is_some() {
            self.dialog = dialog;
        }
        if prs {
            self.open_pull_requests(ctx);
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
