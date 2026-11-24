#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Settings {
    pub dark: bool,
    pub diff_font: f32,
    pub zoom: f32,
    pub refresh_secs: u64,
    pub editor: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { dark: true, diff_font: 12.0, zoom: 1.0, refresh_secs: 3, editor: "code".into() }
    }
}

pub fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

pub fn settings_path() -> PathBuf {
    home_dir().join(".onaygit_settings")
}

pub fn repos_path() -> PathBuf {
    home_dir().join(".onaygit_repos")
}

impl Settings {
    pub fn load() -> Self {
        let mut s = Settings::default();
        for line in std::fs::read_to_string(settings_path()).unwrap_or_default().lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            match k {
                "dark" => s.dark = v == "true",
                "diff_font" => s.diff_font = v.parse().unwrap_or(12.0),
                "zoom" => s.zoom = v.parse().unwrap_or(1.0),
                "refresh_secs" => s.refresh_secs = v.parse().unwrap_or(3),
                "editor" => s.editor = v.to_string(),
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let text = format!(
            "dark={}\ndiff_font={}\nzoom={}\nrefresh_secs={}\neditor={}\n",
            self.dark, self.diff_font, self.zoom, self.refresh_secs, self.editor
        );
        let _ = std::fs::write(settings_path(), text);
    }
}

// ---------------------------------------------------------------- state

pub fn load_repos() -> Vec<PathBuf> {
    std::fs::read_to_string(repos_path())
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}
