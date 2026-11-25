//! Banner shown while a merge, rebase, cherry-pick or revert is waiting for you.
use crate::prelude::*;

impl App {
    pub fn conflict_banner(&mut self, ui: &mut egui::Ui) {
        let Some(op) = self.repo_op else { return };
        let conflicts = self.files.iter().filter(|f| f.conflicted()).count();
        let mut cont = false;
        let mut abort = false;
        egui::Frame::none()
            .fill(Color32::from_rgb(110, 76, 20))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let text = if conflicts > 0 {
                    format!("{} in progress: {conflicts} conflicted file{}", op.label(), if conflicts == 1 { "" } else { "s" })
                } else {
                    format!("{} in progress: all conflicts resolved", op.label())
                };
                ui.label(RichText::new(text).color(Color32::WHITE).size(12.5));
                ui.horizontal(|ui| {
                    if ui.add_enabled(conflicts == 0, egui::Button::new("Continue").small()).clicked() {
                        cont = true;
                    }
                    if ui.add(egui::Button::new("Abort").small()).clicked() {
                        abort = true;
                    }
                });
            });
        if cont {
            self.continue_op();
        }
        if abort {
            self.confirm = Some(Confirm::AbortOp);
        }
    }
}

const OURS_BG: Color32 = Color32::from_rgba_premultiplied(60, 20, 18, 60);
const THEIRS_BG: Color32 = Color32::from_rgba_premultiplied(14, 46, 20, 60);

fn code_block(ui: &mut egui::Ui, p: &Pal, font: f32, bg: Color32, title: &str, lines: &[String]) {
    egui::Frame::none().fill(bg).rounding(6.0).inner_margin(egui::Margin::symmetric(10.0, 8.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(title).size(11.5).color(p.muted));
        if lines.is_empty() {
            ui.label(RichText::new("(empty)").size(font).color(p.muted));
        }
        for l in lines {
            ui.label(RichText::new(if l.is_empty() { " " } else { l }).monospace().size(font).color(p.text));
        }
    });
}

fn context(ui: &mut egui::Ui, p: &Pal, font: f32, lines: &[String]) {
    for l in lines {
        ui.label(RichText::new(if l.is_empty() { " " } else { l }).monospace().size(font).color(p.muted));
    }
}

/// Block by block conflict editor. Returns true when the user presses Save.
pub fn conflict_editor(ui: &mut egui::Ui, p: &Pal, font: f32, segs: &[git::Seg], picks: &mut [git::Pick], edits: &mut [String]) -> bool {
    use git::Pick;
    let total = picks.len();
    let left = picks.iter().filter(|k| **k == Pick::Undecided).count();
    let mut save = false;
    egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 8.0)).show(ui, |ui| {
        ui.horizontal(|ui| {
            let t = if left == 0 { "All conflicts decided".to_string() } else { format!("{left} of {total} conflicts left") };
            ui.label(RichText::new(t).size(13.0).color(p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(left == 0, egui::Button::new("Save and mark resolved")).clicked() {
                    save = true;
                }
            });
        });
    });
    ui.separator();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(6.0);
        let mut c = 0;
        for (i, seg) in segs.iter().enumerate() {
            match seg {
                git::Seg::Text(t) => {
                    let before_conflict = matches!(segs.get(i + 1), Some(git::Seg::Conflict { .. }));
                    let after_conflict = i > 0 && matches!(segs.get(i - 1), Some(git::Seg::Conflict { .. }));
                    egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 0.0)).show(ui, |ui| {
                        if t.len() <= 4 {
                            context(ui, p, font, t);
                            return;
                        }
                        if after_conflict {
                            context(ui, p, font, &t[..2]);
                        }
                        ui.label(RichText::new(format!("   \u{2026} {} unchanged lines", t.len() - if after_conflict { 2 } else { 0 } - if before_conflict { 2 } else { 0 })).size(11.5).color(p.muted));
                        if before_conflict {
                            context(ui, p, font, &t[t.len() - 2..]);
                        }
                    });
                }
                git::Seg::Conflict { ours_label, theirs_label, ours, theirs } => {
                    let k = c;
                    c += 1;
                    egui::Frame::none().inner_margin(egui::Margin::symmetric(14.0, 6.0)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Conflict {} of {total}", k + 1)).size(13.0).strong().color(p.text));
                            let state = match picks[k] {
                                Pick::Undecided => "not decided",
                                Pick::Ours => "keeping ours",
                                Pick::Theirs => "keeping theirs",
                                Pick::Both => "keeping both",
                                Pick::Custom => "edited by hand",
                            };
                            ui.label(RichText::new(state).size(12.0).color(if picks[k] == Pick::Undecided { Color32::from_rgb(230, 160, 60) } else { Color32::from_rgb(63, 185, 80) }));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Edit").clicked() {
                                    if picks[k] != Pick::Custom {
                                        edits[k] = ours.iter().chain(theirs.iter()).cloned().collect::<Vec<_>>().join("\n");
                                    }
                                    picks[k] = Pick::Custom;
                                }
                                if ui.small_button("Both").clicked() {
                                    picks[k] = Pick::Both;
                                }
                                if ui.small_button("Theirs").clicked() {
                                    picks[k] = Pick::Theirs;
                                }
                                if ui.small_button("Ours").clicked() {
                                    picks[k] = Pick::Ours;
                                }
                            });
                        });
                        ui.add_space(4.0);
                        if picks[k] == Pick::Custom {
                            ui.add(
                                egui::TextEdit::multiline(&mut edits[k])
                                    .font(egui::FontId::monospace(font))
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(4),
                            );
                        } else {
                            let ol = if ours_label.is_empty() { "Ours".to_string() } else { format!("Ours ({ours_label})") };
                            let tl = if theirs_label.is_empty() { "Theirs".to_string() } else { format!("Theirs ({theirs_label})") };
                            ui.columns(2, |cols| {
                                code_block(&mut cols[0], p, font, OURS_BG, &ol, ours);
                                code_block(&mut cols[1], p, font, THEIRS_BG, &tl, theirs);
                            });
                        }
                    });
                }
            }
        }
        ui.add_space(12.0);
    });
    save
}
