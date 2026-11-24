#[allow(unused_imports)]
use super::*;

pub fn config_get(key: &str) -> String {
    run(Path::new("."), &["config", "--global", key])
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn config_set(key: &str, value: &str) -> Result<String, String> {
    run(Path::new("."), &["config", "--global", key, value])
}
