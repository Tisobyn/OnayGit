use super::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Add,
    Del,
    Ctx,
    Hunk,
    Meta,
    File,
}

#[derive(Clone)]
pub struct DiffLine {
    pub kind: Kind,
    pub old: Option<u32>,
    pub new: Option<u32>,
    pub text: String,
}

pub fn hunk_start(s: &str) -> u32 {
    s.trim_start_matches(['+', '-'])
        .split(',')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(1)
}

static FULL_CONTEXT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Show whole files in diffs instead of three lines around each change.
pub fn set_full_context(on: bool) {
    FULL_CONTEXT.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn full_context() -> bool {
    FULL_CONTEXT.load(std::sync::atomic::Ordering::Relaxed)
}

pub(super) fn context_arg() -> &'static str {
    if full_context() { "-U100000" } else { "-U3" }
}

/// Turn unified diff text into numbered lines. `show_files` keeps file headers.
pub fn parse_diff(text: &str, show_files: bool) -> Vec<DiffLine> {
    let mut out = Vec::new();
    let (mut o, mut n) = (0u32, 0u32);
    let (mut in_hunk, mut seen_file) = (false, false);
    let mk = |kind, old, new, text: &str| DiffLine { kind, old, new, text: text.to_string() };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            in_hunk = false;
            seen_file = true;
            if show_files {
                let name = rest.rsplit(" b/").next().unwrap_or(rest);
                out.push(mk(Kind::File, None, None, name));
            }
            continue;
        }
        if line.starts_with("@@") {
            let mut parts = line.split_whitespace();
            parts.next();
            o = parts.next().map(hunk_start).unwrap_or(1);
            n = parts.next().map(hunk_start).unwrap_or(1);
            in_hunk = true;
            out.push(mk(Kind::Hunk, None, None, line));
            continue;
        }
        if in_hunk {
            let (kind, body) = match line.chars().next() {
                Some('+') => (Kind::Add, &line[1..]),
                Some('-') => (Kind::Del, &line[1..]),
                Some('\\') => (Kind::Meta, line),
                Some(' ') => (Kind::Ctx, &line[1..]),
                _ => (Kind::Ctx, line),
            };
            let (old, new) = match kind {
                Kind::Add => {
                    n += 1;
                    (None, Some(n - 1))
                }
                Kind::Del => {
                    o += 1;
                    (Some(o - 1), None)
                }
                Kind::Ctx => {
                    o += 1;
                    n += 1;
                    (Some(o - 1), Some(n - 1))
                }
                _ => (None, None),
            };
            out.push(mk(kind, old, new, body));
        } else if !seen_file || line.starts_with("Binary files") {
            out.push(mk(Kind::Meta, None, None, line));
        }
    }
    out
}
