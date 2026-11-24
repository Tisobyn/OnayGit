//! Phosphor Light icons taken from 1git (MIT, see assets/icons/LICENSE-Phosphor.txt).
#[allow(unused_imports)]
use crate::prelude::*;

fn svg(name: &str) -> Option<&'static [u8]> {
    match name {
        "sp_layers" => Some(include_bytes!("../../assets/icons/sp_layers.svg")),
        "sp_work" => Some(include_bytes!("../../assets/icons/sp_work.svg")),
        "sp_home" => Some(include_bytes!("../../assets/icons/sp_home.svg")),
        "sp_code" => Some(include_bytes!("../../assets/icons/sp_code.svg")),
        "sp_folder" => Some(include_bytes!("../../assets/icons/sp_folder.svg")),
        "sp_terminal" => Some(include_bytes!("../../assets/icons/sp_terminal.svg")),
        "sp_server" => Some(include_bytes!("../../assets/icons/sp_server.svg")),
        "sp_globe" => Some(include_bytes!("../../assets/icons/sp_globe.svg")),
        "sp_cloud" => Some(include_bytes!("../../assets/icons/sp_cloud.svg")),
        "sp_bolt" => Some(include_bytes!("../../assets/icons/sp_bolt.svg")),
        "sp_rocket" => Some(include_bytes!("../../assets/icons/sp_rocket.svg")),
        "sp_flask" => Some(include_bytes!("../../assets/icons/sp_flask.svg")),
        "sp_palette" => Some(include_bytes!("../../assets/icons/sp_palette.svg")),
        "sp_book" => Some(include_bytes!("../../assets/icons/sp_book.svg")),
        "sp_bookmark" => Some(include_bytes!("../../assets/icons/sp_bookmark.svg")),
        "sp_star" => Some(include_bytes!("../../assets/icons/sp_star.svg")),
        "sp_heart" => Some(include_bytes!("../../assets/icons/sp_heart.svg")),
        "sp_coffee" => Some(include_bytes!("../../assets/icons/sp_coffee.svg")),
        "sp_music" => Some(include_bytes!("../../assets/icons/sp_music.svg")),
        "sp_headphones" => Some(include_bytes!("../../assets/icons/sp_headphones.svg")),
        "sp_compass" => Some(include_bytes!("../../assets/icons/sp_compass.svg")),
        "sp_leaf" => Some(include_bytes!("../../assets/icons/sp_leaf.svg")),
        "sp_mountain" => Some(include_bytes!("../../assets/icons/sp_mountain.svg")),
        "sp_shield" => Some(include_bytes!("../../assets/icons/sp_shield.svg")),
        "add" => Some(include_bytes!("../../assets/icons/add.svg")),
        "archive" => Some(include_bytes!("../../assets/icons/archive.svg")),
        "arrowDown" => Some(include_bytes!("../../assets/icons/arrowDown.svg")),
        "arrowUp" => Some(include_bytes!("../../assets/icons/arrowUp.svg")),
        "arrowLeft" => Some(include_bytes!("../../assets/icons/arrowLeft.svg")),
        "branch" => Some(include_bytes!("../../assets/icons/branch.svg")),
        "browser" => Some(include_bytes!("../../assets/icons/browser.svg")),
        "caretDown" => Some(include_bytes!("../../assets/icons/caretDown.svg")),
        "caretRight" => Some(include_bytes!("../../assets/icons/caretRight.svg")),
        "chatgpt" => Some(include_bytes!("../../assets/icons/chatgpt.svg")),
        "check" => Some(include_bytes!("../../assets/icons/check.svg")),
        "close" => Some(include_bytes!("../../assets/icons/close.svg")),
        "codex" => Some(include_bytes!("../../assets/icons/codex.svg")),
        "contrast" => Some(include_bytes!("../../assets/icons/contrast.svg")),
        "copy" => Some(include_bytes!("../../assets/icons/copy.svg")),
        "desktop" => Some(include_bytes!("../../assets/icons/desktop.svg")),
        "grid" => Some(include_bytes!("../../assets/icons/grid.svg")),
        "direction" => Some(include_bytes!("../../assets/icons/direction.svg")),
        "folder" => Some(include_bytes!("../../assets/icons/folder.svg")),
        "github" => Some(include_bytes!("../../assets/icons/github.svg")),
        "gitlab" => Some(include_bytes!("../../assets/icons/gitlab.svg")),
        "help" => Some(include_bytes!("../../assets/icons/help.svg")),
        "history" => Some(include_bytes!("../../assets/icons/history.svg")),
        "home" => Some(include_bytes!("../../assets/icons/home.svg")),
        "idCard" => Some(include_bytes!("../../assets/icons/idCard.svg")),
        "key" => Some(include_bytes!("../../assets/icons/key.svg")),
        "keyboard" => Some(include_bytes!("../../assets/icons/keyboard.svg")),
        "layers" => Some(include_bytes!("../../assets/icons/layers.svg")),
        "list" => Some(include_bytes!("../../assets/icons/list.svg")),
        "network" => Some(include_bytes!("../../assets/icons/network.svg")),
        "opencode" => Some(include_bytes!("../../assets/icons/opencode.svg")),
        "palette" => Some(include_bytes!("../../assets/icons/palette.svg")),
        "pencil" => Some(include_bytes!("../../assets/icons/pencil.svg")),
        "publish" => Some(include_bytes!("../../assets/icons/publish.svg")),
        "pull" => Some(include_bytes!("../../assets/icons/pull.svg")),
        "push" => Some(include_bytes!("../../assets/icons/push.svg")),
        "reconnect" => Some(include_bytes!("../../assets/icons/reconnect.svg")),
        "remove" => Some(include_bytes!("../../assets/icons/remove.svg")),
        "settings" => Some(include_bytes!("../../assets/icons/settings.svg")),
        "search" => Some(include_bytes!("../../assets/icons/search.svg")),
        "shield" => Some(include_bytes!("../../assets/icons/shield.svg")),
        "sparkle" => Some(include_bytes!("../../assets/icons/sparkle.svg")),
        "stop" => Some(include_bytes!("../../assets/icons/stop.svg")),
        "swap" => Some(include_bytes!("../../assets/icons/swap.svg")),
        "sync" => Some(include_bytes!("../../assets/icons/sync.svg")),
        "tabs" => Some(include_bytes!("../../assets/icons/tabs.svg")),
        "terminal" => Some(include_bytes!("../../assets/icons/terminal.svg")),
        "tree" => Some(include_bytes!("../../assets/icons/tree.svg")),
        "userCircle" => Some(include_bytes!("../../assets/icons/userCircle.svg")),
        _ => None,
    }
}

/// Paint icon `name` into `rect`, tinted with `color`.
pub fn paint_icon(ui: &egui::Ui, name: &str, rect: egui::Rect, color: Color32) {
    let Some(bytes) = svg(name) else { return };
    egui::Image::from_bytes(format!("bytes://icons/{name}.svg"), bytes)
        .tint(color)
        .paint_at(ui, rect);
}

/// A square icon that takes space in the layout.
pub fn icon(ui: &mut egui::Ui, name: &str, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    paint_icon(ui, name, rect, color);
}
