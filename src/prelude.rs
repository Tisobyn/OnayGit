//! Common imports shared by every module.
#![allow(unused_imports)]

pub use crate::app::refs::StashOp;
pub use crate::app::*;
pub(crate) use crate::git;
pub use crate::git::{Commit, DiffLine, FileChange, Kind};
pub use crate::settings::*;
pub use crate::ui::diff_view::*;
pub use crate::ui::icons::*;
pub use crate::ui::theme::*;
pub use crate::ui::widgets::*;
pub use eframe::egui::{
    self,
    text::{LayoutJob, TextFormat},
    Align2, Color32, FontId, RichText, Sense,
};
pub use std::collections::HashSet;
pub use std::path::{Path, PathBuf};
pub use std::sync::mpsc::{channel, Receiver, Sender};
pub use std::time::{Duration, Instant};
