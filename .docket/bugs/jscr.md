---
id: jscr
title: Add docket cleanup command to remove workspace
status: draft
priority: medium
created: 2026-01-13T04:27:42.132759126Z
---

## Goal

Add `docket cleanup <bug-id>` command to clean up a workspace after work is complete.

## Acceptance Criteria

- [ ] Add `docket cleanup <bug-id>` command
- [ ] Run `jj workspace forget ws-<bug-id>` from the main repo
- [ ] Remove the workspace directory `../ws-<bug-id>`
- [ ] Handle case where workspace doesn't exist gracefully
- [ ] Warn if there are uncommitted changes in the workspace

## Context

After completing work with `docket done`, the jj workspace and directory remain. Currently cleanup requires manual steps:

```bash
cd ../trunk
jj workspace forget ws-<bug-id>
rm -rf ../ws-<bug-id>
```

This command automates that cleanup. Should be safe to run - warn if there are uncommitted changes that would be lost.

## Log

