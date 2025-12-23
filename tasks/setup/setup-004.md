---
id: setup-004
title: Update domguard init to copy AGENTIC guide and config templates
status: todo
priority: high
tags:
- setup
dependencies:
- deployment-001
assignee: developer
created: 2025-12-23T19:29:39.232450144Z
estimate: ~
complexity: 3
area: setup
---

# Update domguard init to copy AGENTIC guide and config templates

## Causation Chain
> Trace the initialization chain: env detection → dependency check →
config load → service bootstrap → ready state. Verify actual failure
modes and error messages in bootstrap code.

## Pre-flight Checks
- [ ] Read dependency task files for implementation context (Session Handoff)
- [ ] `grep -r "init\|bootstrap\|main" src/` - Find initialization
- [ ] Check actual failure modes and error messages
- [ ] Verify dependency checks are comprehensive
- [ ] `git log --oneline -10` - Check recent related commits

## Context
Like TaskGuard's init copies guides and templates, DOMGuard's init should copy the AGENTIC_AI_DOMGUARD_GUIDE.md and config templates to help AI agents and users get started quickly.

## Tasks
- [ ] Update `domguard init` command in `src/main.rs`:
  - [ ] Copy `AGENTIC_AI_DOMGUARD_GUIDE.md` to `.domguard/` directory
  - [ ] Create `.domguard/config.toml` with default settings
  - [ ] Create `.domguard/templates/` directory structure
- [ ] Embed guide content in binary (include_str! or similar)
- [ ] Add `--force` flag to overwrite existing files
- [ ] Print helpful message after init:
  ```
  Initialized DOMGuard in current directory.

  Files created:
    .domguard/config.toml          - Configuration
    .domguard/AGENTIC_AI_GUIDE.md  - AI agent quick reference

  Next steps:
    1. Start Chrome: chrome --remote-debugging-port=9222
    2. Check connection: domguard status
    3. Try it out: domguard debug dom
  ```
- [ ] Test init in fresh directory
- [ ] Test init with existing .domguard (should warn without --force)

## Acceptance Criteria
- [ ] `domguard init` creates .domguard directory
- [ ] Config file created with sensible defaults
- [ ] AGENTIC guide copied for AI agent reference
- [ ] Warning if .domguard already exists
- [ ] `--force` overwrites existing files

## Notes
Reference: TaskGuard's init behavior
Embed files at compile time for single-binary distribution

---
**Session Handoff** (fill when done):
- Changed: [files/functions modified]
- Causality: [what triggers what]
- Verify: [how to test this works]
- Next: [context for dependent tasks]
