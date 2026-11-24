# OnayGit review

Date: 2026-11-25. Version 0.1.0. Written after the Round 5 tests on the real Test repository.

Three parts:

1. [Code review](#1-code-review)
2. [UI and UX review](#2-ui-and-ux-review)
3. [App and feature review](#3-app-and-feature-review)

Then a [single to-do list](#to-do-list-in-order) at the end.

**Short verdict.** The app is small, fast and tidy. Daily git work is solid. The biggest risks are speed on big repositories, no automated UI tests, and a few features people expect from a Git client that are not there yet.

| Area | Score | One line |
|---|---|---|
| Code | 7 / 10 | Clean layers, no panics in app code. Heavy work runs on the UI thread. |
| UI and UX | 8 / 10 | Consistent, calm, good empty states. Some density and discoverability gaps. |
| Features | 7 / 10 | Covers the daily flow well. Missing conflict editor, blame, file history, paging. |
| Size and speed | 9 / 10 | 7 MB app, 3 MB zip. 1git is 114 MB. |

---

## 1. Code review

### Numbers

- 73 Rust files, about 12,600 lines.
- 27 unit tests. All are in the `git/` layer. None cover `app/` or `ui/`.
- Build: zero compiler warnings. `cargo clippy` gives 22 style hints, none serious.
- No `unwrap()` or `expect()` in app code (only in tests).
- Release binary 7.1 MB, stripped, LTO on.

### What is good

- **Clear layers.** `git/` runs git and has no UI code. `app/` holds state and actions. `ui/` only draws. The README says this and the code follows it.
- **UI actions are collected, then applied.** Each screen fills small local variables (`open`, `remove`, `toggle`) and applies them after drawing. This avoids borrow problems and makes the flow easy to read.
- **Real git underneath.** Results always match the terminal. Easy to debug.
- **Safe by default.** Discard asks first. Tokens are in the Keychain and now go through stdin, not the command line. Slicing of text uses char boundaries.
- **Background tasks.** Network work (clone, push, AI, avatars, PR list) runs on threads with a result channel and a visible "busy" state.
- **Good unit tests** for the tricky parts: partial patches, rewrite, conflicts, remotes, worktrees, hosting parsers.
- **Small dependency list.** Only eframe, egui_extras, rfd, serde_json and image.

### Problems, by priority

**High**

1. **`refresh()` runs on the UI thread** (`src/app/repos.rs`, called from `src/app/mod.rs` every 1 to 30 seconds and after almost every action). It runs status, log (300 commits), branches, remote branches, stashes, tags, worktrees, upstream, graph and the diff, one after another. On a big repository or a slow disk this freezes the window for a moment each time.
   *Fix:* run it on a worker thread and swap the result in, or split it: status and diff first, the rest only when their tab is visible.
2. **History is capped at 300 commits** (`git/commit.rs`, `--max-count=300`) with no paging. Older commits cannot be reached. The Graph tab has the same cap.
   *Fix:* load in pages as the list scrolls.
3. **No size guard on diffs.** A huge file or a generated file is parsed and drawn in full. Only the AI prompt is trimmed (`MAX_DIFF`).
   *Fix:* cut at a line limit and show "Diff too large. Show anyway".

**Medium**

4. **No tests for `app/` and `ui/`.** The bugs found in testing (filter focus, keybind chip, PR list not reloading, clone folder) were all in these layers. Add a few state tests (tabs, spaces, sync info, move_tab) and a screenshot smoke test script.
5. **Big files.** `ui/settings_page.rs` is 1,491 lines and holds every settings page. `app/mod.rs` (772) holds the whole `App` struct with about 150 fields. Split settings pages into one file each. Group related fields into small structs (for example `CloneState`, `PrState`, `GraphState`).
6. **macOS only, but not marked.** Keychain (`security`), `osascript`, `open -R` are called directly in `git/keychain.rs`, `ui/settings_page.rs`, `git/external.rs`. Fine for now. Put them behind one small `platform` module so Linux or Windows can be added later.
7. **Shell strings for the editor** (`git/external.rs`, `ui/main_panel.rs`). The path is quoted, the editor command is the user's own setting, so this is acceptable. Note it in a comment and keep it.
8. **Polling instead of watching.** The app polls on a timer. A file watcher (`notify` crate) would refresh only on real changes and save battery.
9. **Errors are plain strings.** `Result<_, String>` everywhere. Works, but you cannot tell "no network" from "bad token" without reading text. Consider a small error enum for the network and auth cases.

**Low**

10. Clippy hints: 13 needless borrows, 2 `?` rewrites, 1 long argument list (8 args), 1 complex type. Run `cargo clippy --fix`.
11. `PORTING.md` is a to-do file from the porting stage. Move it to `docs/` or remove it once all items are done.
12. Public items that are only used once (`Confirm::DeleteBranch` is used, others) are fine. No dead code left after the clean-up.
13. `Cargo.lock` is committed (good for an app). No CI yet.
14. The `rust-objcopy` warning during release builds is a toolchain problem on this Mac, not the project. It does not change the result.

### Security notes

- Tokens: Keychain only. Fixed: no longer visible in the process list.
- Network calls use `curl` with the token in an environment variable or header, not in the URL.
- The AI feature sends the staged diff to the provider you choose. The Privacy page says so. Keep it opt-in.
- SSH machines run `git` over `ssh`. The host and path are quoted. Good.
- The app is signed ad hoc. It is not notarized, so other Macs will show a warning on first open.

### Quick wins (under one hour each)

- `cargo clippy --fix`.
- Line limit for large diffs.
- Move the platform calls into one module.
- Add `.github/workflows/ci.yml` with `cargo build`, `cargo test`, `cargo clippy`.

---

## 2. UI and UX review

### Overall

The look is calm and consistent: one dark theme family, one accent, readable type, equal icon weight. Empty states have a title and a hint. The side rail, the toolbar and the dropdowns now follow one pattern.

### What works well

- **Side rail.** One sync button that changes meaning (fetch, pull, push, publish) with a small badge for ahead and behind. Stashes show a count. The order is logical: remote actions on top, views below.
- **Toolbar.** Current Repository, Current Branch and Worktrees are three equal cells. The dropdown panels have a filter, groups, times and a clear bottom action.
- **History.** Three columns: commit list, changed files, diff. Profile pictures and co-authors make commits easy to scan.
- **Graph tab.** Branches and tags on the left, the whole graph on the right, chips for HEAD and branches.
- **Settings.** Grouped sidebar, one page per topic, short explanations (Remotes).
- **Themes.** Dark, Light, Midnight, Nord and Sand all checked. No contrast problems found.
- **Keyboard.** Cmd+T, Cmd+W, Cmd+1..9, Cmd+P, Cmd+F, Cmd+, and arrow keys in History.
- **Safe actions.** Discard and delete branch ask first. Errors appear in a red bar with Dismiss.

### Problems, by priority

**High**

1. **Rail icons have no labels.** Seven icons in a row, some similar (Changes and Graph, sync states). New users must hover to learn them. Show a tooltip with the shortcut, and add a first-run hint. A "show labels" option would help most.
2. **No loading state for the first view.** After opening a big repository the window just waits. Show a small spinner or skeleton rows while status and history load.
3. **Sync button meaning changes.** Good idea, but the icon changes too. People may not notice it changed from fetch to push. The badge helps. Add the action name to the tooltip (already there) and briefly flash the badge after a fetch finds news.

**Medium**

4. **Density.** The Changes list is spacious. Power users want more rows on screen. Add a compact mode in Appearance.
5. **Tip card.** It is helpful the first time, but it stays in the empty Changes view every time. Hide it after the user stages one file, or add "Don't show again".
6. **Stash UI.** Stashes live as a folded row inside Changes. The rail Stashes button opens Changes and expands it. A dedicated list would be clearer. At least scroll to the row and highlight it.
7. **Dropdown "Add" menu** inside the repository panel is small. Use the same button style as New Branch.
8. **Graph rows show relative time only** ("7 hours ago"). Show the full date on hover.
9. **Tab drag has no drop marker.** The tabs swap live, which is good. A faint insertion line would make it clearer. Dragging a tab out to close it is not supported.
10. **Pull Requests tab in the branch dropdown** only has a button that opens the dialog. Show the list inline.
11. **Settings widths differ** between pages (about 480 to 760 px). This is on purpose, but Keybinds and Git feel unrelated. Pick two widths.

**Low**

12. The Home page shows long temporary paths in full. Middle-truncate paths (`/Users/…/repo`).
13. Home avatars: GitHub owners only. GitLab and other hosts show a letter.
14. Light theme is correct but the least polished (thin borders on cards).
15. Window minimum size is not enforced. Below about 700 px wide the history columns get tight.
16. No visible focus ring when moving with the keyboard in lists.
17. No drag and drop of a folder onto the window to open a repository.

### Accessibility

- egui draws its own widgets, so VoiceOver support is limited. Not fixable quickly.
- Text sizes are small (11 to 13 px). There is a text size setting. Offer larger default sizes.
- Colour is not the only signal for status (letters and icons are used). Good.

### First-run experience

- Welcome window and a three-step guided tour exist and work.
- Missing: a "Open a repository" drop target and a sample "no account yet" path on Home. Clone without an account should be the first option, not the second.

---

## 3. App and feature review

### What the app is

A native macOS Git client for people who like GitHub Desktop's simplicity but want more power: tabs, spaces, graph, worktrees, line staging, SSH machines. About 7 MB.

### Feature map

**Works and tested**

- Repositories: open, create, clone (URL and account), scan a folder, remove from list, tabs, spaces, SSH machines.
- Changes: file list and tree, diff with word highlight and syntax colours, stage file, hunk and single lines, commit, amend, undo last commit, discard, ignore file, folder or extension.
- Co-authors: choose people, trailers added, pictures in history.
- History: commit list, details, changed files column, tag, cherry-pick, revert, reset, reword, squash, drop, move.
- Branches: create, switch, merge, delete, compare with Incoming and Outgoing, rebase.
- Remotes: fetch, pull, push, publish, remotes page, GitHub and GitLab accounts, pull request list and create.
- Extras: stashes, tags, worktrees, ignored-file snapshots, AI commit messages, external editor rules, keybinds, five themes, About page.
- Delivery: command-line launcher (per user), packaging script, MIT license, README with screenshots.

**Not tested here**

- "All users" launcher install (needs an administrator password).
- GitLab sign-in and clone (no GitLab account used).
- Browser (device flow) sign-in end to end.
- Rewrite actions on a repository with real remote history.
- Very large repositories (10k+ files, 100k+ commits).

### Missing features

Things a user of a Git client will ask for, from most to least asked:

1. **Merge conflict editor.** Today you can pick "Use ours", "Use theirs" or "Mark resolved" per file. There is no side-by-side or inline editor for single conflicts.
2. **File history and blame.** Right-click a file to see its commits, and show who changed each line.
3. **Search in History** by author, message body, date or file. Only the subject filter exists.
4. **Paging in History and Graph** (see code issue 2).
5. **Interactive rebase** with a visible todo list. Reword, squash, drop and move exist as separate actions.
6. **Submodules and Git LFS** status.
7. **Commit signing** (GPG or SSH) setting.
8. **Hooks** visibility: show when a pre-commit hook fails and why.
9. **Auto update** and a real GitHub Release. The About page links to Releases, which is still empty.
10. **Notarization** for sharing the app with others.
11. **Single window handling.** Running `onaygit .` while the app is open starts a second app instead of opening a tab in the first.
12. **Drag and drop** a folder onto the window or Dock icon to open it.
13. **Notifications** when a long push, pull or clone ends in the background.
14. **Linux and Windows.** Needs the platform module from code issue 6.
15. **Localization.** English only.

### Strengths compared with GitHub Desktop and 1git

- Much smaller (7 MB against 114 MB for 1git).
- Spaces with their own tabs, worktrees in the toolbar, SSH machines.
- Line-level staging and a graph tab.
- Local history for ignored files (nice for `.env`).
- AI commit message with your own provider or CLI.

### Risks

- **Trust.** A Git client touches people's work. Discard and hard reset are guarded, but there is no "undo discard". Consider keeping discarded changes as a hidden stash for a few days.
- **Performance** on large repositories (code issues 1 to 3).
- **Distribution.** Unsigned apps trigger Gatekeeper warnings. Without notarization, adoption stays low.
- **Single maintainer.** Documentation and tests keep the project understandable. Add CI.

---

## To-do list (in order)

**Status after the follow-up round:** done = 1, 2, 3, 4, 5, 7, 8, 9 (drop a folder), 12, 16. Not done: 6, 9 (`onaygit .` into the running app), 10, 11, 13, 14, 15, 17 to 20. Items 13 to 15 need an Apple Developer ID and a decision to publish a release.

**Next (stability and speed)**
1. Move `refresh()` off the UI thread, or split it by visible tab.
2. Page the History and Graph lists.
3. Add a diff size limit with "Show anyway".
4. Add CI: build, test, clippy.
5. Run `cargo clippy --fix`.

**Then (daily comfort)**
6. Conflict editor.
7. File history and blame.
8. History search (author, body, file).
9. Open a folder by drag and drop, and `onaygit .` into the running app.
10. Spinner while a repository loads.
11. Rail labels option and compact list mode.
12. Inline pull request list in the branch dropdown.

**Then (release)**
13. Notarize the app (needs an Apple Developer ID).
14. Publish a GitHub Release with the zip and fill the Releases link.
15. Add auto update check.
16. Write a short CHANGELOG.

**Later**
17. Platform module and Linux or Windows builds.
18. Small error enum for network and auth.
19. Split `settings_page.rs` and group `App` fields.
20. Localization.
