//! Tiny HTTP client on top of `curl`. The request (and the token in it) goes through
//! stdin as a curl config, so secrets never appear in the process list.
#[allow(unused_imports)]
use super::*;
use std::io::Write;
use std::process::Stdio;

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}

/// The curl config text for a request.
pub fn curl_config(method: &str, url: &str, headers: &[String], body: Option<&str>) -> String {
    let mut c = String::new();
    c.push_str(&format!("url = \"{}\"\n", esc(url)));
    c.push_str(&format!("request = \"{}\"\n", esc(method)));
    c.push_str("silent\nshow-error\nmax-time = 60\n");
    c.push_str("write-out = \"\\n%{http_code}\"\n");
    c.push_str("header = \"User-Agent: OnayGit\"\n");
    for h in headers {
        c.push_str(&format!("header = \"{}\"\n", esc(h)));
    }
    if let Some(b) = body {
        c.push_str("header = \"Content-Type: application/json\"\n");
        c.push_str(&format!("data = \"{}\"\n", esc(b)));
    }
    c
}

/// Returns (status code, body).
pub fn http(method: &str, url: &str, headers: &[String], body: Option<&str>) -> Result<(u16, String), String> {
    let mut child = Command::new("curl")
        .args(["-K", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run curl: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(curl_config(method, url, headers, body).as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let (body, code) = text.rsplit_once('\n').unwrap_or(("", &text));
    Ok((code.trim().parse().unwrap_or(0), body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_escapes_and_hides_nothing_in_args() {
        let c = curl_config("POST", "https://x/y", &["Authorization: Bearer t\"k".into()], Some("{\"a\":\"b\\n\"}"));
        assert!(c.contains("header = \"Authorization: Bearer t\\\"k\""));
        assert!(c.contains("data = \"{\\\"a\\\":\\\"b\\\\n\\\"}\""));
        assert!(c.contains("request = \"POST\""));
    }
}
