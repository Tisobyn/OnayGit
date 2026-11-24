//! Tags.
use super::*;

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub hash: String,
    pub subject: String,
}

pub fn tags(repo: &Path) -> Vec<Tag> {
    run(
        repo,
        &[
            "tag",
            "--sort=-creatordate",
            "--format=%(refname:short)%1f%(objectname:short)%1f%(contents:subject)",
        ],
    )
    .unwrap_or_default()
    .lines()
    .filter_map(|l| {
        let mut p = l.split('\x1f');
        Some(Tag {
            name: p.next()?.to_string(),
            hash: p.next()?.to_string(),
            subject: p.next().unwrap_or("").to_string(),
        })
    })
    .collect()
}

/// Create a tag at `target` (empty means HEAD). A message makes it an annotated tag.
pub fn create_tag(repo: &Path, name: &str, target: &str, message: &str) -> Result<String, String> {
    let target = if target.is_empty() { "HEAD" } else { target };
    if message.trim().is_empty() {
        run(repo, &["tag", name, target])
    } else {
        run(repo, &["tag", "-a", name, target, "-m", message.trim()])
    }
}

pub fn delete_tag(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["tag", "-d", name])
}
