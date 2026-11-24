#[allow(unused_imports)]
use crate::prelude::*;

pub const ACCENT: Color32 = Color32::from_rgb(33, 136, 255);

// ---------------------------------------------------------------- theme

#[derive(Clone, Copy)]
pub struct Pal {
    pub bg: Color32,
    pub sidebar: Color32,
    pub toolbar: Color32,
    pub home: Color32,
    pub hover: Color32,
    pub input: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub add_bg: Color32,
    pub del_bg: Color32,
    pub add_gut: Color32,
    pub del_gut: Color32,
    pub hunk_bg: Color32,
}

pub fn pal(dark: bool) -> Pal {
    if dark {
        Pal {
            bg: Color32::from_rgb(36, 41, 46),
            sidebar: Color32::from_rgb(31, 36, 40),
            toolbar: Color32::from_rgb(36, 41, 46),
            home: Color32::from_rgb(32, 33, 40),
            hover: Color32::from_rgb(45, 46, 55),
            input: Color32::from_rgb(24, 27, 31),
            text: Color32::from_rgb(225, 228, 232),
            muted: Color32::from_rgb(149, 157, 165),
            border: Color32::from_rgb(58, 64, 70),
            add_bg: Color32::from_rgb(25, 58, 38),
            del_bg: Color32::from_rgb(70, 30, 34),
            add_gut: Color32::from_rgb(34, 84, 52),
            del_gut: Color32::from_rgb(100, 40, 46),
            hunk_bg: Color32::from_rgb(40, 48, 58),
        }
    } else {
        Pal {
            bg: Color32::from_rgb(255, 255, 255),
            sidebar: Color32::from_rgb(246, 248, 250),
            toolbar: Color32::from_rgb(246, 248, 250),
            home: Color32::from_rgb(250, 250, 252),
            hover: Color32::from_rgb(232, 235, 239),
            input: Color32::WHITE,
            text: Color32::from_rgb(36, 41, 46),
            muted: Color32::from_rgb(87, 96, 106),
            border: Color32::from_rgb(208, 215, 222),
            add_bg: Color32::from_rgb(230, 255, 236),
            del_bg: Color32::from_rgb(255, 235, 233),
            add_gut: Color32::from_rgb(204, 240, 214),
            del_gut: Color32::from_rgb(255, 215, 213),
            hunk_bg: Color32::from_rgb(221, 244, 255),
        }
    }
}

pub fn apply_visuals(ctx: &egui::Context, dark: bool) {
    let p = pal(dark);
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.bg;
    v.extreme_bg_color = p.input;
    v.selection.bg_fill = ACCENT;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
    ctx.set_visuals(v);
}

// ---------------------------------------------------------------- settings
