//! Thin wrappers around the git command line.
#![allow(unused_imports)]

use std::path::Path;
use std::process::Command;

mod exec;
mod stash;
mod status;
mod tag;
mod worktree;
mod commit;
mod ai;
mod branch;
mod remote;
mod repo;
mod config;
mod conflict;
mod external;
mod graph;
mod hosting;
mod http;
mod keychain;
mod ignore;
mod patch;
mod rewrite;
mod ssh;
mod diff;

pub use exec::*;
pub use stash::*;
pub use status::*;
pub use tag::*;
pub use worktree::*;
pub use commit::*;
pub use ai::*;
pub use branch::*;
pub use remote::*;
pub use repo::*;
pub use config::*;
pub use conflict::*;
pub use external::*;
pub use graph::*;
pub use hosting::*;
pub use http::*;
pub use keychain::*;
pub use ignore::*;
pub use patch::*;
pub use rewrite::*;
pub use ssh::*;
pub use diff::*;
