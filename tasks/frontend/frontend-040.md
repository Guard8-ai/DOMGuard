---
id: frontend-040
title: Add --nth option for selecting nth matching element
status: done
priority: high
tags:
- frontend
- enhancement
- selector
dependencies:
- setup-001
assignee: developer
created: 2025-12-28T20:15:00Z
estimate: 2h
complexity: 2
area: frontend
---

# Add --nth option for selecting nth matching element

## Causation Chain
> User uses generic selector like "button" →
> `document.querySelector()` returns first DOM match (sidebar button) →
> Wrong element clicked → Automation fails

## Problem Description
When using generic CSS selectors like `button`, DOMGuard clicks the first matching element in DOM order, which may be in sidebar/nav instead of the intended main content area.

## Root Cause Analysis
Location: `src/cdp.rs:415-436`
```rust
pub async fn click(&self, selector: &str) -> Result<()> {
    // ...
    const el = document.querySelector('{}');  // Returns first match only
    // ...
}
```

`document.querySelector()` always returns the first matching element. No way to select 2nd, 3rd, etc.

## Solution (Aligned with DOMGuard Philosophy)

**Simple, standard approach**: Add `--nth` option to select nth matching element.

```bash
# Current (first match only)
domguard interact click "button"

# New (select specific match)
domguard interact click "button" --nth 2       # Second button
domguard interact click "button" --nth -1      # Last button
domguard interact click "button" --nth 0       # First (default)
```

Uses `document.querySelectorAll()[n]` - standard DOM API, no magic.

## Implementation

### 1. Update CLI in `src/main.rs` (~line 303)
```rust
/// Click element or coordinates
Click {
    /// CSS selector
    selector: Option<String>,

    /// Click at coordinates (x,y)
    #[arg(long, value_parser = parse_coords)]
    coords: Option<(f64, f64)>,

    /// Select nth matching element (0-indexed, -1 for last)
    #[arg(long, default_value = "0")]
    nth: i32,
},
```

### 2. Update InteractCommand in `src/interact.rs` (~line 40)
```rust
Click {
    selector: Option<String>,
    coords: Option<(f64, f64)>,
    nth: i32,
},
```

### 3. Update CDP click in `src/cdp.rs` (~line 415)
```rust
pub async fn click(&self, selector: &str, nth: i32) -> Result<()> {
    let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
    let result = self
        .evaluate(&format!(
            r#"
            (function() {{
                const els = document.querySelectorAll('{}');
                if (els.length === 0) return false;
                const idx = {} < 0 ? els.length + {} : {};
                if (idx < 0 || idx >= els.length) return false;
                const el = els[idx];
                el.scrollIntoView({{ block: 'center' }});
                el.click();
                return true;
            }})()
            "#,
            escaped, nth, nth, nth
        ))
        .await?;
    // ...
}
```

## Tasks
- [ ] Add `--nth` arg to Click in `src/main.rs`
- [ ] Update InteractCommand::Click in `src/interact.rs`
- [ ] Update `click()` method signature in `src/cdp.rs`
- [ ] Update `interact_click()` to pass nth parameter
- [ ] Update AI guide with --nth examples
- [ ] Build + test + verify on localhost:5173

## Files to Modify
- `src/main.rs` - Add --nth to Click subcommand (~line 303)
- `src/interact.rs` - Update InteractCommand enum and interact_click (~line 40, 146)
- `src/cdp.rs` - Update click() to use querySelectorAll with index (~line 415)
- `AGENTIC_AI_DOMGUARD_GUIDE.md` - Document --nth option

## Acceptance Criteria
- [ ] `domguard interact click "button" --nth 2` clicks third button
- [ ] `domguard interact click "button" --nth -1` clicks last button
- [ ] Default behavior unchanged (--nth 0)
- [ ] Error message if nth out of bounds

## Verification
```bash
# Build
cargo build --release && cargo install --path . --force

# Test on FortuitaSolutions (localhost:5173)
domguard interact click "button" --nth 0   # First button
domguard interact click "button" --nth 1   # Second button
domguard interact click "button" --nth -1  # Last button (Generate Strategy)
```

---
**Session Handoff** (fill when done):
- Changed: `src/main.rs`, `src/interact.rs`, `src/cdp.rs`
- Causality: --nth selects from querySelectorAll() result array
- Verify: Click correct button on FortuitaSolutions
- Next: Consider adding --nth to other commands (type, hover, etc.)