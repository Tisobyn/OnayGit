# OnayGit test report

Manual GUI pass on a throwaway repo and an isolated HOME. Driven with real mouse and key events. The `~/Developer/Test` repo was not touched.

**Result:** most things work. 3 real bugs, 4 small issues, a few areas not covered.

## Works

**Home**
- Grid/list toggle, Add machine dialog, bad SSH host shows an error.
- Create repo popover, open repo from list, dismiss error bar.
- New space (name, icon, colour). Spaces keep their own tabs.

**Rail and toolbar**
- Spaces menu (list, check mark, New space, Manage spaces).
- Tags, Worktrees, Pull requests dialogs. PR dialog shows a hint with no account.
- Current Repository menu, Branch menu (local/remote, merge, delete, new branch).
- Create branch, Publish branch / Push, Fetch button.
- Stash, Apply, Pop, Drop.
- Ignored tab lists ignored files.
- Settings gear.

**Changes**
- File list, diff, stage/unstage file, stage/unstage hunk, select all.
- Commit, Undo last commit, Amend.
- File context menu: Discard (with confirm), Ignore file/folder/ext, Copy path.
- Tree view toggle, word-diff highlight.
- Filter by clicking the field, and with Cmd+F (focuses the field).

**History**
- Commit list, detail, diff, changed files.
- Commit menu: tag, cherry-pick, revert, reset, rewrite, copy hash.
- Tag create and delete, invalid tag name error.

**Settings**
- Application: Only-me install, Welcome window.
- Spaces: list and create.
- Appearance: themes, grain, accent, font, text reset.
- Keybinds: list and search render.
- Git: identity edit and save, auto-include toggle.
- External editor: default, add rule, save, edit, delete.
- AI: add connection, provider tiles, custom command, save, edit, remove.
- AI sparkle fills the commit message end to end.

**Shortcuts**
- Cmd+P switcher opens and takes text.
- Cmd+F focuses the Changes filter.

## Bugs

1. **Filter icon does not focus the field.** Clicking the search icon in Changes opens the field but typing goes nowhere. Fix: set `focus_filter = true` on toggle in `src/ui/sidebar.rs` (~line 65). Cmd+F already works.
2. **Keybind chip click does not record.** "Press shortcut..." never appears, so shortcuts cannot be changed. Look at `src/ui/settings_page.rs` ~line 590 (`ui.scope(...).response.interact(Sense::click())`). The inner chip likely eats the click.
3. **Home sidebar shows a generic folder icon for spaces.** It ignores the chosen icon and colour. See `src/ui/home.rs`.

## Small issues

- SSH failure label reads "Test machine:host failed". Rewording needed.
- Empty space shows a blank page with no hint.
- Worktrees dialog: Open button overlaps the path text.
- Rail sync button gives no visible feedback (only tried with a local-only remote).

## Not tested

- Native file pickers (Create repo, Open, Clone folder).
- Clone page with a real GitHub/GitLab account (needs network and a token).
- Pull request create, Push and create.
- "All users" install (needs admin).
- Auto refresh slider, guided tips toggle.
- Guided tour on first run.
- Help page, Remotes page, repo Appearance, Commit identity per repo.
- Other shortcuts: tab switch, space switch, expand diff, commit, refresh.
- Tab + / close, tab reordering.
- Cherry-pick, revert, reset, rewrite results (menu only checked).
- Accounts sign-in forms for GitHub/GitLab.
- Light and Midnight theme visual pass on every page.

## Notes

- The up-arrow on drop-downs that open upward is egui's own cue, not a bug.
- During the test a push once went to the real OnayGit remote. The stray `test-br` branch was deleted. The demo repo now points at a local bare remote.

---

# UI / UX review

Looked at Home, Changes, History, Ignored, dialogs, rail menus, and the Settings pages (Help, Remotes, Appearance, Commit identity, Ignored files, Accounts, Keybinds). Overall the look is clean and consistent: dark theme, one accent, readable type, good spacing, icons the same stroke weight.

## Good
- Rail, tab bar, toolbar and dialogs line up and share one style.
- Dialogs (Tags, Worktrees, Pull requests) are clear, with a dimmed backdrop.
- Settings sidebar groups (Application / Repository) are easy to scan.
- Empty states have a title and a short hint (Ignored, History).
- Repo appearance colour and icon pickers show the selected item well.

## Problems (fix list)
1. **Rail Ignored icon is a plain square.** Looks like a placeholder. Same icon is used in Settings > Ignored files. Use an eye-off or file-minus icon.
2. **Spaces use a generic folder icon** in the Home sidebar and in the rail Spaces menu. Show each space's chosen icon and colour.
3. **Tip card in the wrong place.** "Click the checkbox next to a file..." shows on History ("Select a commit") and Ignored ("Select a file"). It is a Changes tip. Show it only on Changes, or pick tips for the page.
4. **Dialogs do not close with Escape.** Pull requests, Worktrees and Tags stay open. Only the x works. Add Escape and click-outside.
5. **Low-contrast inputs.** Remotes (name, URL) and Commit identity fields have almost no border or fill. Filled text ("Test User") looks like a placeholder. Add a visible border/fill and brighter text.
6. **Worktrees dialog:** Open button overlaps the long path. Wrap the path or truncate with an ellipsis.
7. **History filter** shows bare text "zz" with no field box after the filter is used. Give it the same framed field as Changes.
8. **Settings pages have different widths** (Accounts about 380px, Remotes 450px, Keybinds 600px). Use one content width.
9. **"New space" field on Home** is tiny and has no border. Make it a normal field, or use a + button that opens the New space panel.
10. **Disabled buttons look too faint** (Add remote, Choose folder and add..., Commit 0 files). Still readable, but hard to tell them from labels.
11. **Keybinds:** the "Open Settings" row shows plain text, not a chip, unlike the others. Also the chip click bug above.
12. **Long path on Home list** is cut with no ellipsis and no tooltip. Show the full path on hover.

## Images
- App icon and tab icons render sharp. Not checked at all sizes (Dock, Cmd+Tab, title bar); that needs the built .app.
- No raster images in the UI. All icons are vector glyphs.

## Not reviewed
- Light and Midnight themes on every page.
- Grid view on Home with many repos.
- Narrow window sizes and very large window.
- Guided tour card.

---

# Fix status

Fixed and re-checked in the app:
- Bug 1: filter icon now focuses the field.
- Bug 2: keybind chips record again (labels no longer swallow the click; labels are non-selectable app-wide).
- Bug 3 / UI 2: spaces show their own icon and colour on Home and in the rail menu.
- UI 1: Ignored icon now matches 1git (document with a check).
- UI 3: tip card shows only on Changes.
- UI 5, 7, 9: text fields have a visible border (Remotes, Commit identity, History filter, New space, Git, Worktrees).
- UI 6: Worktrees path wraps, no overlap.
- UI 10: disabled primary buttons are less faint.
- UI 11: "Open Settings" shortcut is a chip.
- UI 12: full path shows on hover in the Home list.
- Small: SSH error now reads "Could not connect to <host>: ...". Empty space shows a hint.

Not changed:
- UI 4: Escape already closes dialogs. The earlier failure was the test tool, not the app.
- UI 8: settings page widths differ on purpose (they follow 1git).
- Rail sync button feedback: not changed, needs a real remote to judge.
