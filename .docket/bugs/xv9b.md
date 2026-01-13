---
id: xv9b
title: docket work should support --dangerously-skip-permissions and auto-run skill
status: done
priority: high
created: 2026-01-13T05:05:01.048656867Z
---

## Goal

Make `docket work` launch Claude with optimal settings for autonomous work - skip permission prompts and auto-run the implementation skill.

## Acceptance Criteria

- [x] Add `--dangerously-skip-permissions` flag to claude invocation (or make configurable)
- [x] Add `--prompt "/docket:implement"` to auto-run the skill on startup
- [x] Consider adding docket config file for these preferences (e.g., `.docket/config.toml`)
- [x] Document the flags being passed to claude

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

- 2026-01-12: Implemented CLI flags `--skip-permissions` and `--auto` for the work command
- 2026-01-12: Added config file support via `.docket/config.toml` with `[work]` section
- 2026-01-12: Updated claude invocation to pass `--dangerously-skip-permissions` and `--prompt "/docket:implement"` flags
- 2026-01-12: Added documentation in code comments and CLI help text
