//! Tab bar with the open repositories.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn tab_bar(&mut self, ctx: &egui::Context) {
        if self.tabs.is_empty() {
            return;
        }
        let p = self.pal();
        let mut activate: Option<usize> = None;
        let mut close: Option<usize> = None;
        let mut new_tab = false;
        let current = self.repo();

        egui::TopBottomPanel::top("tab_bar")
            .exact_height(30.0)
            .frame(egui::Frame::none().fill(p.sidebar).stroke(egui::Stroke::new(1.0_f32, p.border)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.add_space(6.0);
                    for (i, path) in self.tabs.iter().enumerate() {
                        let name = repo_name(path);
                        let style = self.style_of(path);
                        let active = current.as_ref() == Some(path) && self.screen == Screen::Repo;
                        let text_w = ui.fonts(|f| {
                            f.layout_no_wrap(name.clone(), FontId::proportional(12.5), p.text).size().x
                        });
                        let w = 34.0 + text_w + 22.0;
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 24.0), Sense::click());
                        if active {
                            ui.painter().rect_filled(rect, 6.0, p.bg);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect, 6.0, p.hover);
                        }
                        let icon_rect = egui::Rect::from_center_size(rect.min + egui::vec2(14.0, 12.0), egui::vec2(14.0, 14.0));
                        paint_icon(ui, &style.icon, icon_rect, ACCENTS[style.color % ACCENTS.len()].1);
                        ui.painter().text(
                            rect.min + egui::vec2(28.0, 12.0),
                            Align2::LEFT_CENTER,
                            &name,
                            FontId::proportional(12.5),
                            if active { p.text } else { p.muted },
                        );
                        let x_rect = egui::Rect::from_center_size(rect.max - egui::vec2(12.0, 12.0), egui::vec2(12.0, 12.0));
                        let over_x = resp.hover_pos().is_some_and(|pos| x_rect.expand(3.0).contains(pos));
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
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(rect, 6.0, p.hover);
                    }
                    paint_icon(ui, "add", rect.shrink(5.0), p.muted);
                    if resp.on_hover_text("Open another repository (Cmd+T)").clicked() {
                        new_tab = true;
                    }
                });
            });

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

    /// Cmd+W closes the tab, Cmd+T opens the repository list.
    pub fn tab_shortcuts(&mut self, ctx: &egui::Context) {
        let (close, new_tab, quick, prefs, tab1, tab2) = ctx.input(|i| {
            let c = i.modifiers.command;
            (
                c && i.key_pressed(egui::Key::W),
                c && i.key_pressed(egui::Key::T),
                c && i.key_pressed(egui::Key::P),
                c && i.key_pressed(egui::Key::Comma),
                c && i.key_pressed(egui::Key::Num1),
                c && i.key_pressed(egui::Key::Num2),
            )
        });
        if quick {
            self.switcher_text.clear();
            self.dialog = Some(Dialog::Switcher);
        }
        if prefs {
            self.open_settings();
        }
        if self.screen == Screen::Repo {
            if tab1 {
                self.tab = Tab::Changes;
            }
            if tab2 {
                self.tab = Tab::History;
            }
        }
        if new_tab {
            self.screen = Screen::Home;
        }
        if close && self.screen == Screen::Repo {
            if let Some(r) = self.repo() {
                if let Some(i) = self.tabs.iter().position(|t| *t == r) {
                    self.close_tab(i);
                }
            }
        }
    }
}
