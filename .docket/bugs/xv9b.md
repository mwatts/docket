---
id: xv9b
title: docket work should support --dangerously-skip-permissions and auto-run skill
status: draft
priority: high
created: 2026-01-13T05:05:01.048656867Z
---

## Goal

Make `docket work` launch Claude with optimal settings for autonomous work - skip permission prompts and auto-run the implementation skill.

## Acceptance Criteria

- [ ] Add `--dangerously-skip-permissions` flag to claude invocation (or make configurable)
- [ ] Add `--prompt "/docket:implement"` to auto-run the skill on startup
- [ ] Consider adding docket config file for these preferences (e.g., `.docket/config.toml`)
- [ ] Document the flags being passed to claude

## Context

Currently `docket work <id>` just runs `claude` with the `DOCKET_BUG` env var. Users then need to:
1. Manually type `/docket:implement` to start the skill
2. Deal with permission prompts for every file operation

For power users who trust the workflow, we should support:
```bash
claude --dangerously-skip-permissions --prompt "/docket:implement"
```

Could be:
- Always on (opinionated)
- Flags to `docket work` (e.g., `docket work --auto <id>`)
- Config file setting

## Log

