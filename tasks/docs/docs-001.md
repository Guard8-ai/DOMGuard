---
id: docs-001
title: Add error codes and troubleshooting documentation
status: done
priority: medium
tags:
- documentation
- errors
dependencies: []
assignee: domguard-team
created: 2026-01-11T21:00:00Z
estimate: 3h
complexity: 4
area: docs
---

# Add error codes and troubleshooting documentation

## Problem
Missing documentation:
- No error codes reference
- No troubleshooting for CDP connection failures (cdp.rs has 2281 lines)
- No workflow syntax documentation
- No session file format specification

## Tasks
- [ ] Create error codes reference document
- [ ] Document all CDP error scenarios and solutions
- [ ] Create workflow YAML/TOML syntax documentation
- [ ] Document session JSON file format

## Files to Create
- `docs/reference/error-codes.md`
- `docs/troubleshooting/cdp-connection.md`
- `docs/reference/workflow-syntax.md`
- `docs/reference/session-format.md`

## Acceptance Criteria
- [ ] All known error messages documented
- [ ] CDP troubleshooting covers common failures
- [ ] ReadTheDocs builds successfully
