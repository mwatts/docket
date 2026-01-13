---
id: pbvv
title: Add SQLite cache for fast queries
status: draft
priority: medium
created: 2026-01-13T02:53:12.057486938Z
---

## Goal

Add a SQLite cache at `.docket/.cache/state.db` for fast queries, reconstructable from event logs.

## Acceptance Criteria

- [ ] Create SQLite database schema for bugs table
- [ ] Add `.docket/.cache/` to `.docket/.gitignore`
- [ ] Implement cache rebuild from event logs
- [ ] `docket list` reads from cache instead of parsing all files
- [ ] Cache invalidation when event logs are newer than cache
- [ ] `docket rebuild` command to force cache rebuild

## Context

For small numbers of bugs, globbing and parsing markdown is fine. But as bug count grows, a cache becomes necessary.

The cache is:
- Not committed (in .gitignore)
- Reconstructable from event logs
- Invalidated when logs change

This depends on the event log feature (m4ef) being implemented first.

## Log

