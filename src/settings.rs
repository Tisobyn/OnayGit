#[allow(unused_imports)]
use crate::git::{AiConfig, AiProvider};
use crate::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Settings {
    pub theme: Theme,
    /// Follow the system light/dark setting.
    pub auto_theme: bool,
    pub scrollbars: bool,
    pub smooth: bool,
    /// Stage every changed file when committing.
    pub auto_include: bool,
    pub accent: usize,
    pub tips: bool,
    pub onboarded: bool,
    pub diff_font: f32,
    pub zoom: f32,
    pub refresh_secs: u64,
    pub editor: String,
    pub ai: AiConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: Theme::Dark,
            auto_theme: false,
            scrollbars: true,
            smooth: true,
            auto_include: false,
            accent: 0,
            tips: true,
            onboarded: false,
            diff_font: 12.0,
            zoom: 1.0,
            refresh_secs: 3,
            editor: "code".into(),
            ai: AiConfig::default(),
        }
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
                "dark" => s.theme = if v == "true" { Theme::Dark } else { Theme::Light },
                "scrollbars" => s.scrollbars = v == "true",
                "smooth" => s.smooth = v == "true",
                "auto_include" => s.auto_include = v == "true",
                "auto_theme" => s.auto_theme = v == "true",
                "theme" => s.theme = Theme::from_key(v).unwrap_or(Theme::Dark),
                "accent" => s.accent = v.parse().unwrap_or(0),
                "tips" => s.tips = v != "false",
                "onboarded" => s.onboarded = v == "true",
                "diff_font" => s.diff_font = v.parse().unwrap_or(12.0),
                "zoom" => s.zoom = v.parse().unwrap_or(1.0),
                "refresh_secs" => s.refresh_secs = v.parse().unwrap_or(3),
                "editor" => s.editor = v.to_string(),
                "ai_provider" => s.ai.provider = AiProvider::from_key(v).unwrap_or(AiProvider::ClaudeCode),
                "ai_command" => s.ai.command = v.to_string(),
                "ai_model" => s.ai.model = v.to_string(),
                "ai_base_url" => s.ai.base_url = v.to_string(),
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let text = format!(
            "theme={}\nauto_theme={}\nscrollbars={}\nsmooth={}\nauto_include={}\naccent={}\ntips={}\nonboarded={}\ndiff_font={}\nzoom={}\nrefresh_secs={}\neditor={}\nai_provider={}\nai_command={}\nai_model={}\nai_base_url={}\n",
            self.theme.key(),
            self.auto_theme,
            self.scrollbars,
            self.smooth,
            self.auto_include,
            self.accent,
            self.tips,
            self.onboarded,
            self.diff_font, self.zoom, self.refresh_secs,
            self.editor,
            self.ai.provider.key(),
            self.ai.command.replace('\n', " "),
            self.ai.model,
            self.ai.base_url
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
        .filter(|p| p.exists() || git::is_remote(p))
        .collect()
}
