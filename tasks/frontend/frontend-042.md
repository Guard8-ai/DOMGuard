---
id: frontend-042
title: Fix type --focused command argument parsing
status: done
priority: high
tags:
- frontend
- bug
- cli
dependencies:
- frontend-041
assignee: developer
created: 2025-12-29T11:00:00Z
estimate: 30m
complexity: 1
area: frontend
---

# Fix type --focused command argument parsing

## Causation Chain
> User runs `domguard interact type --focused "hello"` →
> Clap parses "hello" as [SELECTOR], expects another <TEXT> argument →
> Error: "the following required arguments were not provided: <TEXT>" →
> User cannot type into focused element

## Problem Description
The `type` command has this signature:
```
domguard interact type [OPTIONS] [SELECTOR] <TEXT>
```

When using `--focused`, the selector should be optional and the first positional argument should be the text. But currently clap interprets:
```bash
domguard interact type --focused "hello"
```
As: selector="hello", text=missing → ERROR

## Root Cause Analysis
Location: `src/main.rs:321-331`
```rust
Type {
    selector: Option<String>,
    text: String,           // <-- Required positional arg
    #[arg(long)]
    focused: bool,
},
```

With positional arguments, `[SELECTOR]` being optional means it can be skipped, but `<TEXT>` is required and must come after. When `--focused` is used, there's no way to skip `[SELECTOR]` and provide `<TEXT>` directly.

## Solution

Change the command so when `--focused` is used, the first positional argument is the text:

```rust
Type {
    /// CSS selector (or use --focused)
    selector: Option<String>,

    /// Text to type (required unless empty type)
    text: Option<String>,

    /// Type into currently focused element
    #[arg(long)]
    focused: bool,
},
```

Then in the handler, use the first positional as text when `--focused` is true:
- If `--focused` and only one positional: treat it as text
- If no `--focused`: require selector, text is optional second positional

Actually, simpler fix: require `--focused` to always use the next arg as text.

## Implementation

### 1. Update CLI in `src/main.rs`
Make text optional and handle the logic in the command handler:

```rust
Type {
    /// CSS selector (required unless --focused)
    selector: Option<String>,

    /// Text to type
    text: Option<String>,

    /// Type into currently focused element (text becomes first positional arg)
    #[arg(long)]
    focused: bool,
},
```

### 2. Update handler logic
When `--focused` is true and selector is Some but text is None, treat selector as text.

## Tasks
- [ ] Update Type variant in InteractSubcommand to make text optional
- [ ] Update InteractCommand::Type in interact.rs
- [ ] Update handler to parse args correctly
- [ ] Update interact_type function
- [ ] Test `domguard interact type --focused "text"`
- [ ] Test `domguard interact type "selector" "text"`

## Files to Modify
- `src/main.rs` - Update Type command args
- `src/interact.rs` - Update InteractCommand and handler

## Acceptance Criteria
- [ ] `domguard interact type --focused "hello"` types "hello" into focused element
- [ ] `domguard interact type "selector" "text"` still works
- [ ] `domguard interact type "selector"` errors appropriately
- [ ] Error message is clear when neither selector nor --focused provided

## Verification
```bash
# Click input to focus it
domguard interact click "input.pl-9"
# Type into focused element
domguard interact type --focused "MSFT"
# Screenshot to verify
domguard interact screenshot -o /tmp/test-focused.png
```
