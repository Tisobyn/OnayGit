//! Partial staging: build a patch from selected diff lines and apply it to the index.
use super::*;
use std::collections::HashSet;

fn parse_header(h: &str) -> (u32, u32) {
    let mut parts = h.split_whitespace();
    parts.next();
    let a = parts.next().map(hunk_start).unwrap_or(1);
    let b = parts.next().map(hunk_start).unwrap_or(1);
    (a, b)
}

/// Build a patch for the selected changed lines.
///
/// `lines` is the parsed form of `raw`. `selected` holds indexes into `lines`.
/// With `only_hunk` the whole hunk at that index is used.
/// `reverse` builds a patch meant for `git apply --cached -R` (unstaging).
pub fn build_patch(
    raw: &str,
    path: &str,
    lines: &[DiffLine],
    selected: &HashSet<usize>,
    only_hunk: Option<usize>,
    reverse: bool,
) -> Option<String> {
    let mut header = String::new();
    for l in raw.lines() {
        if l.starts_with("@@") {
            break;
        }
        header.push_str(l);
        header.push('\n');
    }
    // New and deleted files become plain edits of an empty or existing file.
    if header.contains("/dev/null") {
        header = format!("diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n");
    }

    let hunks: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind == Kind::Hunk)
        .map(|(i, _)| i)
        .collect();

    let mut out = String::new();
    for (n, &h) in hunks.iter().enumerate() {
        if only_hunk.is_some_and(|o| o != h) {
            continue;
        }
        let end = hunks.get(n + 1).copied().unwrap_or(lines.len());
        let (old_start, new_start) = parse_header(&lines[h].text);

        let mut body = String::new();
        let (mut oc, mut nc) = (0u32, 0u32);
        let (mut changed, mut last) = (false, false);
        for (i, l) in lines.iter().enumerate().take(end).skip(h + 1) {
            let sel = only_hunk.is_some() || selected.contains(&i);
            match l.kind {
                Kind::Ctx => {
                    body.push_str(&format!(" {}\n", l.text));
                    oc += 1;
                    nc += 1;
                    last = true;
                }
                Kind::Add if sel => {
                    body.push_str(&format!("+{}\n", l.text));
                    nc += 1;
                    changed = true;
                    last = true;
                }
                Kind::Del if sel => {
                    body.push_str(&format!("-{}\n", l.text));
                    oc += 1;
                    changed = true;
                    last = true;
                }
                // Not selected: staging keeps a removed line as context,
                // unstaging keeps an added line as context. Otherwise skip it.
                Kind::Add if reverse => {
                    body.push_str(&format!(" {}\n", l.text));
                    oc += 1;
                    nc += 1;
                    last = true;
                }
                Kind::Del if !reverse => {
                    body.push_str(&format!(" {}\n", l.text));
                    oc += 1;
                    nc += 1;
                    last = true;
                }
                Kind::Add | Kind::Del => last = false,
                Kind::Meta
                    if last => {
                        body.push_str(&l.text);
                        body.push('\n');
                    }
                _ => {}
            }
        }
        if !changed {
            continue;
        }

        // Position in the file the patch is applied to, then the other side.
        let (target_start, target_count, other_count) =
            if reverse { (new_start, nc, oc) } else { (old_start, oc, nc) };
        let other_start = match (other_count == 0, target_count == 0) {
            (true, true) => target_start,
            (true, false) => target_start.saturating_sub(1),
            (false, true) => target_start + 1,
            (false, false) => target_start,
        };
        let (os, ns) = if reverse { (other_start, target_start) } else { (target_start, other_start) };
        out.push_str(&format!("@@ -{os},{oc} +{ns},{nc} @@\n{body}"));
    }

    if out.is_empty() {
        None
    } else {
        Some(format!("{header}{out}"))
    }
}

pub fn apply_patch(repo: &Path, patch: &str, reverse: bool) -> Result<(), String> {
    let mut args = vec!["apply", "--cached", "--whitespace=nowarn"];
    if reverse {
        args.push("-R");
    }
    args.push("-");
    run_stdin(repo, &args, patch).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(dir: &Path, args: &[&str]) -> String {
        run(dir, args).unwrap_or_else(|e| panic!("git {args:?} failed: {e}"))
    }

    fn setup(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("onaygit-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        sh(&dir, &["init", "-q"]);
        sh(&dir, &["config", "user.email", "t@t"]);
        sh(&dir, &["config", "user.name", "t"]);
        std::fs::write(dir.join("f.txt"), "a\nb\nc\nd\ne\n").unwrap();
        sh(&dir, &["add", "."]);
        sh(&dir, &["commit", "-qm", "init"]);
        dir
    }

    #[test]
    fn stage_and_unstage_single_lines() {
        let dir = setup("lines");
        std::fs::write(dir.join("f.txt"), "a\nB\nc\nd\ne\nf\n").unwrap();

        let raw = run(&dir, &["diff", "--", "f.txt"]).unwrap();
        let lines = parse_diff(&raw, false);
        // Select only the added line "f".
        let add_f = lines.iter().position(|l| l.kind == Kind::Add && l.text == "f").unwrap();
        let sel: HashSet<usize> = [add_f].into();
        let patch = build_patch(&raw, "f.txt", &lines, &sel, None, false).unwrap();
        apply_patch(&dir, &patch, false).unwrap();

        let cached = run(&dir, &["diff", "--cached", "--", "f.txt"]).unwrap();
        assert!(cached.contains("+f"), "{cached}");
        assert!(!cached.contains("+B"), "{cached}");
        assert!(!cached.contains("-b"), "{cached}");

        // Now unstage that line again.
        let raw = run(&dir, &["diff", "--cached", "--", "f.txt"]).unwrap();
        let lines = parse_diff(&raw, false);
        let add_f = lines.iter().position(|l| l.kind == Kind::Add && l.text == "f").unwrap();
        let sel: HashSet<usize> = [add_f].into();
        let patch = build_patch(&raw, "f.txt", &lines, &sel, None, true).unwrap();
        apply_patch(&dir, &patch, true).unwrap();
        assert!(run(&dir, &["diff", "--cached"]).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stage_whole_hunk_and_partial_new_file() {
        let dir = setup("hunk");
        std::fs::write(dir.join("f.txt"), "a\nB\nc\nd\ne\n").unwrap();
        let raw = run(&dir, &["diff", "--", "f.txt"]).unwrap();
        let lines = parse_diff(&raw, false);
        let h = lines.iter().position(|l| l.kind == Kind::Hunk).unwrap();
        let patch = build_patch(&raw, "f.txt", &lines, &HashSet::new(), Some(h), false).unwrap();
        apply_patch(&dir, &patch, false).unwrap();
        let cached = run(&dir, &["diff", "--cached"]).unwrap();
        assert!(cached.contains("+B") && cached.contains("-b"), "{cached}");

        // Untracked file: stage only the second line.
        std::fs::write(dir.join("new.txt"), "one\ntwo\nthree\n").unwrap();
        sh(&dir, &["add", "-N", "--", "new.txt"]);
        let raw = run(&dir, &["diff", "--", "new.txt"]).unwrap();
        let lines = parse_diff(&raw, false);
        let two = lines.iter().position(|l| l.text == "two").unwrap();
        let sel: HashSet<usize> = [two].into();
        let patch = build_patch(&raw, "new.txt", &lines, &sel, None, false).unwrap();
        apply_patch(&dir, &patch, false).unwrap();
        let staged = sh(&dir, &["show", ":new.txt"]);
        assert_eq!(staged, "two\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
