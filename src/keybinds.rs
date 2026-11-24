//! Keyboard shortcuts: default bindings, user overrides saved to a file, and matching.
use crate::prelude::*;

pub struct Cmd {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub key: &'static str,
    pub alias: Option<&'static str>,
    pub desc: &'static str,
    pub fixed: bool,
}

const fn c(id: &'static str, label: &'static str, group: &'static str, key: &'static str) -> Cmd {
    Cmd { id, label, group, key, alias: None, desc: "", fixed: false }
}

pub const GROUPS: [&str; 4] = ["Application", "Spaces", "Tabs", "Repository"];

pub const COMMANDS: [Cmd; 22] = [
    Cmd { alias: Some("ctrl-,"), desc: "Uses the standard macOS Settings menu shortcut.", fixed: true, ..c("settings", "Open Settings", "Application", "cmd-,") },
    c("quick-open", "Find repository or action", "Application", "cmd-p"),
    c("open", "Open repository", "Application", "cmd-o"),
    c("refresh", "Fetch repository", "Application", "cmd-r"),
    c("previous-space", "Previous space", "Spaces", "ctrl-cmd-left"),
    c("next-space", "Next space", "Spaces", "ctrl-cmd-right"),
    c("new-tab", "New tab", "Tabs", "cmd-t"),
    c("close-tab", "Close tab", "Tabs", "cmd-w"),
    c("previous-tab", "Previous tab", "Tabs", "cmd-left"),
    c("next-tab", "Next tab", "Tabs", "cmd-right"),
    c("tab-1", "Switch to tab 1", "Tabs", "cmd-1"),
    c("tab-2", "Switch to tab 2", "Tabs", "cmd-2"),
    c("tab-3", "Switch to tab 3", "Tabs", "cmd-3"),
    c("tab-4", "Switch to tab 4", "Tabs", "cmd-4"),
    c("tab-5", "Switch to tab 5", "Tabs", "cmd-5"),
    c("tab-6", "Switch to tab 6", "Tabs", "cmd-6"),
    c("tab-7", "Switch to tab 7", "Tabs", "cmd-7"),
    c("tab-8", "Switch to tab 8", "Tabs", "cmd-8"),
    c("tab-9", "Switch to tab 9", "Tabs", "cmd-9"),
    Cmd { alias: Some("ctrl-f"), ..c("find", "Find / filter", "Repository", "cmd-f") },
    Cmd { desc: "While the commit message is focused.", ..c("commit", "Commit checked changes", "Repository", "cmd-enter") },
    Cmd { desc: "While the diff is focused.", ..c("expand-diff", "Expand unchanged diff lines", "Repository", "e") },
];

fn path() -> PathBuf {
    home_dir().join(".onaygit_keybinds")
}

/// Saved overrides, one `id=binding` per line.
pub fn load() -> HashMap<String, String> {
    std::fs::read_to_string(path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once('=').map(|(a, b)| (a.to_string(), b.to_string())))
        .filter(|(id, _)| COMMANDS.iter().any(|c| c.id == id && !c.fixed))
        .collect()
}

pub fn save(ov: &HashMap<String, String>) {
    if ov.is_empty() {
        let _ = std::fs::remove_file(path());
        return;
    }
    let mut lines: Vec<String> = ov.iter().map(|(a, b)| format!("{a}={b}")).collect();
    lines.sort();
    let _ = std::fs::write(path(), lines.join("\n"));
}

pub fn bindings(cmd: &Cmd, ov: &HashMap<String, String>) -> Vec<String> {
    match ov.get(cmd.id) {
        Some(b) => vec![b.clone()],
        None => std::iter::once(cmd.key).chain(cmd.alias).map(String::from).collect(),
    }
}

/// Text on a shortcut chip, for example `⌃⌘←`.
pub fn format(binding: &str) -> String {
    let (mods, key) = if binding == "-" {
        ("", "-")
    } else if let Some(m) = binding.strip_suffix("--") {
        (m, "-")
    } else {
        match binding.rsplit_once('-') {
            Some((m, k)) if !k.is_empty() => (m, k),
            _ => ("", binding),
        }
    };
    let mut out = String::new();
    for (name, sym) in [("ctrl", '⌃'), ("alt", '⌥'), ("shift", '⇧'), ("cmd", '⌘')] {
        if mods.split('-').any(|m| m == name) {
            out.push(sym);
        }
    }
    out.push_str(&match key {
        "enter" => "↩".into(),
        "left" => "←".into(),
        "right" => "→".into(),
        "up" => "↑".into(),
        "down" => "↓".into(),
        "space" => "Space".into(),
        "backspace" => "⌫".into(),
        "delete" => "⌦".into(),
        k => k.to_uppercase(),
    });
    out
}

pub fn label(cmd: &Cmd, ov: &HashMap<String, String>) -> String {
    bindings(cmd, ov).iter().map(|b| format(b)).collect::<Vec<_>>().join(" / ")
}

fn key_name(key: egui::Key) -> Option<String> {
    use egui::Key::*;
    Some(match key {
        ArrowLeft => "left".into(),
        ArrowRight => "right".into(),
        ArrowUp => "up".into(),
        ArrowDown => "down".into(),
        Enter => "enter".into(),
        Space => "space".into(),
        Backspace => "backspace".into(),
        Delete => "delete".into(),
        Comma => ",".into(),
        Period => ".".into(),
        Semicolon => ";".into(),
        Slash => "/".into(),
        Minus => "-".into(),
        Equals => "=".into(),
        k => {
            let n = k.name();
            if n.len() == 1 {
                n.to_lowercase()
            } else if let Some(d) = n.strip_prefix("Num") {
                d.to_string()
            } else {
                return None;
            }
        }
    })
}

fn mods_of(m: &egui::Modifiers) -> String {
    let mut s = String::new();
    if m.ctrl {
        s.push_str("ctrl-");
    }
    if m.alt {
        s.push_str("alt-");
    }
    if m.shift {
        s.push_str("shift-");
    }
    if m.command {
        s.push_str("cmd-");
    }
    s
}

/// True when the binding was pressed this frame.
pub fn pressed(i: &egui::InputState, binding: &str) -> bool {
    i.events.iter().any(|e| match e {
        egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. } => {
            key_name(*key).is_some_and(|k| format!("{}{k}", mods_of(modifiers)) == binding)
        }
        _ => false,
    })
}

pub fn hit(i: &egui::InputState, id: &str, ov: &HashMap<String, String>) -> bool {
    COMMANDS.iter().find(|c| c.id == id).is_some_and(|c| bindings(c, ov).iter().any(|b| pressed(i, b)))
}

/// The combination pressed this frame, for recording a new shortcut.
pub fn captured(i: &egui::InputState) -> Option<String> {
    i.events.iter().find_map(|e| match e {
        egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. } => {
            key_name(*key).map(|k| format!("{}{k}", mods_of(modifiers)))
        }
        _ => None,
    })
}

/// Shortcuts the system keeps for itself.
pub fn reserved(binding: &str) -> bool {
    matches!(
        binding,
        "cmd-a" | "cmd-c" | "cmd-x" | "cmd-v" | "cmd-z" | "shift-cmd-z" | "cmd-q" | "cmd-h" | "alt-cmd-h" | "cmd-m" | "cmd-space"
            | "ctrl-a" | "ctrl-c" | "ctrl-x" | "ctrl-v" | "ctrl-z" | "cmd-up" | "cmd-down" | "cmd-backspace" | "cmd-delete"
    )
}
