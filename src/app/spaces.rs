//! Spaces: named groups of repositories shown on the home screen.
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Space {
    pub name: String,
    /// Icon preset id, see `SPACE_ICONS`.
    pub icon: String,
    /// Index into `SPACE_COLORS`, or none for the default colour.
    pub color: Option<usize>,
    pub repos: Vec<PathBuf>,
}

pub const SPACE_ICONS: [(&str, &str); 24] = [
    ("layers", "Layers"), ("work", "Work"), ("home", "Home"), ("code", "Code"), ("folder", "Folder"), ("terminal", "Terminal"),
    ("server", "Server"), ("globe", "Globe"), ("cloud", "Cloud"), ("bolt", "Bolt"), ("rocket", "Rocket"), ("flask", "Science"),
    ("palette", "Design"), ("book", "Book"), ("bookmark", "Bookmark"), ("star", "Star"), ("heart", "Heart"), ("coffee", "Coffee"),
    ("music", "Music"), ("headphones", "Headphones"), ("compass", "Compass"), ("leaf", "Leaf"), ("mountain", "Mountain"), ("shield", "Shield"),
];

pub const SPACE_COLORS: [(&str, [u8; 3]); 8] = [
    ("Blue", [0x52, 0x8b, 0xff]), ("Violet", [0xa4, 0x78, 0xef]), ("Rose", [0xe8, 0x79, 0xa0]), ("Orange", [0xe9, 0x98, 0x55]),
    ("Amber", [0xd9, 0xb3, 0x4a]), ("Green", [0x62, 0xae, 0x86]), ("Teal", [0x46, 0xb8, 0xb3]), ("Slate", [0x82, 0x94, 0xb3]),
];

/// Icon file name for a space icon id.
pub fn space_icon_name(id: &str) -> String {
    format!("sp_{id}")
}

fn spaces_path() -> PathBuf {
    home_dir().join(".onaygit_spaces")
}

/// One space per line, tab separated: name, `icon:<id>`, `color:<n>`, then repository paths.
pub fn load_spaces() -> Vec<Space> {
    std::fs::read_to_string(spaces_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            let name = p.next()?.to_string();
            let mut icon = "layers".to_string();
            let mut color = None;
            let mut repos = vec![];
            for f in p {
                if let Some(v) = f.strip_prefix("icon:") {
                    icon = v.to_string();
                } else if let Some(v) = f.strip_prefix("color:") {
                    color = v.parse().ok().filter(|&n| n < SPACE_COLORS.len());
                } else {
                    repos.push(PathBuf::from(f));
                }
            }
            (!name.is_empty()).then(|| Space { name, icon, color, repos })
        })
        .collect()
}

impl App {
    fn save_spaces(&self) {
        let text: Vec<String> = self
            .spaces
            .iter()
            .map(|s| {
                let mut parts = vec![s.name.replace('\t', " "), format!("icon:{}", s.icon)];
                if let Some(c) = s.color {
                    parts.push(format!("color:{c}"));
                }
                parts.extend(s.repos.iter().map(|r| r.to_string_lossy().into_owned()));
                parts.join("\t")
            })
            .collect();
        let _ = std::fs::write(spaces_path(), text.join("\n"));
    }

    /// Go to Settings > Spaces with the new space panel open.
    pub fn start_new_space(&mut self) {
        self.open_settings();
        self.section = Section::Spaces;
        self.space_name.clear();
        self.space_icon = "layers".into();
        self.space_color = None;
        self.space_dialog = Some(None);
    }

    pub fn add_space(&mut self) {
        let name = self.space_name.trim().to_string();
        if name.is_empty() || self.spaces.iter().any(|s| s.name == name) {
            return;
        }
        self.spaces.push(Space { name, icon: self.space_icon.clone(), color: self.space_color, repos: vec![] });
        self.space_name.clear();
        self.set_space(Some(self.spaces.len() - 1));
        self.save_spaces();
    }

    pub fn edit_space(&mut self, i: usize) {
        let (icon, color) = (self.space_icon.clone(), self.space_color);
        let name = self.space_name.clone();
        self.rename_space(i, &name);
        if let Some(s) = self.spaces.get_mut(i) {
            if s.name == name.trim() {
                s.icon = icon;
                s.color = color;
                self.save_spaces();
            }
        }
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
            self.set_space(None);
            let name = self.spaces.remove(i).name;
            self.space_tabs.remove(&name);
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
