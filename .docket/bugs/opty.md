---
id: opty
title: docket done should run jj new if needed
status: draft
priority: medium
created: 2026-01-13T04:02:43.238471694Z
---

## Goal

After `docket done` completes, leave the user on a fresh jj change so they're ready for the next task.

## Acceptance Criteria

- [ ] Check if current change has modifications (not empty)
- [ ] If current change has content, run `jj new` to create fresh change
- [ ] If current change is already empty, skip `jj new`
- [ ] Print message indicating new change was created (or skipped)

## Context

When finishing work with `docket done`, the user is typically on the change that implemented the bug. To start fresh work, they need to run `jj new` manually. This is easy to forget.

The check is important because if the user is already on an empty change, running `jj new` would create an unnecessary empty commit in the history.

Can check if change is empty with:
```
jj log -r @ -T 'if(empty, "empty", "has changes")'
```

## Log

