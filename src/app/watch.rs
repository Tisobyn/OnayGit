//! Watch the open repository so the window updates when files change, instead of polling often.
use crate::prelude::*;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct RepoWatch {
    pub path: PathBuf,
    pub dirty: Arc<AtomicBool>,
    _watcher: RecommendedWatcher,
}

/// Does a change at `p` matter for status? Skips build folders and git's own noise.
fn relevant(root: &Path, p: &Path) -> bool {
    let Ok(rel) = p.strip_prefix(root) else { return true };
    let parts: Vec<&str> = rel.components().filter_map(|c| c.as_os_str().to_str()).collect();
    match parts.first() {
        Some(&".git") => matches!(
            parts.get(1),
            Some(&"HEAD" | &"index" | &"refs" | &"packed-refs" | &"MERGE_HEAD" | &"CHERRY_PICK_HEAD" | &"REVERT_HEAD" | &"rebase-merge" | &"rebase-apply" | &"logs")
        ),
        _ => !parts.iter().any(|c| matches!(*c, "target" | "node_modules" | ".DS_Store")),
    }
}

impl App {
    /// Keep one watcher on the open local repository.
    pub fn sync_watcher(&mut self, ctx: &egui::Context) {
        let want = if self.screen == Screen::Repo { self.repo().filter(|r| !git::is_remote(r)) } else { None };
        if self.watch.as_ref().map(|w| &w.path) == want.as_ref() {
            return;
        }
        self.watch = None;
        let Some(path) = want else { return };
        let dirty = Arc::new(AtomicBool::new(false));
        let (flag, ctx2, root) = (dirty.clone(), ctx.clone(), path.clone());
        let made = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res {
                if ev.paths.iter().any(|p| relevant(&root, p)) {
                    flag.store(true, Ordering::Relaxed);
                    ctx2.request_repaint();
                }
            }
        });
        if let Ok(mut w) = made {
            if w.watch(&path, RecursiveMode::Recursive).is_ok() {
                self.watch = Some(RepoWatch { path, dirty, _watcher: w });
            }
        }
    }

    /// True when files changed and a check is due now. Waits a little so a burst of changes is one check.
    pub fn watch_due(&mut self, ctx: &egui::Context) -> bool {
        let Some(w) = &self.watch else { return false };
        if !w.dirty.load(Ordering::Relaxed) {
            return false;
        }
        let wait = Duration::from_millis(400);
        if self.last_refresh.elapsed() < wait {
            ctx.request_repaint_after(wait);
            return false;
        }
        if self.polling || self.busy.is_some() {
            return false;
        }
        w.dirty.store(false, Ordering::Relaxed);
        true
    }
}
