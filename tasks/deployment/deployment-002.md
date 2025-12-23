---
id: deployment-002
title: Add GitHub CI workflow (build, test, clippy, fmt, security audit)
status: todo
priority: critical
tags:
- deployment
dependencies:
- deployment-001
- testing-012
assignee: developer
created: 2025-12-23T19:29:22.207821645Z
estimate: ~
complexity: 3
area: deployment
---

# Add GitHub CI workflow (build, test, clippy, fmt, security audit)

## Causation Chain
> Trace the deployment pipeline: source → build → artifact →
environment config → runtime injection → health check. Verify actual
env var usage and fallback defaults in config files.

## Pre-flight Checks
- [ ] Read dependency task files for implementation context (Session Handoff)
- [ ] `grep -r "env\|getenv\|std::env" src/` - Find env var usage
- [ ] Check actual config file loading order
- [ ] Verify health check endpoints exist
- [ ] `git log --oneline -10` - Check recent related commits

## Context
Ensure code quality and prevent regressions with automated CI. Every PR and push should be validated for build success, test passing, code formatting, and security vulnerabilities.

## Tasks
- [ ] Create `.github/workflows/ci.yml`
- [ ] Configure matrix build: ubuntu-latest, macos-latest, windows-latest
- [ ] Add Rust toolchain setup with `dtolnay/rust-toolchain@stable`
- [ ] Add cargo caching with `actions/cache@v4`
- [ ] Add `cargo build --verbose` step
- [ ] Add `cargo test --verbose` step
- [ ] Add `cargo fmt -- --check` (ubuntu only)
- [ ] Add `cargo clippy -- -D warnings` (ubuntu only)
- [ ] Add security audit job with `cargo audit`
- [ ] Test workflow by pushing to branch

## Acceptance Criteria
- [ ] CI runs on push to master and PRs
- [ ] Build passes on Linux, macOS, Windows
- [ ] Tests pass on all platforms
- [ ] Clippy reports zero warnings
- [ ] Format check passes
- [ ] Security audit runs without critical vulnerabilities

## Notes
Reference: `/data/git/Guard8.ai/TaskGuard/.github/workflows/ci.yml`
Triggers: push to master, develop/*, PRs to master

---
**Session Handoff** (fill when done):
- Changed: [files/functions modified]
- Causality: [what triggers what]
- Verify: [how to test this works]
- Next: [context for dependent tasks]
