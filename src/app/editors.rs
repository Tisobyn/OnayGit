//! External editor: installed applications and per-file-type rules.
#[allow(unused_imports)]
use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct EditorRule {
    /// File types or names, comma separated: `.ts, .tsx, Dockerfile`.
    pub patterns: String,
    pub editor: String,
}

fn rules_path() -> PathBuf {
    home_dir().join(".onaygit_editor_rules")
}

pub fn load_rules() -> Vec<EditorRule> {
    std::fs::read_to_string(rules_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (p, e) = l.split_once('\t')?;
            Some(EditorRule { patterns: p.to_string(), editor: e.to_string() })
        })
        .collect()
}

/// Editors that are installed, as (label, command, icon).
pub fn installed_editors() -> Vec<(String, String, &'static str)> {
    let mut out = vec![("System default".to_string(), "open".to_string(), "pencil")];
    let known = [
        ("VS Code", "Visual Studio Code"),
        ("Cursor", "Cursor"),
        ("Zed", "Zed"),
        ("Sublime Text", "Sublime Text"),
        ("Xcode", "Xcode"),
        ("Nova", "Nova"),
        ("BBEdit", "BBEdit"),
        ("WebStorm", "WebStorm"),
        ("IntelliJ IDEA", "IntelliJ IDEA"),
        ("TextMate", "TextMate"),
    ];
    for (label, app) in known {
        let path = format!("/Applications/{app}.app");
        if Path::new(&path).exists() {
            out.push((label.to_string(), format!("open -a {}", git::quote(&path)), "terminal"));
        }
    }
    out
}

/// Display name of an editor command.
pub fn editor_label(cmd: &str) -> String {
    if cmd == "open" {
        return "System default".into();
    }
    if let Some((label, _, _)) = installed_editors().into_iter().find(|(_, c, _)| c == cmd) {
        return label;
    }
    cmd.trim_start_matches("open -a ").trim_matches('\'').rsplit('/').next().unwrap_or(cmd).trim_end_matches(".app").to_string()
}

impl EditorRule {
    fn matches(&self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.patterns.split([',', ' ']).map(|p| p.trim().to_lowercase()).filter(|p| !p.is_empty()).any(|p| {
            if p.starts_with('.') {
                name.ends_with(&p)
            } else {
                name == p
            }
        })
    }
}

impl App {
    pub fn save_rules(&self) {
        let text: Vec<String> = self.editor_rules.iter().map(|r| format!("{}\t{}", r.patterns.replace('\t', " "), r.editor)).collect();
        let _ = std::fs::write(rules_path(), text.join("\n"));
    }

    /// The editor command for a file: the first matching rule, else the default.
    pub fn editor_for(&self, path: &Path) -> String {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.editor_rules.iter().find(|r| r.matches(&name)).map(|r| r.editor.clone()).unwrap_or_else(|| self.settings.editor.clone())
    }
}
