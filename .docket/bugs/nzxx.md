---
id: nzxx
title: Add docket edit command to open bug in editor
status: draft
priority: low
created: 2026-01-13T02:53:13.884435634Z
---

## Goal

Add `docket edit <id>` command that opens a bug file in the user's preferred editor.

## Acceptance Criteria

- [ ] `docket edit <id>` opens the bug file in $EDITOR (or $VISUAL, or fallback to vim/nano)
- [ ] After editing, validate the YAML frontmatter is still valid
- [ ] If validation fails, offer to re-edit or abort

## Context

Currently users can edit bug files directly, but they need to know the path. `docket edit` provides a convenient shortcut.

Use the `EDITOR` or `VISUAL` environment variables, falling back to common editors.

## Log

