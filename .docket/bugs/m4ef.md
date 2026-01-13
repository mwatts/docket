---
id: m4ef
title: Add event log (JSONL) for history tracking
status: draft
priority: medium
created: 2026-01-13T02:53:11.186516246Z
---

## Goal

Replace mutable markdown files with append-only JSONL event logs for conflict-free merging across workspaces.

## Acceptance Criteria

- [ ] Create event types: created, status_changed, updated, change_linked
- [ ] Store events in `.docket/bugs/<id>.jsonl` (one file per bug)
- [ ] Each event has: id, bug_id, timestamp, type, actor, data
- [ ] Current state derived by replaying events
- [ ] Existing markdown bugs migrated on first access
- [ ] `docket log <id>` shows event history

## Context

Currently bugs are mutable markdown files. If two workspaces modify the same bug, we get merge conflicts.

With event logs:
- Each workspace appends events to the bug's log file
- Merging = union of events, sorted by timestamp
- No conflicts as long as events have unique IDs

Format:
```jsonl
{"id":"evt-x1","bug":"a7f3","ts":"2026-01-13T...","type":"created","data":{...}}
{"id":"evt-x2","bug":"a7f3","ts":"2026-01-13T...","type":"status_changed","data":{"from":"draft","to":"approved"}}
```

This is a prerequisite for the SQLite cache - we cache the derived state.

## Log

