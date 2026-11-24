//! Reading and editing the repository `.gitignore`.
#[allow(unused_imports)]
use super::*;

pub fn read_ignore(repo: &Path) -> String {
    read_file(repo, ".gitignore")
}

pub fn write_ignore(repo: &Path, text: &str) -> Result<(), String> {
    write_file(repo, ".gitignore", text)
}

/// Append a pattern to `.gitignore` unless it is already there.
pub fn ignore_pattern(repo: &Path, pattern: &str) -> Result<(), String> {
    let mut text = read_ignore(repo);
    if text.lines().any(|l| l.trim() == pattern) {
        return Ok(());
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(pattern);
    text.push('\n');
    write_ignore(repo, &text)
}
