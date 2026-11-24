use super::*;

pub fn is_repo(path: &Path) -> bool {
    run(path, &["rev-parse", "--is-inside-work-tree"])
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}

pub fn init(dir: &Path, branch: &str) -> Result<String, String> {
    let branch = if branch.trim().is_empty() { "main" } else { branch.trim() };
    run(dir, &["init", "-b", branch]).or_else(|_| run(dir, &["init"]))
}

/// Find git repositories below `root` (and `root` itself), up to `depth` levels.
pub fn scan(root: &Path, depth: u32) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    scan_into(root, depth, &mut found);
    found
}

pub fn scan_into(dir: &Path, depth: u32, found: &mut Vec<std::path::PathBuf>) {
    if dir.join(".git").exists() {
        found.push(dir.to_path_buf());
        return;
    }
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if !p.is_dir() || name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        scan_into(&p, depth - 1, found);
    }
}
