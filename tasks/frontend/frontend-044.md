---
id: frontend-044
title: Chrome Extension for visual element inspection and selector generation
status: done
priority: high
tags:
- frontend
- chrome-extension
- inspector
dependencies:
- frontend-040
assignee: developer
created: 2026-01-12T00:00:00Z
estimate: 8h
complexity: 7
area: frontend
github_issue: 1
---

# Chrome Extension for Visual Element Inspection

## Problem
DOMGuard requires `--remote-debugging-port=9222` and manual selector discovery. No visual feedback during automation.

## Solution
Chrome extension with:
1. Visual element inspector (hover to highlight, click to copy selector)
2. Action recorder (generate CLI commands from manual actions)
3. Command execution from extension popup

## Phase 1: MVP (This Task)
- [ ] Create extension manifest v3
- [ ] Content script for element highlighting
- [ ] Popup UI with element info and selector copy
- [ ] Basic inspector mode (hover to highlight)

## Files to Create
- `extension/manifest.json`
- `extension/content.js` - Element inspection/highlighting
- `extension/popup.html` - Extension popup UI
- `extension/popup.js` - Popup logic
- `extension/styles.css` - Highlight styles
- `extension/icons/` - Extension icons

## Acceptance Criteria
- [ ] Extension loads in Chrome (unpacked)
- [ ] Hover highlights elements
- [ ] Click copies CSS selector
- [ ] Popup shows selected element info
