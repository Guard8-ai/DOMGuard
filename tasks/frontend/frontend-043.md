---
id: frontend-043
title: Add automatic screenshot cleanup option
status: done
priority: medium
tags:
- frontend
- cleanup
- screenshots
dependencies:
- frontend-040
assignee: developer
created: 2026-01-12T00:00:00Z
estimate: 1h
complexity: 2
area: frontend
---

# Add automatic screenshot cleanup option

## Problem
DOMGuard leaves screenshot files in the screenshots directory that can accidentally get committed to git. Users need to manually clean these up.

## Solution
Add option to automatically delete screenshots after session ends or on command.

## Tasks
- [ ] Add `interact cleanup` command to delete all screenshots
- [ ] Add `--cleanup` flag to session stop to auto-delete screenshots
- [ ] Add config option `auto_cleanup_screenshots: bool`
- [ ] Respect `.gitignore` patterns for screenshot directory

## Files to Modify
- `src/main.rs` - Add cleanup subcommand
- `src/interact.rs` - Implement cleanup logic
- `src/config.rs` - Add auto_cleanup config option

## Acceptance Criteria
- [ ] `domguard interact cleanup` deletes all screenshots
- [ ] `domguard session stop --cleanup` deletes session screenshots
- [ ] Config option controls automatic cleanup behavior
