//! Rewrite history without an interactive editor: drop, squash and reword.
use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RewriteOp {
    /// Remove the commit.
    Drop,
    /// Fold the commit into the one before it.
    Squash,
    /// Change the commit message.
    Reword,
    /// Swap the commit with the one before it (it becomes older).
    MoveOlder,
}

fn temp_dir() -> Result<std::path::PathBuf, String> {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("onaygit-rewrite-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn rewrite(repo: &Path, op: RewriteOp, hash: &str, message: &str) -> Result<String, String> {
    let parent = format!("{hash}^");
    if run(repo, &["rev-parse", "--verify", "-q", &parent]).is_err() {
        return Err("The first commit cannot be rewritten this way".into());
    }

    if op == RewriteOp::Drop {
        return run_env(repo, &["rebase", "--autostash", "--onto", &parent, hash], &[]);
    }

    // Mark the commit in the todo list through a tiny editor script.
    use std::os::unix::fs::PermissionsExt;
    let dir = temp_dir()?;
    let action = if op == RewriteOp::Squash { "squash" } else { "reword" };
    let script = dir.join("seq.sh");
    let body = if op == RewriteOp::MoveOlder {
        format!(
            "#!/bin/sh\nperl -e 'open F,\"<\",$ARGV[0]; @l=<F>; close F; for $i (1..$#l){{ if($l[$i]=~/^pick (\\w+)/ && index(\"{hash}\",$1)==0 && $l[$i-1]=~/^pick /){{ @l[$i-1,$i]=@l[$i,$i-1]; last }} }} open F,\">\",$ARGV[0]; print F @l;' \"$1\"\n"
        )
    } else {
        format!(
            "#!/bin/sh\nperl -pi -e 's/^pick (\\w+)/(index(\"{hash}\", $1) == 0) ? \"{action} $1\" : \"pick $1\"/e' \"$1\"\n"
        )
    };
    std::fs::write(&script, body).map_err(|e| e.to_string())?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;

    let editor = if op == RewriteOp::Reword {
        let msg = dir.join("message.txt");
        std::fs::write(&msg, format!("{}\n", message.trim())).map_err(|e| e.to_string())?;
        format!("cp '{}'", msg.display())
    } else {
        "true".to_string()
    };
    let seq = script.to_string_lossy().into_owned();
    let env = [("GIT_SEQUENCE_EDITOR", seq.as_str()), ("GIT_EDITOR", editor.as_str())];
    if op == RewriteOp::Squash || op == RewriteOp::MoveOlder {
        // The todo list must hold the previous commit too, so start one commit earlier.
        let grand = format!("{hash}^^");
        if run(repo, &["rev-parse", "--verify", "-q", &grand]).is_err() {
            return run_env(repo, &["rebase", "-i", "--autostash", "--root"], &env);
        }
        return run_env(repo, &["rebase", "-i", "--autostash", &grand], &env);
    }
    run_env(repo, &["rebase", "-i", "--autostash", &parent], &env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(d: &Path, a: &[&str]) -> String {
        run(d, a).unwrap()
    }

    #[test]
    fn move_commit_older() {
        let d = std::env::temp_dir().join(format!("onaygit-move-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        g(&d, &["init", "-q", "-b", "main"]);
        g(&d, &["config", "user.email", "a@b.c"]);
        g(&d, &["config", "user.name", "t"]);
        for n in ["one", "two", "three"] {
            std::fs::write(d.join(format!("{n}.txt")), n).unwrap();
            g(&d, &["add", "."]);
            g(&d, &["commit", "-q", "-m", n]);
        }
        let head = g(&d, &["rev-parse", "HEAD"]).trim().to_string();
        rewrite(&d, RewriteOp::MoveOlder, &head, "").unwrap();
        let log = g(&d, &["log", "--format=%s"]);
        assert_eq!(log.lines().collect::<Vec<_>>(), vec!["two", "three", "one"]);
        let _ = std::fs::remove_dir_all(&d);
    }
}
