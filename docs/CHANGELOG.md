# Changelog

## 0.1.0 (unreleased)

First public version.

- Changes: file list and tree, diff with word highlight, stage by file, hunk and line, commit, amend, undo, discard, ignore.
- History: commit list with pictures and co-authors, changed files column, tag, cherry-pick, revert, reset, reword, squash, drop, move. Load 300 more commits at a time. File history and blame.
- Graph tab: all branches and tags with the whole commit graph.
- Side rail: one sync button with ahead and behind badge, pull requests, stashes.
- Toolbar: repository, branch and worktrees cells. Repository and branch dropdown panels, with pull requests listed inline.
- Tabs (drag to reorder), spaces, SSH machines, drop a folder on the window to open it.
- GitHub and GitLab accounts, publish, pull requests.
- AI commit messages, external editor rules, keybinds, five themes.
- Faster: the timer refresh only reads status and the open diff. Long diffs are cut at 4000 lines until you ask for all.
- Conflict editor: choose ours, theirs, both or edit each conflict, then save and mark resolved.
- History search: press Enter in the filter to search all history by words, `author:`, `file:`, `since:`, `until:`. Full date on hover.
- File watcher: the window updates when files change. The timer is only a safety net.
- One window: `onaygit .` opens the folder in the running app.
- Loading overlay when a repository opens, spinner on the sync button, optional labels under rail icons, compact lists.
- Fixes: New Branch works with Enter and shows errors in the panel. Graph hides stash commits. Commit runs on a thread.
- Packaging script for `OnayGit.app`. MIT license.
