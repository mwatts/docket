---
id: afan
title: docket work runs claude in current directory instead of workspace
status: done
priority: high
created: 2026-01-13T05:40:06.226470646Z
---

## Goal

When `docket work <id>` launches claude, it should run in the workspace directory, not the current directory.

## Acceptance Criteria

- [x] Verify `std::env::set_current_dir()` is called before `exec()`
- [x] Ensure claude sees the workspace as its working directory
- [x] Claude should find `.docket/current.json` in the workspace
- [x] Any file changes claude makes should be in the workspace, not trunk

## Context

Currently `docket work` does:
1. Creates workspace at `../ws-<id>`
2. Calls `std::env::set_current_dir(&workspace_path)`
3. Writes `.docket/current.json`
4. Execs `claude`

But when claude runs, it appears to operate in the original directory (trunk) rather than the workspace. This causes:
- Implementation changes go to trunk instead of the workspace
- The workspace only gets metadata updates
- The isolated workspace model breaks down

Discovered when implementing qhus - the status.rs changes ended up in trunk while ws-qhus only had bug file changes.

Need to verify the `set_current_dir` + `exec` interaction is working correctly, or find an alternative approach.

## Log

- 2026-01-13: Fixed by adding `.current_dir(&workspace_path)` to the Command before exec()
