---
id: qhus
title: docket done should auto-snapshot and generate commit message
status: done
priority: high
created: 2026-01-13T05:20:45.278271727Z
---

## Goal

When run from a workspace, `docket done` should handle all the finishing work - snapshot changes, generate a good commit message, and mark the bug complete.

## Acceptance Criteria

- [x] Detect if running from a workspace (check for `.docket/current.json` or `ws-*` directory pattern)
- [x] Run `jj` to snapshot any uncommitted changes
- [x] Generate commit message from bug: "Implement <title> (<id>)"
- [x] Set the commit description with `jj describe`
- [x] Link the change to the bug (call `docket link` internally)
- [x] Mark bug as done + sync acceptance criteria (existing behavior)
- [x] Skip the "create fresh change" step when in a workspace

## Context

Currently after implementing a bug, users must manually:
1. Run jj to snapshot changes
2. Write a commit message with `jj describe`
3. Run `docket done`
4. Squash any empty changes

This is tedious. `docket done` should be a one-stop "I'm finished" command that handles all of this when run from the workspace.

The commit message format could be:
```
Implement <title> (<id>)
```

Or we could look at the Log section and include recent entries.

## Log

- 2026-01-12: Implemented workspace detection via detect_workspace() function that checks for .docket/current.json and ws-* directory pattern
- 2026-01-12: Added jj_snapshot() to capture uncommitted changes before completion
- 2026-01-12: Added jj_describe() to set commit message in format "Implement <title> (<id>)"
- 2026-01-12: Integrated change linking directly into done() by calling bug.add_change() with current change ID
- 2026-01-12: Modified done() to skip create_fresh_change_if_needed() when running from workspace
