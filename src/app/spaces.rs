//! Spaces: named groups of repositories shown on the home screen.
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Space {
    pub name: String,
    pub repos: Vec<PathBuf>,
}

fn spaces_path() -> PathBuf {
    home_dir().join(".onaygit_spaces")
}

/// One space per line: name, then repository paths, separated by tabs.
pub fn load_spaces() -> Vec<Space> {
    std::fs::read_to_string(spaces_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            let name = p.next()?.to_string();
            (!name.is_empty()).then(|| Space { name, repos: p.map(PathBuf::from).collect() })
        })
        .collect()
}

impl App {
    fn save_spaces(&self) {
        let text: Vec<String> = self
            .spaces
            .iter()
            .map(|s| {
                let mut parts = vec![s.name.replace('\t', " ")];
                parts.extend(s.repos.iter().map(|r| r.to_string_lossy().into_owned()));
                parts.join("\t")
            })
            .collect();
        let _ = std::fs::write(spaces_path(), text.join("\n"));
    }

    pub fn add_space(&mut self) {
        let name = self.space_name.trim().to_string();
        if name.is_empty() || self.spaces.iter().any(|s| s.name == name) {
            return;
        }
        self.spaces.push(Space { name, repos: vec![] });
        self.space_name.clear();
        self.selected_space = Some(self.spaces.len() - 1);
        self.save_spaces();
    }

    pub fn rename_space(&mut self, i: usize, name: &str) {
        let name = name.trim();
        if name.is_empty() || self.spaces.iter().enumerate().any(|(k, s)| k != i && s.name == name) {
            return;
        }
        if let Some(s) = self.spaces.get_mut(i) {
            s.name = name.to_string();
            self.save_spaces();
        }
    }

    pub fn remove_space(&mut self, i: usize) {
        if i < self.spaces.len() {
            self.spaces.remove(i);
            self.selected_space = None;
            self.save_spaces();
        }
    }

    pub fn toggle_in_space(&mut self, i: usize, repo: &Path) {
        if let Some(s) = self.spaces.get_mut(i) {
            match s.repos.iter().position(|r| r == repo) {
                Some(k) => {
                    s.repos.remove(k);
                }
                None => s.repos.push(repo.to_path_buf()),
            }
            self.save_spaces();
        }
    }
}
