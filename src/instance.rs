//! One window: a second launch hands its folder to the running app and exits.
use crate::settings::home_dir;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

fn sock_path() -> PathBuf {
    home_dir().join(".onaygit.sock")
}

/// Pass this launch to a running app. Returns true if one took it.
pub fn hand_over() -> bool {
    if std::env::var("ONAYGIT_SCREEN").is_ok() {
        return false;
    }
    let Ok(mut s) = UnixStream::connect(sock_path()) else { return false };
    let arg = std::env::args()
        .nth(1)
        .and_then(|a| std::fs::canonicalize(a).ok())
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    s.write_all(format!("{arg}\n").as_bytes()).is_ok()
}

/// Listen for later launches. Each line is a folder, or empty to just bring the window forward.
pub fn listen(tx: Sender<String>, ctx: eframe::egui::Context) {
    let path = sock_path();
    // Nobody answered in `hand_over`, so an old socket file is stale.
    let _ = std::fs::remove_file(&path);
    let Ok(listener) = UnixListener::bind(&path) else { return };
    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let mut line = String::new();
            if BufReader::new(conn).read_line(&mut line).is_ok() {
                let _ = tx.send(line.trim().to_string());
                ctx.request_repaint();
            }
        }
    });
}
