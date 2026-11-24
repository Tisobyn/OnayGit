//! Small syntax highlighter for diff lines: keywords, strings, comments, numbers, types, calls.
use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tok {
    Plain,
    Keyword,
    Str,
    Comment,
    Number,
    Func,
    Type,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang {
    None,
    Rust,
    Js,
    Python,
    Go,
    C,
    Swift,
    Shell,
    Config,
    Css,
}

pub fn lang_of(path: &str) -> Lang {
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "rs" => Lang::Rust,
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "json" => Lang::Js,
        "py" | "rb" => Lang::Python,
        "go" => Lang::Go,
        "c" | "h" | "cpp" | "cc" | "hpp" | "java" | "kt" | "cs" | "m" | "mm" => Lang::C,
        "swift" => Lang::Swift,
        "sh" | "zsh" | "bash" => Lang::Shell,
        "toml" | "yaml" | "yml" | "ini" | "conf" => Lang::Config,
        "css" | "scss" => Lang::Css,
        _ if name == "Dockerfile" || name == "Makefile" => Lang::Shell,
        _ => Lang::None,
    }
}

fn keywords(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Rust => &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
        ],
        Lang::Js => &[
            "async", "await", "break", "case", "catch", "class", "const", "continue", "default", "else", "enum", "export", "extends", "false", "finally", "for", "from",
            "function", "if", "implements", "import", "in", "instanceof", "interface", "let", "new", "null", "of", "return", "static", "super", "switch", "this", "throw",
            "true", "try", "type", "typeof", "undefined", "var", "void", "while", "yield",
        ],
        Lang::Python => &[
            "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "end", "except", "False", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "None", "not", "or", "pass", "raise", "return", "self", "True", "try", "while", "with", "yield",
        ],
        Lang::Go => &[
            "break", "case", "chan", "const", "continue", "default", "defer", "else", "false", "for", "func", "go", "if", "import", "interface", "map", "nil", "package",
            "range", "return", "select", "struct", "switch", "true", "type", "var",
        ],
        Lang::C => &[
            "abstract", "auto", "break", "case", "catch", "char", "class", "const", "continue", "default", "do", "double", "else", "enum", "extends", "false", "final", "float",
            "for", "fun", "if", "implements", "import", "int", "interface", "long", "namespace", "new", "null", "override", "package", "private", "protected", "public",
            "return", "short", "static", "struct", "super", "switch", "this", "throw", "true", "try", "typedef", "using", "val", "var", "void", "volatile", "while",
        ],
        Lang::Swift => &[
            "actor", "as", "async", "await", "break", "case", "catch", "class", "continue", "default", "defer", "else", "enum", "extension", "false", "for", "func", "guard",
            "if", "import", "in", "init", "let", "nil", "private", "protocol", "public", "return", "self", "static", "struct", "super", "switch", "throw", "true", "try",
            "var", "where", "while",
        ],
        Lang::Shell => &["case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function", "if", "in", "local", "return", "then", "while"],
        Lang::Config => &["true", "false", "null"],
        _ => &[],
    }
}

/// Split one line into coloured spans. `in_block` carries an open `/* */` comment from the line before.
pub fn highlight(lang: Lang, line: &str, in_block: &mut bool) -> Vec<(Range<usize>, Tok)> {
    let mut out: Vec<(Range<usize>, Tok)> = Vec::new();
    if lang == Lang::None {
        return out;
    }
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let n = chars.len();
    let end = |k: usize| if k >= n { line.len() } else { chars[k].0 };
    let line_comment: &[&str] = match lang {
        Lang::Python | Lang::Shell | Lang::Config => &["#"],
        Lang::Css => &[],
        _ => &["//"],
    };
    let block = matches!(lang, Lang::Rust | Lang::Js | Lang::Go | Lang::C | Lang::Swift | Lang::Css);
    let kws = keywords(lang);
    let mut i = 0;
    while i < n {
        let c = chars[i].1;
        let rest = &line[chars[i].0..];
        if *in_block {
            match rest.find("*/") {
                Some(p) => {
                    let stop = chars[i].0 + p + 2;
                    out.push((chars[i].0..stop, Tok::Comment));
                    *in_block = false;
                    while i < n && chars[i].0 < stop {
                        i += 1;
                    }
                }
                None => {
                    out.push((chars[i].0..line.len(), Tok::Comment));
                    return out;
                }
            }
            continue;
        }
        if block && rest.starts_with("/*") {
            *in_block = true;
            out.push((chars[i].0..chars[i].0 + 2, Tok::Comment));
            i += 2;
            continue;
        }
        if line_comment.iter().any(|m| rest.starts_with(m)) {
            out.push((chars[i].0..line.len(), Tok::Comment));
            return out;
        }
        if c == '"' || c == '`' || (c == '\'' && !(lang == Lang::Rust && !rest[1..].chars().take(3).any(|x| x == '\''))) {
            let mut j = i + 1;
            while j < n && chars[j].1 != c {
                if chars[j].1 == '\\' {
                    j += 1;
                }
                j += 1;
            }
            let stop = end((j + 1).min(n));
            out.push((chars[i].0..stop, Tok::Str));
            i = j + 1;
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            while j < n && (chars[j].1.is_ascii_alphanumeric() || chars[j].1 == '.' || chars[j].1 == '_') {
                j += 1;
            }
            out.push((chars[i].0..end(j), Tok::Number));
            i = j;
            continue;
        }
        if c.is_alphabetic() || c == '_' || (c == '$' && lang == Lang::Shell) {
            let mut j = i + 1;
            while j < n && (chars[j].1.is_alphanumeric() || chars[j].1 == '_') {
                j += 1;
            }
            let word = &line[chars[i].0..end(j)];
            let next = line[end(j)..].chars().next();
            let tok = if kws.contains(&word) {
                Tok::Keyword
            } else if next == Some('(') || (lang == Lang::Rust && next == Some('!')) {
                Tok::Func
            } else if c.is_uppercase() && word.len() > 1 && lang != Lang::Shell && lang != Lang::Config {
                Tok::Type
            } else {
                Tok::Plain
            };
            if tok != Tok::Plain {
                out.push((chars[i].0..end(j), tok));
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_line() {
        let mut b = false;
        let s = highlight(Lang::Rust, "pub fn go(x: u32) -> Foo { // hi", &mut b);
        assert!(s.iter().any(|(r, t)| *t == Tok::Keyword && r.start == 0 && r.end == 3));
        assert!(s.iter().any(|(_, t)| *t == Tok::Func));
        assert!(s.iter().any(|(_, t)| *t == Tok::Type));
        assert!(s.iter().any(|(_, t)| *t == Tok::Comment));
    }

    #[test]
    fn block_comment_spans_lines() {
        let mut b = false;
        highlight(Lang::Js, "a /* start", &mut b);
        assert!(b);
        let s = highlight(Lang::Js, "still */ let x = \"é\";", &mut b);
        assert!(!b);
        assert!(s.iter().any(|(_, t)| *t == Tok::Str));
    }
}
