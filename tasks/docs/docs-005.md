---
id: docs-005
title: Add CONTRIBUTING.md with development guidelines
status: todo
priority: medium
tags:
- docs
dependencies:
- deployment-002
assignee: developer
created: 2025-12-23T19:29:48.362004904Z
estimate: ~
complexity: 3
area: docs
---

# Add CONTRIBUTING.md with development guidelines

## Causation Chain
> Trace the documentation chain: code signature → docstring → generated
docs → published output. Check actual code-to-docs sync status - are
examples runnable?

## Pre-flight Checks
- [ ] Read dependency task files for implementation context (Session Handoff)
- [ ] Compare doc examples with actual API signatures
- [ ] Check that code snippets are runnable
- [ ] Verify cross-references are valid
- [ ] `git log --oneline -10` - Check recent related commits

## Context
Guide contributors on development setup, code standards, and submission process. Ensures consistent quality and smooth PR reviews.

## Tasks
- [ ] Create `CONTRIBUTING.md` with sections:
  - [ ] Welcome message and project overview
  - [ ] Development Setup:
    - [ ] Prerequisites (Rust 1.70+, Chrome)
    - [ ] Clone and build instructions
    - [ ] Running tests locally
  - [ ] Code Standards:
    - [ ] Run `cargo fmt` before committing
    - [ ] Run `cargo clippy` - zero warnings policy
    - [ ] Run `cargo test` - all tests must pass
  - [ ] Pull Request Process:
    - [ ] Fork and branch naming (feature/, fix/, docs/)
    - [ ] Commit message format
    - [ ] PR description template
    - [ ] CI must pass before merge
  - [ ] Issue Reporting:
    - [ ] Bug report template hints
    - [ ] Feature request guidelines
  - [ ] Code of Conduct reference

## Acceptance Criteria
- [ ] Clear step-by-step setup instructions
- [ ] Code quality requirements documented
- [ ] PR process is unambiguous
- [ ] Links to related docs work

## Notes
Reference: `/data/git/Guard8.ai/TaskGuard/CONTRIBUTING.md` (if exists) or docs/contributing/
Keep it concise but comprehensive

---
**Session Handoff** (fill when done):
- Changed: [files/functions modified]
- Causality: [what triggers what]
- Verify: [how to test this works]
- Next: [context for dependent tasks]
