//! Remotes.
#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

pub fn remotes(repo: &Path) -> Vec<Remote> {
    run(repo, &["remote", "-v"])
        .unwrap_or_default()
        .lines()
        .filter(|l| l.ends_with("(fetch)"))
        .filter_map(|l| {
            let (name, rest) = l.split_once('\t')?;
            let url = rest.trim_end_matches("(fetch)").trim();
            Some(Remote { name: name.to_string(), url: url.to_string() })
        })
        .collect()
}

/// The remote used for push and pull: origin, else the first one.
pub fn primary_remote(list: &[Remote]) -> String {
    list.iter()
        .find(|r| r.name == "origin")
        .or(list.first())
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "origin".to_string())
}

pub fn add_remote(repo: &Path, name: &str, url: &str) -> Result<String, String> {
    run(repo, &["remote", "add", name, url])
}

pub fn remove_remote(repo: &Path, name: &str) -> Result<String, String> {
    run(repo, &["remote", "remove", name])
}

pub fn rename_remote(repo: &Path, old: &str, new: &str) -> Result<String, String> {
    run(repo, &["remote", "rename", old, new])
}

pub fn set_remote_url(repo: &Path, name: &str, url: &str) -> Result<String, String> {
    run(repo, &["remote", "set-url", name, url])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_edit_rename_remove() {
        let d = std::env::temp_dir().join(format!("onaygit-remote-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        run(&d, &["init", "-q"]).unwrap();
        assert!(remotes(&d).is_empty());
        assert_eq!(primary_remote(&remotes(&d)), "origin");

        add_remote(&d, "up", "https://example.com/a.git").unwrap();
        add_remote(&d, "origin", "https://example.com/b.git").unwrap();
        assert_eq!(remotes(&d).len(), 2);
        assert_eq!(primary_remote(&remotes(&d)), "origin");

        set_remote_url(&d, "up", "https://example.com/c.git").unwrap();
        rename_remote(&d, "up", "upstream").unwrap();
        let list = remotes(&d);
        assert!(list.contains(&Remote { name: "upstream".into(), url: "https://example.com/c.git".into() }));

        remove_remote(&d, "origin").unwrap();
        assert_eq!(primary_remote(&remotes(&d)), "upstream");
        let _ = std::fs::remove_dir_all(&d);
    }
}
