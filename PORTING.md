# Porting 1git features to OnayGit

Source: `~/Developer/1git-master` (`packages/git`, `apps/gpuix/src/feature`).
Goal: same features, GitHub Desktop look, more compact. Rust + egui + git CLI.

Status: `[x]` done, `[ ]` todo. Port in order, one by one.

## Core Git
- [x] 1. Discard changes (one file, all files, with confirm)
- [x] 2. Amend last commit and undo last commit
- [x] 3. Merge into current branch, delete branch, remote branches
- [x] 4. Branch compare (ahead/behind against another branch)
- [x] 5. Stage by hunk
- [x] 6. Stage by line
- [x] 7. Ignore file actions (add to .gitignore), ignored file history

## Stash, tags, worktrees
- [x] 8. Stash: list, create, apply, pop, drop
- [x] 9. Tags: list, create, delete, push
- [x] 10. Worktrees: list, add, remove, switch

## History
- [x] 11. Per-commit file list in History
- [x] 12. Commit graph
- [x] 13. Cherry-pick, revert, reset to commit
- [x] 14. Rewrite history (reword, squash, drop)
- [x] 15. Sync and conflict review

## App shell
- [x] 16. Tabs (several repositories in one window)
- [x] 17. Remotes settings (list, add, remove, rename)
- [x] 18. Repository color and icon, themes
- [x] 19. Guided tips and onboarding

## Hosting and AI
- [x] 20. GitHub and GitLab accounts
- [x] 21. Publish repository
- [x] 22. Pull / merge requests
- [x] 23. AI commit messages (Claude Code, Codex, any API)
- [x] 24. SSH remote repositories (machines)

## Round 2 (gaps found against 1git)
- [x] 25. Changes tree view (folders, folder checkbox)
- [x] 26. Word-level highlights in the diff
- [x] 27. Sync review: incoming / outgoing, Merge, Rebase
- [x] 28. Image diff (before / after)
- [x] 29. Quick repository switcher and shortcuts
- [x] 30. Move commits in history (newer / older)
- [x] 31. Spaces (groups of repositories)
- [ ] Not done: expand context lines in the diff, per-space keybind settings
