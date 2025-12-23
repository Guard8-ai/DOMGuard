---
id: deployment-003
title: Add GitHub Release workflow (Linux, macOS, Windows binaries)
status: todo
priority: critical
tags:
- deployment
dependencies:
- deployment-002
assignee: developer
created: 2025-12-23T19:29:26.893135295Z
estimate: ~
complexity: 3
area: deployment
---

# Add GitHub Release workflow (Linux, macOS, Windows binaries)

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
Automate binary releases for all platforms. When a version tag is pushed, automatically build and publish binaries for Linux, macOS (ARM64), and Windows to GitHub Releases.

## Tasks
- [ ] Create `.github/workflows/release.yml`
- [ ] Trigger on tag push `v*`
- [ ] Configure build matrix:
  - [ ] Linux x86_64: `ubuntu-latest`, `x86_64-unknown-linux-gnu`
  - [ ] macOS ARM64: `macos-latest`, `aarch64-apple-darwin`
  - [ ] Windows x86_64: `windows-latest`, `x86_64-pc-windows-msvc`
- [ ] Add Rust toolchain with target setup
- [ ] Build with `cargo build --release --target ${{ matrix.target }}`
- [ ] Create artifact directory and copy binaries
- [ ] Upload artifacts with `actions/upload-artifact@v4`
- [ ] Add release job that downloads all artifacts
- [ ] Generate SHA256 checksums
- [ ] Create GitHub Release with `softprops/action-gh-release@v2`
- [ ] Test with a test tag (e.g., `v0.1.0-test`)

## Acceptance Criteria
- [ ] Release workflow triggers on `v*` tags
- [ ] Binaries built for Linux, macOS ARM64, Windows
- [ ] Checksums generated and included
- [ ] GitHub Release created with all binaries
- [ ] Pre-release flag for `-rc`, `-beta`, `-alpha` tags

## Notes
Reference: `/data/git/Guard8.ai/TaskGuard/.github/workflows/release.yml`
Binary names: `domguard-linux-x86_64`, `domguard-macos-aarch64`, `domguard-windows-x86_64.exe`

---
**Session Handoff** (fill when done):
- Changed: [files/functions modified]
- Causality: [what triggers what]
- Verify: [how to test this works]
- Next: [context for dependent tasks]
