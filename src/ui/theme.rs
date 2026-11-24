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
            add_bg: c(25, 58, 38),
            del_bg: c(70, 30, 34),
            add_gut: c(34, 84, 52),
            del_gut: c(100, 40, 46),
            hunk_bg: c(40, 48, 58),
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
            add_bg: c(15, 45, 28),
            del_bg: c(60, 22, 26),
            add_gut: c(24, 70, 42),
            del_gut: c(88, 32, 38),
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
            add_bg: c(38, 68, 60),
            del_bg: c(74, 44, 52),
            add_gut: c(50, 88, 76),
            del_gut: c(98, 56, 66),
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
    v.panel_fill = p.bg;
    v.window_fill = p.bg;
    v.extreme_bg_color = p.input;
    v.selection.bg_fill = accent();
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
    ctx.set_visuals(v);
}
