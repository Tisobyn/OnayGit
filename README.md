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
  keybinds.rs        shortcut list, saved overrides, key capture
  app/
    mod.rs           App state, enums and the update loop
    repos.rs         open/add/remove repos, refresh, stage, background jobs
    staging.rs       line/hunk staging, compare, ignore actions
    refs.rs          stash, tag and worktree actions
    history.rs       commit actions, rewrite, conflicts
    tabs.rs          tabs and per-repository color/icon
    spaces.rs        spaces (repository groups)
    tasks.rs         background tasks (network, AI, ssh) and their results
    accounts.rs      GitHub/GitLab accounts (tokens in the Keychain)
    hosting.rs       publish repository, pull requests
    ai.rs            AI commit messages
    machines.rs      SSH machines
    editors.rs       external editors and per-file-type rules
    ignored.rs       ignored files and their local snapshots
  git/               thin wrappers over the git command (no UI code)
    exec.rs          run git, with or without stdin
    status.rs        status, stage, unstage, discard, file diff
    commit.rs        commit, amend, undo, log
    branch.rs        branches, checkout, merge, ahead/behind, compare
    diff.rs          parse unified diffs into numbered lines
    patch.rs         build and apply partial patches (has tests)
    ignore.rs        .gitignore helpers
    stash.rs         stash list, push, apply, pop, drop
    remote.rs        remotes (has tests)
    tag.rs           tags
    worktree.rs      worktrees (has tests)
    graph.rs         commit graph lanes
    rewrite.rs       reword, squash, drop (non-interactive rebase)
    conflict.rs      merge/rebase state, conflict resolution (has tests)
    repo.rs          init, scan for repositories
    config.rs        git config
    external.rs      open in editor, show in Finder
    images.rs        old version of image files for the diff
    ssh.rs           repositories on other machines (ssh://host/path)
    http.rs          small HTTP client over curl
    keychain.rs      secrets in the macOS Keychain
    hosting.rs       GitHub/GitLab API (has tests)
    ai.rs            AI providers for commit messages (has tests)
  ui/                drawing code, one file per screen area
    theme.rs         5 themes and accent colors
    widgets.rs       small reusable widgets
    diff_view.rs     diff renderer with line selection
    home.rs          start screen (no repo open)
    settings_page.rs settings screens
    toolbar.rs       top bar: repo, branch, sync
    sidebar.rs       Changes and History tabs
    main_panel.rs    diff of the selected file or commit
    dialogs.rs       confirm dialogs and the error bar
    stash.rs         stashed changes list in the Changes tab
    manage.rs        Tags, Worktrees and Reword windows
    history.rs       History tab: commit list, files, context menu
    graph.rs         draws one graph row
    conflicts.rs     conflict banner
    icons.rs         Phosphor icons from 1git (assets/icons)
    tabs.rs          tab bar and Cmd+T / Cmd+W
    settings_repo.rs Remotes and repository appearance pages
    tips.rs          welcome window and tip boxes
    tour.rs          guided tour (three tips next to real controls)
    clone.rs         full-page Clone (account repositories or URL)
    syntax.rs        small syntax highlighter for diffs
    shell.rs         window frame: title bar tabs, left rail, rounded card
    switcher.rs      Cmd+P repository switcher
    accounts_page.rs Accounts and AI settings pages
    publish.rs       Publish, Pull requests, Add machine, machine view
assets/              app icon (from 1git)
PORTING.md           list of 1git features and what is done
```

Rule of thumb: git commands live in `git/`, drawing lives in `ui/`, state lives in `app/`.
