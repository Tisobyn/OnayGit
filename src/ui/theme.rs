#[allow(unused_imports)]
use crate::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

// ---------------------------------------------------------------- accent

/// Accent colors the user can pick in Appearance settings.
pub const ACCENTS: [(&str, Color32); 7] = [
    ("Blue", Color32::from_rgb(33, 136, 255)),
    ("Green", Color32::from_rgb(46, 160, 67)),
    ("Purple", Color32::from_rgb(137, 87, 229)),
    ("Pink", Color32::from_rgb(219, 97, 162)),
    ("Red", Color32::from_rgb(218, 54, 51)),
    ("Amber", Color32::from_rgb(191, 135, 0)),
    ("Teal", Color32::from_rgb(31, 164, 172)),
];

static ACCENT_RGB: AtomicU32 = AtomicU32::new(0x2188ff);

/// The current accent color.
pub fn accent() -> Color32 {
    let v = ACCENT_RGB.load(Ordering::Relaxed);
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub fn set_accent(index: usize) {
    let c = ACCENTS[index.min(ACCENTS.len() - 1)].1;
    ACCENT_RGB.store(((c.r() as u32) << 16) | ((c.g() as u32) << 8) | c.b() as u32, Ordering::Relaxed);
}

// ---------------------------------------------------------------- themes

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    Dark,
    Midnight,
    Nord,
    Light,
    Sand,
}

impl Theme {
    pub const ALL: [Theme; 5] = [Theme::Dark, Theme::Midnight, Theme::Nord, Theme::Light, Theme::Sand];

    pub fn label(self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Midnight => "Midnight",
            Theme::Nord => "Nord",
            Theme::Light => "Light",
            Theme::Sand => "Sand",
        }
    }

    pub fn key(self) -> &'static str {
        self.label()
    }

    pub fn from_key(k: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.key() == k)
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Theme::Dark | Theme::Midnight | Theme::Nord)
    }
}

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

const fn c(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub fn pal(theme: Theme) -> Pal {
    match theme {
        Theme::Dark => Pal {
            bg: c(36, 41, 46),
            sidebar: c(31, 36, 40),
            toolbar: c(36, 41, 46),
            home: c(32, 33, 40),
            hover: c(45, 46, 55),
            input: c(24, 27, 31),
            text: c(225, 228, 232),
            muted: c(149, 157, 165),
            border: c(58, 64, 70),
            add_bg: c(27, 48, 38),
            del_bg: c(55, 34, 38),
            add_gut: c(32, 62, 45),
            del_gut: c(72, 40, 46),
            hunk_bg: c(36, 42, 50),
        },
        Theme::Midnight => Pal {
            bg: c(13, 17, 23),
            sidebar: c(10, 13, 18),
            toolbar: c(13, 17, 23),
            home: c(9, 12, 16),
            hover: c(30, 36, 46),
            input: c(6, 8, 12),
            text: c(230, 237, 243),
            muted: c(125, 133, 144),
            border: c(38, 44, 54),
            add_bg: c(16, 36, 28),
            del_bg: c(44, 24, 28),
            add_gut: c(22, 54, 38),
            del_gut: c(62, 30, 36),
            hunk_bg: c(22, 32, 48),
        },
        Theme::Nord => Pal {
            bg: c(46, 52, 64),
            sidebar: c(41, 46, 57),
            toolbar: c(46, 52, 64),
            home: c(40, 45, 56),
            hover: c(59, 66, 82),
            input: c(36, 41, 51),
            text: c(236, 239, 244),
            muted: c(144, 155, 175),
            border: c(67, 76, 94),
            add_bg: c(42, 62, 58),
            del_bg: c(64, 46, 54),
            add_gut: c(52, 78, 70),
            del_gut: c(80, 54, 62),
            hunk_bg: c(52, 60, 76),
        },
        Theme::Light => Pal {
            bg: c(255, 255, 255),
            sidebar: c(246, 248, 250),
            toolbar: c(246, 248, 250),
            home: c(250, 250, 252),
            hover: c(232, 235, 239),
            input: Color32::WHITE,
            text: c(36, 41, 46),
            muted: c(87, 96, 106),
            border: c(208, 215, 222),
            add_bg: c(230, 255, 236),
            del_bg: c(255, 235, 233),
            add_gut: c(204, 240, 214),
            del_gut: c(255, 215, 213),
            hunk_bg: c(221, 244, 255),
        },
        Theme::Sand => Pal {
            bg: c(253, 246, 227),
            sidebar: c(246, 238, 216),
            toolbar: c(246, 238, 216),
            home: c(250, 243, 222),
            hover: c(234, 226, 204),
            input: c(255, 252, 240),
            text: c(59, 66, 70),
            muted: c(120, 128, 128),
            border: c(216, 208, 184),
            add_bg: c(225, 243, 206),
            del_bg: c(250, 224, 214),
            add_gut: c(200, 230, 176),
            del_gut: c(244, 204, 190),
            hunk_bg: c(222, 232, 230),
        },
    }
}

pub fn apply_visuals(ctx: &egui::Context, theme: Theme) {
    let p = pal(theme);
    let mut v = if theme.is_dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
    let ac = accent();
    v.panel_fill = p.bg;
    v.window_fill = p.bg;
    v.extreme_bg_color = p.input;
    v.faint_bg_color = p.hover;
    v.window_rounding = egui::Rounding::same(10.0);
    v.window_stroke = egui::Stroke::new(1.0_f32, p.border);
    v.window_shadow = egui::Shadow { offset: egui::vec2(0.0, 6.0), blur: 24.0, spread: 0.0, color: Color32::from_black_alpha(70) };
    v.popup_shadow = v.window_shadow;
    v.menu_rounding = egui::Rounding::same(8.0);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 70);
    v.selection.stroke = egui::Stroke::new(1.0_f32, ac);
    v.hyperlink_color = ac;
    let r = egui::Rounding::same(6.0);
    let w = &mut v.widgets;
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
    w.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
    for x in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        x.rounding = r;
    }
    w.inactive.weak_bg_fill = p.hover;
    w.inactive.bg_fill = p.hover;
    w.inactive.bg_stroke = egui::Stroke::NONE;
    w.inactive.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
    w.hovered.weak_bg_fill = p.border;
    w.hovered.bg_fill = p.border;
    w.hovered.bg_stroke = egui::Stroke::NONE;
    w.hovered.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
    w.active.weak_bg_fill = p.border;
    w.active.bg_stroke = egui::Stroke::new(1.0_f32, ac);
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        use egui::{FontFamily, FontId, TextStyle};
        s.text_styles = [
            (TextStyle::Heading, FontId::new(17.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(9.0, 3.0);
        s.spacing.interact_size.y = 22.0;
        s.spacing.icon_width = 14.0;
        s.spacing.icon_width_inner = 8.0;
        s.spacing.window_margin = egui::Margin::same(14.0);
        s.spacing.menu_margin = egui::Margin::same(6.0);
        s.spacing.scroll.bar_width = 6.0;
        s.spacing.scroll.floating = true;
    });
}
