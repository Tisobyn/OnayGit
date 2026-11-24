//! Tabs (several repositories open in one window) and per-repository looks.
#[allow(unused_imports)]
use crate::prelude::*;

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

/// Open tabs per space. Lines are `space<TAB>path`; a line with no tab belongs to all repositories.
pub fn load_tabs() -> HashMap<String, Vec<PathBuf>> {
    let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for l in std::fs::read_to_string(tabs_path()).unwrap_or_default().lines().filter(|l| !l.trim().is_empty()) {
        let (space, path) = l.split_once('\t').unwrap_or(("", l));
        let path = PathBuf::from(path);
        if path.exists() || git::is_remote(&path) {
            map.entry(space.to_string()).or_default().push(path);
        }
    }
    map
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
        let mut text: Vec<String> = self.tabs.iter().map(|p| format!("{}\t{}", self.tabs_space, p.display())).collect();
        for (space, list) in &self.space_tabs {
            text.extend(list.iter().map(|p| format!("{space}\t{}", p.display())));
        }
        let _ = std::fs::write(tabs_path(), text.join("\n"));
    }

    /// Show another space. Each space keeps its own tabs.
    pub fn set_space(&mut self, space: Option<usize>) {
        let key = space.and_then(|i| self.spaces.get(i)).map(|s| s.name.clone()).unwrap_or_default();
        if key != self.tabs_space {
            let old = std::mem::take(&mut self.tabs);
            self.space_tabs.insert(std::mem::replace(&mut self.tabs_space, key.clone()), old);
            self.tabs = self.space_tabs.remove(&key).unwrap_or_default();
            self.save_tabs();
            let cur = self.repo();
            if cur.as_ref().is_none_or(|r| !self.tabs.contains(r)) {
                self.current = None;
                self.screen = Screen::Home;
            }
        }
        self.selected_space = space;
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
}
