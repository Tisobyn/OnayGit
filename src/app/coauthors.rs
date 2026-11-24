//! Co-authors the user can add to a commit as `Co-authored-by` trailers.
#[allow(unused_imports)]
use crate::prelude::*;

fn path() -> PathBuf {
    home_dir().join(".onaygit_coauthors")
}

/// One per line, tab separated: `on` or `off`, name, email.
pub fn load() -> Vec<(git::Person, bool)> {
    let mut v: Vec<(git::Person, bool)> = std::fs::read_to_string(path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            let on = p.next()? == "on";
            let name = p.next()?.to_string();
            let email = p.next()?.to_string();
            (!email.is_empty()).then_some((git::Person { name, email }, on))
        })
        .collect();
    if v.is_empty() {
        v.push((git::Person { name: "Claude Sonnet 5.5".into(), email: "noreply@anthropic.com".into() }, false));
    }
    v
}

pub fn save(list: &[(git::Person, bool)]) {
    let text: String = list.iter().map(|(p, on)| format!("{}\t{}\t{}\n", if *on { "on" } else { "off" }, p.name, p.email)).collect();
    let _ = std::fs::write(path(), text);
}

impl App {
    /// People chosen as co-authors for the next commit.
    pub fn chosen_coauthors(&self) -> Vec<git::Person> {
        self.coauthors.iter().filter(|(_, on)| *on).map(|(p, _)| p.clone()).collect()
    }

    /// Co-authors seen in the loaded history that are not in the list yet.
    pub fn learn_coauthors(&mut self) {
        let mine = self.git_email.to_lowercase();
        let mut found: Vec<git::Person> = vec![];
        for c in &self.commits {
            for p in &c.coauthors {
                let e = p.email.to_lowercase();
                if !e.is_empty() && e != mine && !found.iter().any(|f| f.email.to_lowercase() == e) {
                    found.push(p.clone());
                }
            }
        }
        for p in found {
            if !self.coauthors.iter().any(|(x, _)| x.email.eq_ignore_ascii_case(&p.email)) {
                self.coauthors.push((p, false));
            }
        }
    }

    /// The popup with the co-author choices, opened from the commit box.
    pub fn coauthor_popup(&mut self, ctx: &egui::Context) {
        let Some(anchor) = self.coauthor_open else { return };
        let p = self.pal();
        let mut changed = false;
        let mut remove: Option<usize> = None;
        let mut add = false;
        let area = egui::Area::new(egui::Id::new("coauthor_popup"))
            .order(egui::Order::Foreground)
            .pivot(Align2::LEFT_BOTTOM)
            .fixed_pos(anchor)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(p.bg)
                    .stroke(egui::Stroke::new(1.0_f32, p.border))
                    .rounding(10.0)
                    .shadow(egui::epaint::Shadow { offset: egui::vec2(0.0, 6.0), blur: 18.0, spread: 0.0, color: Color32::from_black_alpha(70) })
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(280.0);
                        ui.label(RichText::new("Co-authors").size(14.0).strong().color(p.text));
                        ui.label(RichText::new("Added to the commit message as Co-authored-by.").size(12.0).color(p.muted));
                        ui.add_space(6.0);
                        for (i, (person, on)) in self.coauthors.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if round_check(ui, &p, *on).clicked() {
                                    *on = !*on;
                                    changed = true;
                                }
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.label(RichText::new(&person.name).size(13.0).color(p.text));
                                    ui.label(RichText::new(&person.email).size(11.5).color(p.muted));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let (r, resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                                    if resp.hovered() {
                                        ui.painter().rect_filled(r, 5.0, p.hover);
                                    }
                                    paint_icon(ui, "close", r.shrink(5.0), p.muted);
                                    if resp.on_hover_text("Remove from list").clicked() {
                                        remove = Some(i);
                                    }
                                });
                            });
                        }
                        ui.add_space(6.0);
                        framed(ui, &p, egui::TextEdit::singleline(&mut self.coauthor_name).hint_text("Name").desired_width(f32::INFINITY));
                        framed(ui, &p, egui::TextEdit::singleline(&mut self.coauthor_email).hint_text("Email").desired_width(f32::INFINITY));
                        let ok = !self.coauthor_name.trim().is_empty() && self.coauthor_email.contains('@');
                        ui.add_space(2.0);
                        if ui.add_enabled(ok, egui::Button::new("Add co-author")).clicked() {
                            add = true;
                        }
                    });
            });
        if add {
            let person = git::Person { name: self.coauthor_name.trim().to_string(), email: self.coauthor_email.trim().to_string() };
            if let Some(e) = self.coauthors.iter_mut().find(|(x, _)| x.email.eq_ignore_ascii_case(&person.email)) {
                e.1 = true;
            } else {
                self.coauthors.push((person, true));
            }
            self.coauthor_name.clear();
            self.coauthor_email.clear();
            changed = true;
        }
        if let Some(i) = remove {
            self.coauthors.remove(i);
            changed = true;
        }
        if changed {
            save(&self.coauthors);
        }
        // A click outside closes it.
        let clicked_out = ctx.input(|i| i.pointer.any_pressed()) && ctx.pointer_interact_pos().is_some_and(|pos| !area.response.rect.contains(pos));
        if clicked_out || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.coauthor_open = None;
        }
    }
}
