//! Profile pictures for commit authors. Found through the GitHub CLI, noreply
//! addresses or Gravatar, and cached as small PNG files.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

const SIZE: u32 = 64;
const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);
const RETRY: Duration = Duration::from_secs(24 * 3600);

fn dir() -> PathBuf {
    crate::settings::home_dir().join(".onaygit_avatars")
}

fn key(email: &str) -> String {
    email.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '-' { c } else { '_' }).collect()
}

/// Where the picture of `email` is stored once fetched.
pub fn avatar_path(email: &str) -> PathBuf {
    dir().join(format!("{}.png", key(email)))
}

fn younger(p: &Path, limit: Duration) -> bool {
    std::fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| SystemTime::now().duration_since(t).ok()).is_some_and(|d| d < limit)
}

/// Picture URL when the email itself tells who the user is.
pub fn url_from_email(email: &str) -> Option<String> {
    let e = email.to_lowercase();
    if let Some(user) = e.strip_suffix("@users.noreply.github.com") {
        return Some(match user.split_once('+') {
            Some((id, _)) if id.chars().all(|c| c.is_ascii_digit()) => format!("https://avatars.githubusercontent.com/u/{id}?s={SIZE}"),
            Some((_, login)) => format!("https://github.com/{login}.png?size={SIZE}"),
            None => format!("https://github.com/{user}.png?size={SIZE}"),
        });
    }
    // Claude's commit address belongs to the `claude` account on GitHub.
    if e == "noreply@anthropic.com" {
        return Some(format!("https://github.com/claude.png?size={SIZE}"));
    }
    None
}

fn gh(args: &[&str]) -> Option<String> {
    for bin in ["gh", "/opt/homebrew/bin/gh", "/usr/local/bin/gh"] {
        if let Ok(o) = Command::new(bin).args(args).stdin(Stdio::null()).output() {
            return o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty());
        }
    }
    None
}

fn sha256_hex(text: &str) -> Option<String> {
    let mut c = Command::new("shasum").args(["-a", "256"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?;
    c.stdin.take()?.write_all(text.as_bytes()).ok()?;
    let out = c.wait_with_output().ok()?;
    String::from_utf8_lossy(&out.stdout).split_whitespace().next().map(String::from)
}

fn download(url: &str, to: &Path) -> bool {
    let tmp = to.with_extension("dl");
    let ok = Command::new("curl")
        .args(["-sfL", "-m", "15", "-A", "OnayGit", "-o"])
        .arg(&tmp)
        .arg(url)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    let saved = std::fs::read(&tmp)
        .ok()
        .and_then(|b| image::load_from_memory(&b).ok())
        .map(|img| img.resize_to_fill(SIZE, SIZE, image::imageops::FilterType::Triangle))
        .map(|img| img.save_with_format(to, image::ImageFormat::Png).is_ok())
        .unwrap_or(false);
    let _ = std::fs::remove_file(&tmp);
    saved
}

/// Fetch the picture for `email`. `commit` is `(owner/repo, sha)` of a GitHub commit this
/// person wrote, which lets the GitHub CLI name the exact account.
pub fn fetch_avatar(email: &str, commit: Option<(&str, &str)>) -> Result<PathBuf, String> {
    let to = avatar_path(email);
    let none = to.with_extension("none");
    if to.exists() && younger(&to, KEEP) {
        return Ok(to);
    }
    if none.exists() && younger(&none, RETRY) {
        return Err("no picture".into());
    }
    let _ = std::fs::create_dir_all(dir());
    let mut urls: Vec<String> = vec![];
    urls.extend(url_from_email(email));
    if urls.is_empty() {
        if let Some((slug, sha)) = commit {
            if let Some(u) = gh(&["api", &format!("repos/{slug}/commits/{sha}"), "--jq", ".author.avatar_url // empty"]) {
                urls.push(format!("{u}&s={SIZE}"));
            }
        }
        if !email.is_empty() {
            if let Some(u) = gh(&["api", "-X", "GET", "search/users", "-f", &format!("q={email} in:email"), "--jq", ".items[0].avatar_url // empty"]) {
                urls.push(format!("{u}&s={SIZE}"));
            }
            if let Some(h) = sha256_hex(&email.trim().to_lowercase()) {
                urls.push(format!("https://www.gravatar.com/avatar/{h}?s={SIZE}&d=404"));
            }
        }
    }
    for u in urls {
        if download(&u, &to) {
            let _ = std::fs::remove_file(&none);
            return Ok(to);
        }
    }
    let _ = std::fs::write(&none, "");
    Err("no picture".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noreply_addresses() {
        assert_eq!(url_from_email("123+bob@users.noreply.github.com").unwrap(), "https://avatars.githubusercontent.com/u/123?s=64");
        assert_eq!(url_from_email("bob@users.noreply.github.com").unwrap(), "https://github.com/bob.png?size=64");
        assert!(url_from_email("noreply@anthropic.com").unwrap().contains("/claude.png"));
        assert!(url_from_email("me@example.com").is_none());
    }

    #[test]
    fn parses_people() {
        let p = super::super::Person::parse("Claude Sonnet 5.5 <noreply@anthropic.com>").unwrap();
        assert_eq!(p.name, "Claude Sonnet 5.5");
        assert_eq!(p.email, "noreply@anthropic.com");
        assert_eq!(p.trailer(), "Co-authored-by: Claude Sonnet 5.5 <noreply@anthropic.com>");
    }
}
