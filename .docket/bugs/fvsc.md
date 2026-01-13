---
id: fvsc
title: Create docket:implement Claude Code skill
status: approved
priority: high
created: 2026-01-13T02:53:21.622112628Z
---

## Goal

Create a Claude Code skill that reads the current bug context and guides implementation work.

## Acceptance Criteria

- [ ] Skill file created at `.claude/commands/docket:implement.md`
- [ ] Skill reads `.docket/current.json` to understand the bug
- [ ] Skill guides Claude to follow the goal and acceptance criteria
- [ ] Skill instructs Claude to update the bug's Log section with progress
- [ ] Skill instructs Claude to check off acceptance criteria as completed

## Context

When `docket work <id>` runs, it:
1. Creates/enters a jj workspace
2. Writes `.docket/current.json` with bug details
3. Sets `DOCKET_BUG` environment variable
4. Execs `claude`

The skill should pick up from there and guide the agent through implementation.

The skill should be namespaced as `docket:implement` so repos can have their own implement skills.

## Log
