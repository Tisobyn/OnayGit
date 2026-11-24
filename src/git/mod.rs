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
mod branch;
mod repo;
mod config;
mod conflict;
mod external;
mod graph;
mod ignore;
mod patch;
mod rewrite;
mod diff;

pub use exec::*;
pub use stash::*;
pub use status::*;
pub use tag::*;
pub use worktree::*;
pub use commit::*;
pub use branch::*;
pub use repo::*;
pub use config::*;
pub use conflict::*;
pub use external::*;
pub use graph::*;
pub use ignore::*;
pub use patch::*;
pub use rewrite::*;
pub use diff::*;
