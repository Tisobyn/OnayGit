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
            ui.menu_button(RichText::new(label).size(12.5).color(p.muted), |ui| {
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
        if self.filter_open {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                let r = ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter commits").desired_width(ui.available_width() - 10.0));
                if self.focus_filter {
                    r.request_focus();
                    self.focus_filter = false;
                }
            });
        }
        ui.add_space(2.0);

        // Hint line at the very bottom.
        egui::TopBottomPanel::bottom("history_hint")
            .exact_height(30.0)
            .frame(egui::Frame::none().fill(p.sidebar).inner_margin(egui::Margin::symmetric(14.0, 0.0)))
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("Read-only \u{b7} first-parent diffs").size(11.0).color(p.muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Navigate").size(11.0).color(p.muted));
                        for name in ["arrowDown", "arrowUp"] {
                            let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 12.0), Sense::hover());
                            paint_icon(ui, name, r, p.muted);
                        }
                    });
                });
            });

        // Files of the selected commit, pinned at the bottom.
        if self.selected_commit.is_some() {
            egui::TopBottomPanel::bottom("commit_files")
                .resizable(true)
                .default_height(170.0)
                .frame(egui::Frame::none().fill(p.sidebar))
                .show_inside(ui, |ui| {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.label(RichText::new("Changed files").size(12.5).color(p.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new(self.commit_files.len().to_string()).size(12.5).color(p.muted));
                            if self.selected_commit_file.is_some() && ui.add(egui::Button::new(RichText::new("Show all").size(12.0).color(p.muted)).small().frame(false)).clicked() {
                                pick_file = Some(None);
                            }
                        });
                    });
                    ui.add_space(2.0);
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (st, path) in &self.commit_files {
                            let sel = self.selected_commit_file.as_deref() == Some(path.as_str());
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), Sense::click());
                            if sel || resp.hovered() {
                                ui.painter().rect_filled(rect.shrink2(egui::vec2(6.0, 1.0)), 6.0, if sel { p.hover } else { p.hover.gamma_multiply(0.5) });
                            }
                            let mut badge_ui = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(egui::pos2(rect.min.x + 16.0, rect.center().y - 7.0), egui::vec2(14.0, 14.0))));
                            status_badge(&mut badge_ui, *st);
                            let (dir, name) = match path.rsplit_once('/') {
                                Some((d, n)) => (d.to_string(), n.to_string()),
                                None => (String::new(), path.clone()),
                            };
                            let clip = ui.painter().with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)));
                            let g = clip.layout_no_wrap(name, FontId::proportional(13.5), p.text);
                            let nw = g.size().x;
                            clip.galley(egui::pos2(rect.min.x + 40.0, rect.center().y - g.size().y / 2.0), g, p.text);
                            if !dir.is_empty() {
                                clip.text(egui::pos2(rect.min.x + 48.0 + nw, rect.center().y), Align2::LEFT_CENTER, dir, FontId::proportional(12.0), p.muted);
                            }
                            if resp.clicked() {
                                pick_file = Some(Some(path.clone()));
                            }
                        }
                    });
                });
        }

        let comparing = self.compare.is_some();
        let list: &Vec<Commit> = if comparing { &self.compare_commits } else { &self.commits };
        let needs_graph = !comparing && self.commits.iter().any(|c| c.parents.len() > 1);
        let graph_w = if needs_graph { graph_px(self.graph_width) } else { 0.0 };
        let q = self.filter.to_lowercase();
        let unpushed = match self.ahead_behind {
            Some((a, _)) => a as usize,
            None => usize::MAX,
        };
        // Up and down arrows move through the commits.
        if !ui.ctx().wants_keyboard_input() {
            let (down, up) = ui.input(|i| (i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowUp)));
            if down || up {
                let cur = list.iter().position(|c| Some(c.hash.as_str()) == self.selected_commit.as_deref());
                let next = match (cur, down) {
                    (None, _) => Some(0),
                    (Some(i), true) => (i + 1 < list.len()).then_some(i + 1),
                    (Some(i), false) => i.checked_sub(1),
                };
                if let Some(c) = next.and_then(|i| list.get(i)) {
                    pick = Some(c.hash.clone());
                }
            }
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, c) in list.iter().enumerate() {
                if !q.is_empty() && !(c.subject.to_lowercase().contains(&q) || c.author.to_lowercase().contains(&q) || c.hash.starts_with(&q)) {
                    continue;
                }
                let sel = self.selected_commit.as_deref() == Some(c.hash.as_str());
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 54.0), Sense::click());
                if sel || resp.hovered() {
                    ui.painter().rect_filled(rect.shrink2(egui::vec2(6.0, 1.0)), 6.0, if sel { p.hover } else { p.hover.gamma_multiply(0.5) });
                }
                if needs_graph {
                    if let Some(row) = self.graph.get(i) {
                        paint_graph_row(&ui.painter().with_clip_rect(rect), rect, row);
                    }
                }
                let x = rect.min.x + if needs_graph { graph_w + 2.0 } else { 16.0 };
                let right = if !comparing && i < unpushed { 36.0 } else { 12.0 };
                let clip = ui.painter().with_clip_rect(egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x - right, rect.max.y)));
                match self.settings.commit_badges.then(|| split_conventional(&c.subject)).flatten() {
                    Some((kind, rest)) => {
                        let galley = ui.painter().layout_no_wrap(kind.to_string(), FontId::proportional(11.0), badge_color(kind));
                        let br = egui::Rect::from_min_size(egui::pos2(x, rect.min.y + 9.0), galley.size() + egui::vec2(12.0, 4.0));
                        clip.rect_filled(br, 4.0, badge_color(kind).gamma_multiply(0.18));
                        clip.galley(br.min + egui::vec2(6.0, 2.0), galley, p.text);
                        clip.text(egui::pos2(br.max.x + 8.0, rect.min.y + 18.0), Align2::LEFT_CENTER, rest, FontId::proportional(13.5), p.text);
                    }
                    None => {
                        clip.text(egui::pos2(x, rect.min.y + 18.0), Align2::LEFT_CENTER, &c.subject, FontId::proportional(13.5), p.text);
                    }
                }
                let short: String = c.hash.chars().take(7).collect();
                clip.text(egui::pos2(x, rect.min.y + 37.0), Align2::LEFT_CENTER, format!("{} \u{b7} {} \u{b7} {short}", c.author, c.when), FontId::proportional(11.5), p.muted);
                if !comparing && i < unpushed {
                    paint_icon(ui, "arrowUp", egui::Rect::from_center_size(egui::pos2(rect.max.x - 24.0, rect.center().y), egui::vec2(14.0, 14.0)), p.muted);
                }
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

/// Splits `feat(core): add x` into (`feat`, `(core) add x`).
pub fn split_conventional(subject: &str) -> Option<(&str, String)> {
    const KINDS: [&str; 11] = ["feat", "fix", "refactor", "docs", "chore", "test", "perf", "build", "ci", "style", "revert"];
    let (head, rest) = subject.split_once(": ")?;
    let head = head.trim_end_matches('!');
    let (kind, scope) = match head.split_once('(') {
        Some((k, s)) => (k, Some(s.strip_suffix(')')?)),
        None => (head, None),
    };
    KINDS.contains(&kind).then(|| (kind, match scope { Some(s) => format!("({s}) {rest}"), None => rest.to_string() }))
}

pub fn badge_color(kind: &str) -> Color32 {
    match kind {
        "feat" => Color32::from_rgb(98, 174, 134),
        "fix" => Color32::from_rgb(232, 121, 130),
        "refactor" => Color32::from_rgb(164, 120, 239),
        "docs" => Color32::from_rgb(82, 139, 255),
        "perf" => Color32::from_rgb(233, 152, 85),
        "test" => Color32::from_rgb(217, 179, 74),
        _ => Color32::from_rgb(130, 148, 179),
    }
}
