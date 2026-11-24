//! Tabs (several repositories open in one window) and per-repository looks.
#[allow(unused_imports)]
use crate::prelude::*;

/// Icons a repository can use.
pub const REPO_ICONS: [&str; 12] = [
    "folder", "branch", "layers", "terminal", "sparkle", "tree", "archive", "home", "network", "palette", "history",
    "shield",
];

#[derive(Clone, PartialEq)]
pub struct RepoStyle {
    /// Index into ACCENTS.
    pub color: usize,
    pub icon: String,
}

fn tabs_path() -> PathBuf {
    home_dir().join(".onaygit_tabs")
}

fn styles_path() -> PathBuf {
    home_dir().join(".onaygit_styles")
}

pub fn load_tabs() -> Vec<PathBuf> {
    std::fs::read_to_string(tabs_path())
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}

pub fn load_styles() -> HashMap<PathBuf, RepoStyle> {
    let mut map = HashMap::new();
    for line in std::fs::read_to_string(styles_path()).unwrap_or_default().lines() {
        let mut p = line.split('\t');
        if let (Some(path), Some(color), Some(icon)) = (p.next(), p.next(), p.next()) {
            map.insert(
                PathBuf::from(path),
                RepoStyle { color: color.parse().unwrap_or(0), icon: icon.to_string() },
            );
        }
    }
    map
}

impl App {
    fn save_tabs(&self) {
        let text: Vec<String> = self.tabs.iter().map(|p| p.display().to_string()).collect();
        let _ = std::fs::write(tabs_path(), text.join("\n"));
    }

    fn save_styles(&self) {
        let text: Vec<String> = self
            .repo_styles
            .iter()
            .map(|(p, s)| format!("{}\t{}\t{}", p.display(), s.color, s.icon))
            .collect();
        let _ = std::fs::write(styles_path(), text.join("\n"));
    }

    /// Make sure the open repository has a tab.
    pub fn ensure_tab(&mut self) {
        if let Some(r) = self.repo() {
            if !self.tabs.contains(&r) {
                self.tabs.push(r);
                self.save_tabs();
            }
        }
    }

    /// Switch to the repository of tab `i`.
    pub fn activate_tab(&mut self, i: usize) {
        if let Some(path) = self.tabs.get(i).cloned() {
            self.current = self.repos.iter().position(|p| *p == path);
            self.selected_file = None;
            self.selected_stash = None;
            self.selected_commit = None;
            self.commit_diff.clear();
            self.compare = None;
            self.screen = Screen::Repo;
            self.refresh();
        }
    }

    pub fn close_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        let closed = self.tabs.remove(i);
        self.save_tabs();
        if self.repo().as_ref() == Some(&closed) {
            if self.tabs.is_empty() {
                self.current = None;
                self.screen = Screen::Home;
                self.refresh();
            } else {
                self.activate_tab(i.min(self.tabs.len() - 1));
            }
        }
    }

    /// Color and icon of a repository (defaults come from its name).
    pub fn style_of(&self, path: &Path) -> RepoStyle {
        self.repo_styles.get(path).cloned().unwrap_or_else(|| {
            let sum: usize = path.to_string_lossy().bytes().map(|b| b as usize).sum();
            RepoStyle { color: sum % ACCENTS.len(), icon: "folder".to_string() }
        })
    }

    pub fn set_style(&mut self, path: &Path, style: RepoStyle) {
        self.repo_styles.insert(path.to_path_buf(), style);
        self.save_styles();
    }
}
