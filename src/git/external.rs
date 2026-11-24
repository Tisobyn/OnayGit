#[allow(unused_imports)]
use super::*;

/// Open a file in the user's editor command, for example `code` or `zed`.
pub fn open_editor(editor: &str, path: &Path) -> Result<(), String> {
    if let Some((host, remote)) = split_remote(path) {
        // VS Code style editors can open files over SSH.
        let cmd = format!("{editor} --remote ssh-remote+{host} {}", quote(&remote));
        return Command::new("sh").arg("-c").arg(cmd).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    let quoted = format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
    Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} {quoted}"))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn reveal(path: &Path) {
    let _ = Command::new("open").arg("-R").arg(path).spawn();
}
