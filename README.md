# OnayGit

A simple Git client in Rust with a GitHub Desktop look. Built with egui and the git command line.

```bash
cargo run --release
```

## Project layout

```
src/
  main.rs            window setup and icon
  prelude.rs         common imports used by every module
  settings.rs        saved settings and repository list (files in ~/)
  app/
    mod.rs           App state, enums and the update loop
    repos.rs         open/add/remove repos, refresh, stage, background jobs
    staging.rs       line/hunk staging, compare, ignore actions
    refs.rs          stash, tag and worktree actions
  git/               thin wrappers over the git command (no UI code)
    exec.rs          run git, with or without stdin
    status.rs        status, stage, unstage, discard, file diff
    commit.rs        commit, amend, undo, log
    branch.rs        branches, checkout, merge, ahead/behind, compare
    diff.rs          parse unified diffs into numbered lines
    patch.rs         build and apply partial patches (has tests)
    ignore.rs        .gitignore helpers
    stash.rs         stash list, push, apply, pop, drop
    tag.rs           tags
    worktree.rs      worktrees (has tests)
    repo.rs          init, scan for repositories
    config.rs        git config
    external.rs      open in editor, show in Finder
  ui/                drawing code, one file per screen area
    theme.rs         colors and light/dark
    widgets.rs       small reusable widgets
    diff_view.rs     diff renderer with line selection
    home.rs          start screen (no repo open)
    settings_page.rs settings screens
    toolbar.rs       top bar: repo, branch, sync
    sidebar.rs       Changes and History tabs
    main_panel.rs    diff of the selected file or commit
    dialogs.rs       confirm dialogs and the error bar
    stash.rs         stashed changes list in the Changes tab
    manage.rs        Tags and Worktrees windows
assets/              app icon (from 1git)
PORTING.md           list of 1git features and what is done
```

Rule of thumb: git commands live in `git/`, drawing lives in `ui/`, state lives in `app/`.
