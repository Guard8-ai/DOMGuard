---
id: testing-001
title: Add integration tests for core modules
status: done
priority: critical
tags:
- testing
- integration
- cdp
dependencies: []
assignee: domguard-team
created: 2026-01-11T21:00:00Z
estimate: 8h
complexity: 8
area: testing
---

# Add integration tests for core modules

## Problem
7 core modules have 0 tests:
- `main.rs` (3154 lines) - CLI entry point
- `cdp.rs` (2281 lines) - Chrome DevTools Protocol
- `interact.rs` (959 lines) - Browser interaction
- `debug.rs` (1054 lines) - Debug commands
- `inspire.rs` (374 lines) - AI inspiration
- `correction.rs` (450 lines) - Self-correction
- `takeover.rs` (361 lines) - Session takeover

## Tasks
- [ ] Create `tests/` directory structure
- [ ] Add CDP connection tests (mock Chrome)
- [ ] Add interact command tests
- [ ] Add debug command tests
- [ ] Add CLI integration tests
- [ ] Set up test fixtures for browser state
- [ ] Add CI step for integration tests

## Files to Create
- `tests/cdp_tests.rs`
- `tests/interact_tests.rs`
- `tests/cli_tests.rs`

## Acceptance Criteria
- [ ] Each core module has at least 5 tests
- [ ] CDP connection error paths tested
- [ ] CLI argument parsing tested
- [ ] Tests run in CI pipeline
