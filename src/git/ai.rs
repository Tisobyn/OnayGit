//! AI commit messages through Claude Code, Codex, a custom command, or an OpenAI compatible API.
#[allow(unused_imports)]
use super::*;
use serde_json::{json, Value};
use std::io::Write;
use std::process::Stdio;

/// Larger diffs are cut to this many bytes before they are sent.
pub const MAX_DIFF: usize = 64 * 1024;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AiProvider {
    ClaudeCode,
    Codex,
    Custom,
    Api,
}

impl AiProvider {
    pub const ALL: [AiProvider; 4] = [AiProvider::ClaudeCode, AiProvider::Codex, AiProvider::Custom, AiProvider::Api];

    pub fn key(self) -> &'static str {
        match self {
            AiProvider::ClaudeCode => "claude",
            AiProvider::Codex => "codex",
            AiProvider::Custom => "custom",
            AiProvider::Api => "api",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AiProvider::ClaudeCode => "Claude Code",
            AiProvider::Codex => "Codex",
            AiProvider::Custom => "Custom command",
            AiProvider::Api => "API",
        }
    }

    pub fn from_key(k: &str) -> Option<AiProvider> {
        AiProvider::ALL.into_iter().find(|p| p.key() == k)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AiConfig {
    pub provider: AiProvider,
    /// Shell command for the custom provider. It gets the prompt in $ONAY_PROMPT and the diff on stdin.
    pub command: String,
    pub model: String,
    /// Base URL of an OpenAI compatible API, for example https://api.openai.com/v1.
    pub base_url: String,
    /// Name shown in Settings.
    pub name: String,
    /// Extra guidance added to the prompt.
    pub instructions: String,
}

impl AiConfig {
    pub fn prompt(&self) -> String {
        if self.instructions.trim().is_empty() {
            PROMPT.to_string()
        } else {
            format!("{PROMPT}\n\nAdditional instructions: {}", self.instructions.trim())
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            provider: AiProvider::ClaudeCode,
            command: String::new(),
            model: "gpt-4o-mini".into(),
            base_url: "https://api.openai.com/v1".into(),
            name: String::new(),
            instructions: String::new(),
        }
    }
}

pub const AI_SECRET: &str = "ai-api-key";

pub const PROMPT: &str = "Write a git commit message for the diff on stdin. \
Line 1 is a summary in the imperative mood, under 72 characters. \
Optionally add a blank line and a short body. \
Answer with the message only, no quotes and no code fences.";

pub fn trim_diff(diff: &str) -> String {
    if diff.len() <= MAX_DIFF {
        return diff.to_string();
    }
    let mut end = MAX_DIFF;
    while !diff.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[diff cut here]\n", &diff[..end])
}

/// Split the model answer into (summary, body).
pub fn parse_message(out: &str) -> (String, String) {
    let mut text = out.trim().to_string();
    if text.starts_with("```") {
        text = text.lines().skip(1).collect::<Vec<_>>().join("\n");
        text = text.trim_end_matches("```").trim().to_string();
    }
    let text = text.trim_matches('"').trim();
    let mut parts = text.splitn(2, '\n');
    let summary = parts.next().unwrap_or("").trim().trim_start_matches("Summary:").trim().to_string();
    let body = parts.next().unwrap_or("").trim().to_string();
    (summary, body)
}

fn run_shell(command: &str, prompt: &str, diff: &str) -> Result<String, String> {
    let mut child = Command::new("/bin/zsh")
        .args(["-lc", command])
        .env("ONAY_PROMPT", prompt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(diff.as_bytes());
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { "The command failed".into() } else { err })
    }
}

/// Ask the configured provider for a commit message.
pub fn generate(cfg: &AiConfig, api_key: Option<&str>, diff: &str) -> Result<(String, String), String> {
    let diff = trim_diff(diff);
    let raw = match cfg.provider {
        AiProvider::ClaudeCode => run_shell("claude -p \"$ONAY_PROMPT\"", &cfg.prompt(), &diff)?,
        AiProvider::Codex => run_shell("codex exec \"$ONAY_PROMPT\"", &cfg.prompt(), &diff)?,
        AiProvider::Custom => {
            if cfg.command.trim().is_empty() {
                return Err("Set a command in Settings > AI".into());
            }
            run_shell(&cfg.command, &cfg.prompt(), &diff)?
        }
        AiProvider::Api => {
            let key = api_key.ok_or("Add an API key in Settings > AI")?;
            let body = json!({
                "model": cfg.model,
                "messages": [
                    { "role": "system", "content": cfg.prompt() },
                    { "role": "user", "content": diff },
                ],
            });
            let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
            let (code, text) = http("POST", &url, &[format!("Authorization: Bearer {key}")], Some(&body.to_string()))?;
            let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            if code >= 400 || code == 0 {
                let msg = v["error"]["message"].as_str().unwrap_or("request failed");
                return Err(format!("API said {code}: {msg}"));
            }
            v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        }
    };
    let (summary, body) = parse_message(&raw);
    if summary.is_empty() {
        return Err("The AI returned an empty message".into());
    }
    Ok((summary, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_messages() {
        assert_eq!(parse_message("Fix crash\n\nDetails here"), ("Fix crash".into(), "Details here".into()));
        assert_eq!(parse_message("```\nAdd tabs\n\nBody\n```"), ("Add tabs".into(), "Body".into()));
        assert_eq!(parse_message("\"Quoted\""), ("Quoted".into(), String::new()));
        assert_eq!(parse_message("Summary: Do it"), ("Do it".into(), String::new()));
    }

    #[test]
    fn cuts_big_diffs() {
        let big = "é".repeat(MAX_DIFF);
        let t = trim_diff(&big);
        assert!(t.len() <= MAX_DIFF + 20);
        assert!(t.ends_with("[diff cut here]\n"));
        assert_eq!(trim_diff("small"), "small");
    }

    #[test]
    fn custom_command_end_to_end() {
        let cfg = AiConfig {
            provider: AiProvider::Custom,
            command: "cat >/dev/null; printf 'Add thing\\n\\nWhy: %s' \"${ONAY_PROMPT:0:5}\"".into(),
            ..Default::default()
        };
        let (s, b) = generate(&cfg, None, "diff --git a b").unwrap();
        assert_eq!(s, "Add thing");
        assert!(b.starts_with("Why: Write"));
    }
}
