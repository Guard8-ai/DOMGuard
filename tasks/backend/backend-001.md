---
id: backend-001
title: Fix unsafe unwrap() calls on system time
status: done
priority: high
tags:
- error-handling
- safety
dependencies: []
assignee: domguard-team
created: 2026-01-11T21:00:00Z
estimate: 1h
complexity: 2
area: backend
---

# Fix unsafe unwrap() calls on system time

## Problem
6 unsafe `unwrap()` calls on system time can crash if clock is before UNIX epoch.

Locations:
- `interact.rs:501` - screenshot naming
- `interact.rs:780` - PDF export
- `interact.rs:927` - snapshot export
- `debug.rs:737` - debug timestamps

## Tasks
- [ ] Create helper function for safe timestamp generation
- [ ] Update all 6 locations to use safe pattern
- [ ] Add test for timestamp generation

## Files to Modify
- `src/interact.rs` (lines 501, 780, 927)
- `src/debug.rs` (line 737)

## Acceptance Criteria
- [ ] No unwrap() on system time in production code
- [ ] All timestamp operations have fallback
- [ ] Clippy passes with no warnings
