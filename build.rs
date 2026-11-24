use std::process::Command;

fn run(args: &[&str]) -> String {
    Command::new(args[0]).args(&args[1..]).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn main() {
    let hash = run(&["git", "rev-parse", "--short", "HEAD"]);
    let date = run(&["date", "+%Y-%m-%d"]);
    println!("cargo:rustc-env=ONAYGIT_GIT_HASH={}", if hash.is_empty() { "unknown".into() } else { hash });
    println!("cargo:rustc-env=ONAYGIT_BUILD_DATE={date}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
