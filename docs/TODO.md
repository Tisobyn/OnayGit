# OnayGit to-do

Ten items, in order. Built from [REVIEW.md](REVIEW.md) and [AUDIT.md](AUDIT.md).
Done items are checked. Tick the box when you finish one.

## Stability and speed

- [x] **1. Keep the UI thread free.** Timer check and commit run on threads. Git never waits for a password prompt. *(done)*
- [ ] **2. Move the full `refresh()` off the UI thread.** Status, log, branches, tags, graph and stashes still load in one go after actions. Load them on a worker, show a spinner meanwhile. Also covers `select_commit`, `load_graph`, checkout and merge.
- [ ] **3. Command timeouts.** `git::run` has no limit. Add a timeout and a Cancel button for fetch, pull, push and clone.
- [x] **4. Watch files instead of polling.** Done with the `notify` crate.

## Daily comfort

- [x] **5. Conflict editor.** Done: per conflict Ours, Theirs, Both or Edit.
- [x] **6. Search in History.** Done: words, author:, file:, since:, until:.
- [x] **7. One window.** Done: `onaygit .` opens the folder in the running app.
- [x] **8. Loading and density.** Done: loading overlay, rail labels option, compact lists, full date on hover. The load itself still blocks the UI (see item 2).

## Release and code health

- [ ] **9. Ship it.** Notarize (needs Apple Developer ID), publish a GitHub Release with the zip, fill the Releases link, add an update check.
- [ ] **10. Clean structure.** Error enum for network and auth. `platform` module (Keychain, `osascript`, `open`) for Linux and Windows. Split `settings_page.rs`. Group `App` fields. Add UI state tests.

## Later

Interactive rebase list, submodules and LFS, commit signing, hook output, notifications, localization, undo for discard.
