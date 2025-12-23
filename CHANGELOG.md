# Changelog

All notable changes to DOMGuard will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added

#### Core Features
- `init` command to initialize DOMGuard in a project
- `status` command to check Chrome connection

#### Debug Mode
- `debug dom` - DOM tree inspection
- `debug aria` - Accessibility tree
- `debug console` - Console messages with `--follow` and `--filter`
- `debug network` - Network requests with filtering
- `debug storage` - localStorage/sessionStorage
- `debug cookies` - Cookie inspection
- `debug styles` - Computed styles
- `debug eval` - JavaScript evaluation
- `debug performance` - Core Web Vitals and metrics
- `debug throttle-cpu` - CPU throttling emulation
- `debug throttle-network` - Network throttling (3G, slow-3g, offline)
- `debug snapshot` - Full DOM export
- `debug tabs` - Tab management (list, new, switch, close)

#### Interact Mode
- `interact click` - Click by selector or coordinates
- `interact hover` - Hover element
- `interact type` - Type text into elements
- `interact key` - Keyboard input
- `interact hold-key` - Hold key for duration
- `interact navigate` - URL navigation
- `interact back/forward/refresh` - Browser navigation
- `interact scroll` - Scroll by pixels or to element
- `interact screenshot` - Viewport, full page, element screenshots
- `interact screenshot-region` - Screenshot specific region
- `interact pdf` - PDF export
- `interact select` - Dropdown selection
- `interact upload` - File upload
- `interact dialog` - Alert/confirm/prompt handling
- `interact wait` - Wait for element or text
- `interact wait-duration` - Fixed wait time
- `interact resize` - Viewport resize
- `interact mouse-move` - Move cursor without click
- `interact cursor-position` - Get cursor coordinates
- `interact triple-click` - Select paragraph
- `interact mouse-down/mouse-up` - Granular mouse control
- `interact drag` - Drag and drop

#### Session Recording
- `session start/stop` - Record browser actions
- `session status` - Current session info
- `session list/show` - View sessions
- `session export` - Export to JSON
- `session delete` - Remove sessions

#### Workflows
- `workflow create` - Create from session or YAML
- `workflow list/show` - View workflows
- `workflow run` - Execute workflow (with `--dry-run`)
- `workflow delete` - Remove workflows

#### Security Features
- `security check` - CAPTCHA and sensitive field detection
- `security block/unblock` - Site blocking
- `security mask` - Credential masking
- CAPTCHA detection (reCAPTCHA, hCaptcha, Cloudflare, etc.)

#### User Takeover
- `takeover request` - Request human intervention
- `takeover status` - Check takeover state
- `takeover done` - Signal completion

#### Self-Correction
- `correction config` - Configure retry behavior
- `correction analyze` - Analyze page issues
- `correction retry` - Manual retry with correction

#### Site Instructions
- `sites create/update/delete` - Per-site configurations
- `sites list/show` - View site configs

#### Design Extraction
- `inspire` - Extract design patterns (colors, typography, spacing)

#### Output
- `--json` flag for programmatic output
- Human-readable default output

[Unreleased]: https://github.com/Guard8-ai/DOMGuard/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Guard8-ai/DOMGuard/releases/tag/v0.1.0
