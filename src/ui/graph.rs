//! Commit graph drawing for one history row.
use crate::prelude::*;

pub const LANE_W: f32 = 10.0;
const MAX_LANES: usize = 8;

const COLORS: [Color32; 6] = [
    Color32::from_rgb(33, 136, 255),
    Color32::from_rgb(63, 185, 80),
    Color32::from_rgb(210, 153, 34),
    Color32::from_rgb(163, 113, 247),
    Color32::from_rgb(248, 81, 73),
    Color32::from_rgb(56, 189, 196),
];

/// Width in pixels the graph column needs.
pub fn graph_px(lanes: usize) -> f32 {
    lanes.min(MAX_LANES) as f32 * LANE_W + 12.0
}

pub fn paint_graph_row(painter: &egui::Painter, rect: egui::Rect, row: &git::GraphRow) {
    let x = |lane: usize| rect.min.x + 10.0 + lane as f32 * LANE_W;
    let (top, mid, bot) = (rect.min.y, rect.center().y, rect.max.y);
    let color = |lane: usize| COLORS[lane % COLORS.len()];
    let stroke = |lane: usize| egui::Stroke::new(1.5_f32, color(lane));
    let line = |a: egui::Pos2, b: egui::Pos2, lane: usize| painter.line_segment([a, b], stroke(lane));
    let lanes = row.top.len().max(row.bottom.len()).min(MAX_LANES);

    for i in 0..lanes {
        if i == row.col {
            continue;
        }
        let (t, b) = (row.top.get(i) == Some(&true), row.bottom.get(i) == Some(&true));
        if t {
            if row.merge_from.contains(&i) {
                line(egui::pos2(x(i), top), egui::pos2(x(row.col), mid), i);
            } else {
                line(egui::pos2(x(i), top), egui::pos2(x(i), mid), i);
            }
        }
        if b {
            if row.branch_to.contains(&i) {
                line(egui::pos2(x(row.col), mid), egui::pos2(x(i), bot), i);
                if t {
                    line(egui::pos2(x(i), mid), egui::pos2(x(i), bot), i);
                }
            } else {
                line(egui::pos2(x(i), mid), egui::pos2(x(i), bot), i);
            }
        }
    }
    if row.col < MAX_LANES {
        if row.top.get(row.col) == Some(&true) {
            line(egui::pos2(x(row.col), top), egui::pos2(x(row.col), mid), row.col);
        }
        if row.bottom.get(row.col) == Some(&true) {
            line(egui::pos2(x(row.col), mid), egui::pos2(x(row.col), bot), row.col);
        }
        painter.circle_filled(egui::pos2(x(row.col), mid), 3.5, color(row.col));
    }
}
