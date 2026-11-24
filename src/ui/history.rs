//! History tab: compare bar, commit list with graph, files of the selected commit.
#[allow(unused_imports)]
use crate::prelude::*;

impl App {
    pub fn history_tab(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut new_compare: Option<Option<String>> = None;
        let mut pick: Option<String> = None;
        let mut pick_file: Option<Option<String>> = None;
        let mut action: Option<(CommitAction, String)> = None;

        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let label = match &self.compare {
                Some(b) => format!("Compare: {b}"),
                None => "Compare to branch...".to_string(),
            };
            ui.menu_button(label, |ui| {
                ui.set_min_width(200.0);
                if ui.button("None (show history)").clicked() {
                    new_compare = Some(None);
                    ui.close_menu();
                }
                if let Some(u) = &self.upstream {
                    if ui.button(format!("Review upstream ({u})")).clicked() {
                        new_compare = Some(Some(u.clone()));
                        ui.close_menu();
                    }
                }
                ui.separator();
                let others = self.branches.iter().filter(|b| **b != self.branch);
                for b in others.chain(self.remote_branches.iter()) {
                    if ui.button(b).clicked() {
                        new_compare = Some(Some(b.clone()));
                        ui.close_menu();
                    }
                }
            });
        });
        let mut direction: Option<bool> = None;
        let mut integrate: Option<bool> = None;
        if let Some(b) = &self.compare {
            let (ours, theirs) = self.compare_counts;
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.selectable_label(!self.compare_outgoing, format!("Incoming {theirs}")).clicked() {
                    direction = Some(false);
                }
                if ui.selectable_label(self.compare_outgoing, format!("Outgoing {ours}")).clicked() {
                    direction = Some(true);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(6.0);
                    if ui.add_enabled(self.busy.is_none(), egui::Button::new("Rebase").small()).on_hover_text(format!("Rebase onto {b}")).clicked() {
                        integrate = Some(false);
                    }
                    if ui.add_enabled(self.busy.is_none(), egui::Button::new("Merge").small()).on_hover_text(format!("Merge {b} into {}", self.branch)).clicked() {
                        integrate = Some(true);
                    }
                });
            });
        }
        ui.separator();

        // Files of the selected commit, pinned at the bottom.
        if self.selected_commit.is_some() {
            egui::TopBottomPanel::bottom("commit_files")
                .resizable(true)
                .default_height(150.0)
                .frame(egui::Frame::none().fill(p.sidebar))
                .show_inside(ui, |ui| {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        let n = self.commit_files.len();
                        ui.label(RichText::new(format!("{n} changed file{}", if n == 1 { "" } else { "s" })).size(12.0).color(p.muted));
                        if self.selected_commit_file.is_some() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(6.0);
                                if ui.small_button("Show all").clicked() {
                                    pick_file = Some(None);
                                }
                            });
                        }
                    });
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (st, path) in &self.commit_files {
                            let sel = self.selected_commit_file.as_deref() == Some(path.as_str());
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), Sense::click());
                            let fill = if sel { accent() } else if resp.hovered() { p.hover } else { Color32::TRANSPARENT };
                            ui.painter().rect_filled(rect, 0.0, fill);
                            let fg = if sel { Color32::WHITE } else { p.text };
                            let badge = match st {
                                'A' => Color32::from_rgb(63, 185, 80),
                                'D' => Color32::from_rgb(248, 81, 73),
                                _ => Color32::from_rgb(210, 153, 34),
                            };
                            let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(4.0, 0.0)));
                            clip.text(rect.min + egui::vec2(10.0, 11.0), Align2::LEFT_CENTER, path, FontId::proportional(12.5), fg);
                            ui.painter().text(
                                rect.right_center() - egui::vec2(10.0, 0.0),
                                Align2::RIGHT_CENTER,
                                st.to_string(),
                                FontId::proportional(12.0),
                                if sel { Color32::WHITE } else { badge },
                            );
                            if resp.clicked() {
                                pick_file = Some(Some(path.clone()));
                            }
                        }
                    });
                });
        }

        let comparing = self.compare.is_some();
        let list: &Vec<Commit> = if comparing { &self.compare_commits } else { &self.commits };
        let graph_w = if comparing { 0.0 } else { graph_px(self.graph_width) };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, c) in list.iter().enumerate() {
                let sel = self.selected_commit.as_deref() == Some(c.hash.as_str());
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), Sense::click());
                let fill = if sel { accent() } else if resp.hovered() { p.hover } else { Color32::TRANSPARENT };
                ui.painter().rect_filled(rect, 0.0, fill);
                if !comparing {
                    if let Some(row) = self.graph.get(i) {
                        paint_graph_row(&ui.painter().with_clip_rect(rect), rect, row);
                    }
                }
                let fg = if sel { Color32::WHITE } else { p.text };
                let dim = if sel { Color32::from_rgb(220, 235, 255) } else { p.muted };
                let x = rect.min.x + if comparing { 10.0 } else { graph_w + 2.0 };
                let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)));
                clip.text(egui::pos2(x, rect.min.y + 12.0), Align2::LEFT_CENTER, &c.subject, FontId::proportional(13.0), fg);
                clip.text(
                    egui::pos2(x, rect.min.y + 28.0),
                    Align2::LEFT_CENTER,
                    format!("{} · {}", c.author, c.when),
                    FontId::proportional(11.0),
                    dim,
                );
                if resp.clicked() {
                    pick = Some(c.hash.clone());
                }
                resp.context_menu(|ui| {
                    let h = c.hash.clone();
                    if ui.button("Create tag here...").clicked() {
                        action = Some((CommitAction::Tag, h.clone()));
                        ui.close_menu();
                    }
                    if ui.button("Cherry-pick").clicked() {
                        action = Some((CommitAction::CherryPick, h.clone()));
                        ui.close_menu();
                    }
                    if ui.button("Revert").clicked() {
                        action = Some((CommitAction::Revert, h.clone()));
                        ui.close_menu();
                    }
                    ui.menu_button("Reset current branch to here", |ui| {
                        for (label, a) in [
                            ("Soft (keep changes staged)", CommitAction::ResetSoft),
                            ("Mixed (keep changes)", CommitAction::ResetMixed),
                            ("Hard (discard changes)", CommitAction::ResetHard),
                        ] {
                            if ui.button(label).clicked() {
                                action = Some((a, h.clone()));
                                ui.close_menu();
                            }
                        }
                    });
                    ui.add_enabled_ui(!comparing, |ui| {
                        ui.menu_button("Rewrite history", |ui| {
                            for (label, a) in [
                                ("Reword...", CommitAction::Reword),
                                ("Squash into previous", CommitAction::Squash),
                                ("Move newer", CommitAction::MoveNewer),
                                ("Move older", CommitAction::MoveOlder),
                                ("Drop commit", CommitAction::Drop),
                            ] {
                                if ui.button(label).clicked() {
                                    action = Some((a, h.clone()));
                                    ui.close_menu();
                                }
                            }
                        });
                    });
                    ui.separator();
                    if ui.button("Copy commit hash").clicked() {
                        ui.ctx().copy_text(h);
                        ui.close_menu();
                    }
                });
            }
            if list.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No commits yet").color(p.muted));
                });
            }
        });

        if let Some(o) = direction {
            self.compare_outgoing = o;
            self.selected_commit = None;
            self.commit_diff.clear();
            self.refresh();
        }
        if let (Some(merge), Some(other), Some(repo)) = (integrate, self.compare.clone(), self.repo()) {
            if merge {
                let r = git::merge(&repo, &other);
                self.report(r);
                self.refresh();
            } else {
                self.rebase_onto = Some(other);
            }
        }
        if let Some(c) = new_compare {
            self.set_compare(c);
        }
        if let Some(h) = pick {
            self.select_commit(h);
        }
        if let Some(f) = pick_file {
            self.select_commit_file(f);
        }
        if let Some((a, h)) = action {
            self.commit_action(a, h);
        }
    }
}
