# OnayGit to-do

Ten items, in order. Built from [REVIEW.md](REVIEW.md) and [AUDIT.md](AUDIT.md).
Done items are checked. Tick the box when you finish one.

## Stability and speed

- [x] **1. Keep the UI thread free.** Timer check and commit run on threads. Git never waits for a password prompt. *(done)*
- [ ] **2. Move the full `refresh()` off the UI thread.** Status, log, branches, tags, graph and stashes still load in one go after actions. Load them on a worker, show a spinner meanwhile. Also covers `select_commit`, `load_graph`, checkout and merge.
- [ ] **3. Command timeouts.** `git::run` has no limit. Add a timeout and a Cancel button for fetch, pull, push and clone.
- [ ] **4. Watch files instead of polling.** Use the `notify` crate. Refresh on real changes. Saves battery.

## Daily comfort

- [ ] **5. Conflict editor.** Inline editor per conflict, not only "Use ours / theirs".
- [ ] **6. Search in History.** By author, message body, date and file.
- [ ] **7. One window.** `onaygit .` should open a tab in the running app, not a second app.
- [ ] **8. Loading and density.** Spinner while a repository loads. Rail labels option. Compact list mode. Full date on hover in Graph.

## Release and code health

- [ ] **9. Ship it.** Notarize (needs Apple Developer ID), publish a GitHub Release with the zip, fill the Releases link, add an update check.
- [ ] **10. Clean structure.** Error enum for network and auth. `platform` module (Keychain, `osascript`, `open`) for Linux and Windows. Split `settings_page.rs`. Group `App` fields. Add UI state tests.

## Later

Interactive rebase list, submodules and LFS, commit signing, hook output, notifications, localization, undo for discard.
