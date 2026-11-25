mod app;
mod instance;
mod keybinds;
mod prelude;
mod settings;
mod ui;
mod git;

use eframe::egui;

fn load_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../assets/icon-512.png"))
        .ok()?
        .into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}

fn main() -> eframe::Result<()> {
    if instance::hand_over() {
        return Ok(());
    }
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 821.0])
        .with_min_inner_size([800.0, 500.0])
        .with_title("OnayGit")
        .with_fullsize_content_view(true)
        .with_title_shown(false)
        .with_titlebar_shown(false);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "OnayGit",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
