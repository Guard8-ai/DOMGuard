---
id: frontend-039
title: Fix type command for React controlled inputs
status: done
priority: high
tags:
- frontend
- react
- bug
- type-command
dependencies:
- setup-001
assignee: developer
created: 2025-12-28T19:55:00Z
estimate: 2h
complexity: 3
area: frontend
---

# Fix type command for React controlled inputs

## Causation Chain
> Current `type_into()` sets `el.value` directly →
> React's synthetic event system intercepts `.value` setter →
> React's internal state doesn't update →
> `onChange` never fires → Input appears empty

## Problem Description
The `type` command fails on React controlled inputs because React intercepts the `.value` setter. The current implementation in `src/cdp.rs:439-470` sets `.value` directly and dispatches `input`/`change` events, but React ignores this because its internal value tracking isn't bypassed.

## Root Cause Analysis
Location: `src/cdp.rs:452-455`
```rust
el.value = '{}';
el.dispatchEvent(new Event('input', {{ bubbles: true }}));
el.dispatchEvent(new Event('change', {{ bubbles: true }}));
```

React wraps the native `.value` property descriptor. Setting `.value` directly goes through React's wrapper, which doesn't trigger state updates. Must use the **native prototype's value setter** to bypass React's interception.

## Solution (Aligned with DOMGuard Philosophy)

**Zero-friction, just works**: Auto-detect React and use the right approach. No flags needed.

Update `type_into()` in `src/cdp.rs` to use native value setter:

```javascript
(function() {
    const el = document.querySelector('SELECTOR');
    if (!el) return false;
    el.focus();
    if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
        // Use native setter to bypass React's interception
        const proto = el.tagName === 'INPUT'
            ? HTMLInputElement.prototype
            : HTMLTextAreaElement.prototype;
        const nativeSetter = Object.getOwnPropertyDescriptor(proto, 'value').set;
        nativeSetter.call(el, 'TEXT');
        el.dispatchEvent(new Event('input', { bubbles: true }));
        el.dispatchEvent(new Event('change', { bubbles: true }));
    } else if (el.contentEditable === 'true') {
        el.textContent = 'TEXT';
    }
    return true;
})()
```

This works for **both** React and non-React inputs - universal solution.

## Tasks
- [ ] Update `type_into()` in `src/cdp.rs` to use native value setter
- [ ] Update `type_focused()` in `src/cdp.rs` with same pattern
- [ ] Test with React controlled input
- [ ] Test with non-React input (verify backwards compatibility)
- [ ] Run `cargo build --release` and `cargo test`

## Files to Modify
- `src/cdp.rs` - `type_into()` function (~line 439)
- `src/cdp.rs` - `type_focused()` function (~line 473)

## Acceptance Criteria
- [ ] `domguard interact type "textarea" "text"` works on React apps
- [ ] `domguard interact type "input" "text"` works on React apps
- [ ] Backwards compatible with plain HTML inputs
- [ ] No new flags or CLI options needed (just works)

## Verification
```bash
# Build
cargo build --release

# Test on React app (e.g., any React form)
domguard interact type "textarea" "test text"
domguard interact type "input[type=text]" "hello"

# Verify text appears and React state updates
```

---
**Session Handoff** (fill when done):
- Changed: `src/cdp.rs` - `type_into()`, `type_focused()`
- Causality: Native setter bypasses React wrapper → triggers real input event → React state updates
- Verify: Test on React controlled textarea/input
- Next: None (bug fix complete)