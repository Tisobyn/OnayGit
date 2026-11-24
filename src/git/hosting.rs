//! GitHub and GitLab: accounts, creating repositories, pull / merge requests.
#[allow(unused_imports)]
use super::*;
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Provider {
    GitHub,
    GitLab,
}

impl Provider {
    pub fn key(self) -> &'static str {
        match self {
            Provider::GitHub => "github",
            Provider::GitLab => "gitlab",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::GitHub => "GitHub",
            Provider::GitLab => "GitLab",
        }
    }

    pub fn from_key(k: &str) -> Option<Provider> {
        match k {
            "github" => Some(Provider::GitHub),
            "gitlab" => Some(Provider::GitLab),
            _ => None,
        }
    }

    pub fn default_host(self) -> &'static str {
        match self {
            Provider::GitHub => "github.com",
            Provider::GitLab => "gitlab.com",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub provider: Provider,
    pub host: String,
    pub user: String,
}

impl Account {
    /// Name of the Keychain item that holds the token.
    pub fn secret_name(&self) -> String {
        format!("{}:{}:{}", self.provider.key(), self.host, self.user)
    }

    pub fn token(&self) -> Option<String> {
        secret_get(&self.secret_name())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub head: String,
    pub base: String,
    pub url: String,
    pub draft: bool,
}

impl PullRequest {
    pub fn to_line(&self) -> String {
        let c = |s: &str| s.replace(['\t', '\n'], " ");
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.number,
            c(&self.title),
            c(&self.author),
            c(&self.head),
            c(&self.base),
            c(&self.url),
            self.draft
        )
    }

    pub fn from_line(l: &str) -> Option<PullRequest> {
        let mut p = l.split('\t');
        Some(PullRequest {
            number: p.next()?.parse().ok()?,
            title: p.next()?.to_string(),
            author: p.next()?.to_string(),
            head: p.next()?.to_string(),
            base: p.next()?.to_string(),
            url: p.next()?.to_string(),
            draft: p.next()? == "true",
        })
    }
}

fn api_base(provider: Provider, host: &str) -> String {
    match provider {
        Provider::GitHub if host == "github.com" => "https://api.github.com".to_string(),
        Provider::GitHub => format!("https://{host}/api/v3"),
        Provider::GitLab => format!("https://{host}/api/v4"),
    }
}

fn headers(provider: Provider, token: &str) -> Vec<String> {
    match provider {
        Provider::GitHub => vec![
            format!("Authorization: Bearer {token}"),
            "Accept: application/vnd.github+json".to_string(),
        ],
        Provider::GitLab => vec![format!("PRIVATE-TOKEN: {token}")],
    }
}

fn call(provider: Provider, host: &str, token: &str, method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
    let url = format!("{}{}", api_base(provider, host), path);
    let body = body.map(|b| b.to_string());
    let (code, text) = http(method, &url, &headers(provider, token), body.as_deref())?;
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if code >= 400 || code == 0 {
        let msg = value["message"]
            .as_str()
            .map(String::from)
            .or_else(|| value["error"].as_str().map(String::from))
            .unwrap_or_else(|| text.chars().take(200).collect());
        return Err(format!("{} said {code}: {msg}", provider.label()));
    }
    Ok(value)
}

/// Check the token and return the user name.
/// Token of the signed-in GitHub CLI (`gh auth token`).
pub fn gh_cli_token(host: &str) -> Result<String, String> {
    let out = std::process::Command::new("gh")
        .args(["auth", "token", "--hostname", host])
        .output()
        .map_err(|_| "The GitHub CLI (gh) is not installed".to_string())?;
    let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() && !t.is_empty() {
        Ok(t)
    } else {
        Err("Sign in first with: gh auth login".to_string())
    }
}

pub fn whoami(provider: Provider, host: &str, token: &str) -> Result<String, String> {
    let v = call(provider, host, token, "GET", "/user", None)?;
    let key = if provider == Provider::GitHub { "login" } else { "username" };
    v[key].as_str().map(String::from).ok_or_else(|| "Unexpected answer from the server".to_string())
}

/// Create a repository for the user. Returns its https clone URL.
pub fn create_repo(acc: &Account, token: &str, name: &str, private: bool, description: &str) -> Result<String, String> {
    match acc.provider {
        Provider::GitHub => {
            let body = json!({ "name": name, "private": private, "description": description });
            let v = call(acc.provider, &acc.host, token, "POST", "/user/repos", Some(body))?;
            v["clone_url"].as_str().map(String::from).ok_or_else(|| "No clone URL in the answer".to_string())
        }
        Provider::GitLab => {
            let vis = if private { "private" } else { "public" };
            let body = json!({ "name": name, "visibility": vis, "description": description });
            let v = call(acc.provider, &acc.host, token, "POST", "/projects", Some(body))?;
            v["http_url_to_repo"].as_str().map(String::from).ok_or_else(|| "No clone URL in the answer".to_string())
        }
    }
}

/// `owner/repo` and host from a remote URL.
pub fn parse_remote_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    let (host, path) = if let Some(rest) = url.strip_prefix("git@") {
        let (h, p) = rest.split_once(':')?;
        (h.to_string(), p.to_string())
    } else {
        let rest = url.split_once("://")?.1;
        let rest = rest.split_once('@').map(|x| x.1).unwrap_or(rest);
        let (h, p) = rest.split_once('/')?;
        (h.split(':').next()?.to_string(), p.to_string())
    };
    let slug = path.trim_matches('/').trim_end_matches(".git").to_string();
    (slug.contains('/') && !host.is_empty()).then_some((host, slug))
}

fn encode_slug(slug: &str) -> String {
    slug.replace('/', "%2F")
}

pub fn parse_pull_requests(provider: Provider, v: &Value) -> Vec<PullRequest> {
    v.as_array()
        .map(|a| {
            a.iter()
                .map(|p| match provider {
                    Provider::GitHub => PullRequest {
                        number: p["number"].as_u64().unwrap_or(0),
                        title: p["title"].as_str().unwrap_or("").to_string(),
                        author: p["user"]["login"].as_str().unwrap_or("").to_string(),
                        head: p["head"]["ref"].as_str().unwrap_or("").to_string(),
                        base: p["base"]["ref"].as_str().unwrap_or("").to_string(),
                        url: p["html_url"].as_str().unwrap_or("").to_string(),
                        draft: p["draft"].as_bool().unwrap_or(false),
                    },
                    Provider::GitLab => PullRequest {
                        number: p["iid"].as_u64().unwrap_or(0),
                        title: p["title"].as_str().unwrap_or("").to_string(),
                        author: p["author"]["username"].as_str().unwrap_or("").to_string(),
                        head: p["source_branch"].as_str().unwrap_or("").to_string(),
                        base: p["target_branch"].as_str().unwrap_or("").to_string(),
                        url: p["web_url"].as_str().unwrap_or("").to_string(),
                        draft: p["draft"].as_bool().unwrap_or(false),
                    },
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn list_pull_requests(acc: &Account, token: &str, slug: &str) -> Result<Vec<PullRequest>, String> {
    let path = match acc.provider {
        Provider::GitHub => format!("/repos/{slug}/pulls?state=open&per_page=50"),
        Provider::GitLab => format!("/projects/{}/merge_requests?state=opened&per_page=50", encode_slug(slug)),
    };
    let v = call(acc.provider, &acc.host, token, "GET", &path, None)?;
    Ok(parse_pull_requests(acc.provider, &v))
}

/// Open a pull (or merge) request. Returns its web URL.
pub fn create_pull_request(
    acc: &Account,
    token: &str,
    slug: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
) -> Result<String, String> {
    match acc.provider {
        Provider::GitHub => {
            let b = json!({ "title": title, "body": body, "head": head, "base": base });
            let v = call(acc.provider, &acc.host, token, "POST", &format!("/repos/{slug}/pulls"), Some(b))?;
            v["html_url"].as_str().map(String::from).ok_or_else(|| "No URL in the answer".to_string())
        }
        Provider::GitLab => {
            let b = json!({ "source_branch": head, "target_branch": base, "title": title, "description": body });
            let path = format!("/projects/{}/merge_requests", encode_slug(slug));
            let v = call(acc.provider, &acc.host, token, "POST", &path, Some(b))?;
            v["web_url"].as_str().map(String::from).ok_or_else(|| "No URL in the answer".to_string())
        }
    }
}

/// Environment that lets git push and pull with the account token, without prompts.
/// The token is only in the environment of the git process.
pub fn auth_env(acc: &Account, token: &str) -> Result<Vec<(String, String)>, String> {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join("onaygit-askpass");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let script = dir.join("askpass.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\ncase \"$1\" in\n  Username*) echo \"$ONAYGIT_USER\" ;;\n  *) echo \"$ONAYGIT_TOKEN\" ;;\nesac\n",
    )
    .map_err(|e| e.to_string())?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let user = if acc.provider == Provider::GitHub { "x-access-token".to_string() } else { "oauth2".to_string() };
    Ok(vec![
        ("GIT_ASKPASS".into(), script.to_string_lossy().into_owned()),
        ("GIT_TERMINAL_PROMPT".into(), "0".into()),
        ("ONAYGIT_USER".into(), user),
        ("ONAYGIT_TOKEN".into(), token.to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_urls() {
        assert_eq!(parse_remote_url("git@github.com:me/app.git"), Some(("github.com".into(), "me/app".into())));
        assert_eq!(parse_remote_url("https://github.com/me/app.git"), Some(("github.com".into(), "me/app".into())));
        assert_eq!(parse_remote_url("https://user:pw@gitlab.com/grp/sub/app"), Some(("gitlab.com".into(), "grp/sub/app".into())));
        assert_eq!(parse_remote_url("ssh://git@host:2222/me/app.git"), Some(("host".into(), "me/app".into())));
        assert_eq!(parse_remote_url("/local/path"), None);
    }

    #[test]
    fn parses_github_and_gitlab_lists() {
        let gh: Value = serde_json::from_str(
            r#"[{"number":7,"title":"Fix","user":{"login":"ann"},"head":{"ref":"fix"},"base":{"ref":"main"},"html_url":"https://github.com/o/r/pull/7","draft":true}]"#,
        )
        .unwrap();
        let l = parse_pull_requests(Provider::GitHub, &gh);
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].number, l[0].author.as_str(), l[0].head.as_str(), l[0].draft), (7, "ann", "fix", true));

        let gl: Value = serde_json::from_str(
            r#"[{"iid":3,"title":"Add","author":{"username":"bob"},"source_branch":"feat","target_branch":"main","web_url":"https://gitlab.com/o/r/-/merge_requests/3"}]"#,
        )
        .unwrap();
        let l = parse_pull_requests(Provider::GitLab, &gl);
        assert_eq!((l[0].number, l[0].base.as_str(), l[0].url.as_str()), (3, "main", "https://gitlab.com/o/r/-/merge_requests/3"));
    }

    #[test]
    fn pull_request_lines_roundtrip() {
        let p = PullRequest {
            number: 9,
            title: "A\tb".into(),
            author: "me".into(),
            head: "h".into(),
            base: "main".into(),
            url: "https://x".into(),
            draft: true,
        };
        let back = PullRequest::from_line(&p.to_line()).unwrap();
        assert_eq!(back.title, "A b");
        assert_eq!((back.number, back.draft), (9, true));
    }

    #[test]
    fn api_urls_and_headers() {
        assert_eq!(api_base(Provider::GitHub, "github.com"), "https://api.github.com");
        assert_eq!(api_base(Provider::GitHub, "ghe.corp"), "https://ghe.corp/api/v3");
        assert_eq!(api_base(Provider::GitLab, "gitlab.com"), "https://gitlab.com/api/v4");
        assert_eq!(headers(Provider::GitLab, "t"), vec!["PRIVATE-TOKEN: t".to_string()]);
        assert_eq!(encode_slug("a/b/c"), "a%2Fb%2Fc");
    }
}
