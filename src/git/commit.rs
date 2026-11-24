use super::*;

#[derive(Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub email: String,
    /// People named in `Co-authored-by` trailers.
    pub coauthors: Vec<Person>,
    pub when: String,
    pub subject: String,
    pub parents: Vec<String>,
}

/// A name and email, as in `Name <email>`.
#[derive(Clone, Debug, PartialEq)]
pub struct Person {
    pub name: String,
    pub email: String,
}

impl Person {
    /// Parse `Name <email>`. Without angle brackets the whole text is the name.
    pub fn parse(s: &str) -> Option<Person> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        match (s.rfind('<'), s.rfind('>')) {
            (Some(a), Some(b)) if a < b => Some(Person { name: s[..a].trim().to_string(), email: s[a + 1..b].trim().to_string() }),
            _ => Some(Person { name: s.to_string(), email: String::new() }),
        }
    }

    pub fn trailer(&self) -> String {
        format!("Co-authored-by: {} <{}>", self.name, self.email)
    }
}

impl Commit {
    /// The author followed by the co-authors, without repeats.
    pub fn people(&self) -> Vec<Person> {
        let mut v = vec![Person { name: self.author.clone(), email: self.email.clone() }];
        for c in &self.coauthors {
            if !v.iter().any(|p| p.email.eq_ignore_ascii_case(&c.email) && !c.email.is_empty()) && c.name != self.author {
                v.push(c.clone());
            }
        }
        v
    }
}

/// `body` with a `Co-authored-by` line for each person not named in it yet.
pub fn with_coauthors(body: &str, people: &[Person]) -> String {
    let lower = body.to_lowercase();
    let lines: Vec<String> = people.iter().filter(|p| !lower.contains(&p.email.to_lowercase())).map(|p| p.trailer()).collect();
    if lines.is_empty() {
        return body.to_string();
    }
    let has_trailers = body.lines().last().is_some_and(|l| l.trim().to_lowercase().starts_with("co-authored-by:"));
    let sep = if body.trim().is_empty() { "" } else if has_trailers { "\n" } else { "\n\n" };
    format!("{}{sep}{}", body.trim_end(), lines.join("\n"))
}

#[cfg(test)]
pub fn log(repo: &Path) -> Vec<Commit> {
    log_range(repo, "HEAD")
}

/// Commits in a revision range such as `HEAD..feature`, at most 300.
pub fn log_range(repo: &Path, range: &str) -> Vec<Commit> {
    log_n(repo, range, 300)
}

/// The newest `n` commits of a range or ref such as `HEAD` or `--all`.
pub fn log_n(repo: &Path, range: &str, n: usize) -> Vec<Commit> {
    let max = format!("--max-count={n}");
    let mut args = vec!["log", &max, "--topo-order", LOG_FORMAT];
    args.extend(range.split_whitespace());
    parse_log(&run(repo, &args).unwrap_or_default())
}

const LOG_FORMAT: &str = "--format=%H%x1f%an%x1f%ar%x1f%s%x1f%P%x1f%ae%x1f%(trailers:key=Co-authored-by,valueonly,unfold,separator=%x1e)";

fn parse_log(text: &str) -> Vec<Commit> {
    text.lines()
        .filter_map(|l| {
            let mut p = l.split('\x1f');
            Some(Commit {
                hash: p.next()?.to_string(),
                author: p.next()?.to_string(),
                when: p.next()?.to_string(),
                subject: p.next()?.to_string(),
                parents: p.next().unwrap_or("").split_whitespace().map(String::from).collect(),
                email: p.next().unwrap_or("").to_string(),
                coauthors: p.next().unwrap_or("").split('\x1e').filter_map(Person::parse).collect(),
            })
        })
        .collect()
}

/// Commits that changed one file, following renames.
pub fn file_log(repo: &Path, path: &str) -> Vec<Commit> {
    parse_log(&run(repo, &["log", "--follow", "--max-count=300", LOG_FORMAT, "--", path]).unwrap_or_default())
}

/// One line of `git blame`: (short hash, author, text).
pub fn blame(repo: &Path, path: &str) -> Vec<(String, String, String)> {
    let out = run(repo, &["blame", "--line-porcelain", "--", path]).unwrap_or_default();
    let mut rows = Vec::new();
    let (mut hash, mut author) = (String::new(), String::new());
    for l in out.lines() {
        if let Some(t) = l.strip_prefix('\t') {
            rows.push((hash.chars().take(7).collect(), author.clone(), t.to_string()));
            if rows.len() >= 5000 {
                break;
            }
        } else if let Some(a) = l.strip_prefix("author ") {
            author = a.to_string();
        } else if l.len() >= 40 && l.as_bytes()[..40].iter().all(|b| b.is_ascii_hexdigit()) && l.as_bytes().get(40) == Some(&b' ') {
            hash = l[..40].to_string();
        }
    }
    rows
}

pub fn show(repo: &Path, hash: &str) -> String {
    run(repo, &["show", "--format=%an <%ae>%n%ad%n%n%B", hash]).unwrap_or_default()
}

/// (subject, body) of HEAD.
pub fn last_commit(repo: &Path) -> Option<(String, String)> {
    let out = run(repo, &["log", "-1", "--format=%s%x1f%b"]).ok()?;
    let (s, b) = out.split_once('\x1f')?;
    Some((s.trim().to_string(), b.trim().to_string()))
}

/// Undo the last commit, keeping its changes staged.
pub fn undo_commit(repo: &Path) -> Result<String, String> {
    if run(repo, &["rev-parse", "--verify", "-q", "HEAD~1"]).is_ok() {
        run(repo, &["reset", "--soft", "HEAD~1"])
    } else {
        run(repo, &["update-ref", "-d", "HEAD"])
    }
}

/// Files changed by a commit as (status letter, path).
pub fn commit_files(repo: &Path, hash: &str) -> Vec<(char, String)> {
    run(repo, &["show", "--name-status", "--no-renames", "--format=", hash])
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (st, path) = l.split_once('\t')?;
            Some((st.chars().next()?, path.to_string()))
        })
        .collect()
}

/// Diff of one file in one commit.
pub fn show_file(repo: &Path, hash: &str, path: &str) -> String {
    run(repo, &["show", super::diff::context_arg(), "--format=", hash, "--", path]).unwrap_or_default()
}

pub fn cherry_pick(repo: &Path, hash: &str) -> Result<String, String> {
    run(repo, &["cherry-pick", hash])
}

pub fn revert(repo: &Path, hash: &str) -> Result<String, String> {
    run(repo, &["revert", "--no-edit", hash])
}

/// `mode` is soft, mixed or hard.
pub fn reset_to(repo: &Path, hash: &str, mode: &str) -> Result<String, String> {
    run(repo, &["reset", &format!("--{mode}"), hash])
}

#[cfg(test)]
mod coauthor_tests {
    use super::*;

    fn claude() -> Person {
        Person { name: "Claude".into(), email: "noreply@anthropic.com".into() }
    }

    #[test]
    fn adds_trailer_after_blank_line() {
        assert_eq!(with_coauthors("", &[claude()]), "Co-authored-by: Claude <noreply@anthropic.com>");
        assert_eq!(with_coauthors("Body", &[claude()]), "Body\n\nCo-authored-by: Claude <noreply@anthropic.com>");
    }

    #[test]
    fn no_duplicates() {
        let b = "Body\n\nCo-authored-by: Claude <noreply@anthropic.com>";
        assert_eq!(with_coauthors(b, &[claude()]), b);
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;
    use std::path::PathBuf;

    fn repo(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("onaygit-hist-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        for a in [&["init", "-q", "-b", "main"][..], &["config", "user.email", "t@t"], &["config", "user.name", "Tester"]] {
            run(&d, a).unwrap();
        }
        let commit = |file: &str, text: &str, msg: &str| {
            std::fs::write(d.join(file), text).unwrap();
            run(&d, &["add", "."]).unwrap();
            run(&d, &["commit", "-qm", msg]).unwrap();
        };
        commit("a.txt", "one\n", "first");
        commit("b.txt", "other\n", "second");
        commit("a.txt", "one\ntwo\n", "third");
        d
    }

    #[test]
    fn log_n_limits_and_file_log_follows_one_file() {
        let d = repo("log");
        assert_eq!(log_n(&d, "HEAD", 2).len(), 2);
        assert_eq!(log_n(&d, "HEAD", 10).len(), 3);
        let f = file_log(&d, "a.txt");
        assert_eq!(f.iter().map(|c| c.subject.as_str()).collect::<Vec<_>>(), ["third", "first"]);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn blame_gives_author_and_text_per_line() {
        let d = repo("blame");
        let b = blame(&d, "a.txt");
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].2, "one");
        assert_eq!(b[1].2, "two");
        assert_eq!(b[0].1, "Tester");
        assert_ne!(b[0].0, b[1].0);
        let _ = std::fs::remove_dir_all(d);
    }
}
