//! Image files in a diff: save the old version to a temporary file so it can be shown.
use super::*;
use std::path::PathBuf;

pub fn is_image(path: &str) -> bool {
    let ext = Path::new(path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg")
}

/// Write `rev:path` (for example `HEAD:logo.png`) to a temp file and return its path.
pub fn blob_to_temp(repo: &Path, rev: &str, path: &str) -> Option<PathBuf> {
    let spec = format!("{rev}:{path}");
    let out = git_command(repo, &["show", &spec], &[]).output().ok()?;
    if !out.status.success() || out.stdout.is_empty() {
        return None;
    }
    let dir = std::env::temp_dir().join("onaygit-images");
    std::fs::create_dir_all(&dir).ok()?;
    let ext = Path::new(path).extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&(repo, &spec, out.stdout.len()), &mut h);
    let file = dir.join(format!("{:x}.{ext}", std::hash::Hasher::finish(&h)));
    std::fs::write(&file, &out.stdout).ok()?;
    Some(file)
}
