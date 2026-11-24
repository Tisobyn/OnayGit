# OnayGit audit

Date: 2026-11-25. I used the release build, with a demo repo and the Test repo.

## Fixed in this round

- **Freezes.** The timer check (status, branch, ahead/behind) now runs on a thread. Slow disks or SSH hosts no longer stop the window. Commit and amend also run on a thread, so a slow hook cannot freeze the UI. Tested: a hook that sleeps 6 s and fails. The window kept drawing and showed "Committing failed: hook says no".
- **Hanging prompts.** Git runs with `GIT_TERMINAL_PROMPT=0`, so it never waits for a password in the dark. `GIT_OPTIONAL_LOCKS=0` stops read-only commands from fighting with your own git use.
- **Tips and tour removed.** Tip cards, "Next tip", the guided tour and their settings are gone. The first-run Welcome window stays (it is not a tip).
- **Graph bug.** Stash internals ("WIP on…", "index on…") showed as commits. Now hidden.
- **Release build warning.** `rust-objcopy` failed on this Mac. Strip is off in Cargo and the package script strips the binary itself.
- **Welcome text** said sync is in the top bar. It is the top rail button.

## Checked by using the app

| Area | Result |
|---|---|
| Welcome window, Get started | OK |
| Changes: select file, check box, commit (async) | OK |
| Failed and slow commit hook | OK, error bar |
| Rail: Changes, History, Stashes, Graph, Home, Settings | OK |
| Graph after fix: 15 commits, chips, lanes | OK |
| Home list view, spaces, quick actions | OK |
| Settings: Application page, launcher status | OK |
| Tabs: switch, Home tab | OK |
| 29 unit tests | pass |
| Build | zero warnings |

## Still open (also in [TODO.md](TODO.md))

1. `refresh()` after actions still runs on the UI thread (checkout, merge, select commit, graph load). Fine on small repos. Slow on huge ones.
2. No timeout on `git`. A hung fetch only stops its own task, but it cannot be cancelled.
3. No busy hint while a commit runs. The button stays active. A second click is ignored.
4. The red error bar stays when you change screens. It covers the bottom rail buttons.
5. The Stashes rail button does nothing visible when Changes is already open.
6. Error text says "Committing failed". "Commit failed" reads better.
7. Settings page "Auto refresh" slider shows 3 s but the slider knob sits at the far left. Easy to misread.
8. The demo repo path on Home is long. Middle-truncate it.

## Not tested

Notarization, "All users" launcher install, GitLab sign-in, repositories with 100k commits, Linux and Windows.
