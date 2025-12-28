---
id: frontend-041
title: Add --text option for clicking elements by visible text content
status: in-progress
priority: high
tags:
- frontend
- enhancement
- selector
- dropdown
dependencies:
- frontend-040
assignee: developer
created: 2025-12-29T10:00:00Z
estimate: 2h
complexity: 2
area: frontend
---

# Add --text option for clicking elements by visible text content

## Causation Chain
> AI agent needs to click dropdown option "MSFT" →
> Only CSS selectors available, no way to target by text →
> Agent must use complex `debug eval` with XPath →
> Poor DX for AI automation workflows

## Problem Description
Modern UI frameworks (React, Radix, Shadcn) generate dynamic dropdown options that are difficult to target with CSS selectors alone. AI agents need to click elements based on their visible text content (e.g., "MSFT", "Microsoft Corporation").

Current workaround requires complex JavaScript:
```bash
domguard debug eval "document.evaluate(\"//button[contains(text(),'MSFT')]\", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue?.click()"
```

This is error-prone and verbose for AI agents.

## Root Cause Analysis
Location: `src/cdp.rs` click function only accepts CSS selectors.

The `click()` method uses `document.querySelectorAll()` which only supports CSS selectors, not text-based selection.

## Solution (Aligned with DOMGuard Philosophy)

**Simple, standard approach**: Add `--text` option to click elements containing specific text.

```bash
# Current (CSS selectors only)
domguard interact click "button"

# New (click by text content)
domguard interact click --text "MSFT"                    # Any element containing "MSFT"
domguard interact click --text "Microsoft Corporation"   # Partial match
domguard interact click --text "Generate Strategy"       # Button text

# Combined with --nth for multiple matches
domguard interact click --text "Button" --nth 1          # Second element with "Button"
```

Uses standard DOM traversal - finds elements where `textContent` contains the search text.

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
    #[arg(long, default_value = "0", allow_hyphen_values = true)]
    nth: i32,

    /// Click element containing this text
    #[arg(long)]
    text: Option<String>,
},
```

### 2. Update InteractCommand in `src/interact.rs` (~line 40)
```rust
Click {
    selector: Option<String>,
    coords: Option<(f64, f64)>,
    nth: i32,
    text: Option<String>,
},
```

### 3. Add click_by_text in `src/cdp.rs`
```rust
pub async fn click_by_text(&self, text: &str, nth: i32) -> Result<()> {
    let escaped = text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n");
    let result = self
        .evaluate(&format!(
            r#"
            (function() {{
                const text = '{}';
                const nth = {};
                // Find all elements containing the text
                const walker = document.createTreeWalker(
                    document.body,
                    NodeFilter.SHOW_ELEMENT,
                    {{
                        acceptNode: (node) => {{
                            // Check if this element directly contains the text (not just children)
                            const directText = Array.from(node.childNodes)
                                .filter(n => n.nodeType === Node.TEXT_NODE)
                                .map(n => n.textContent)
                                .join('');
                            if (directText.includes(text)) return NodeFilter.FILTER_ACCEPT;
                            // Also check full textContent for leaf-ish elements
                            if (node.children.length <= 2 && node.textContent.includes(text)) {{
                                return NodeFilter.FILTER_ACCEPT;
                            }}
                            return NodeFilter.FILTER_SKIP;
                        }}
                    }}
                );
                const matches = [];
                while (walker.nextNode()) matches.push(walker.currentNode);
                if (matches.length === 0) return {{ found: false, count: 0 }};
                const idx = nth < 0 ? matches.length + nth : nth;
                if (idx < 0 || idx >= matches.length) return {{ found: false, count: matches.length, index: idx }};
                const el = matches[idx];
                el.scrollIntoView({{ block: 'center' }});
                el.click();
                return {{ found: true, count: matches.length, clicked: el.tagName }};
            }})()
            "#,
            escaped, nth
        ))
        .await?;
    // Handle result...
}
```

### 4. Update interact_click in `src/interact.rs`
```rust
pub async fn interact_click(
    cdp: &CdpConnection,
    selector: Option<String>,
    coords: Option<(f64, f64)>,
    nth: i32,
    text: Option<String>,
    json_output: bool,
) -> Result<()> {
    if let Some(text) = text {
        cdp.click_by_text(&text, nth).await?;
    } else if let Some(sel) = selector {
        cdp.click(&sel, nth).await?;
    } else if let Some((x, y)) = coords {
        cdp.click_at(x, y).await?;
    } else {
        anyhow::bail!("Click requires --text, selector, or --coords");
    }
    Ok(())
}
```

## Tasks
- [ ] Add `--text` arg to Click in `src/main.rs`
- [ ] Update InteractCommand::Click in `src/interact.rs`
- [ ] Add `click_by_text()` method in `src/cdp.rs`
- [ ] Update `interact_click()` to handle text option
- [ ] Update AI guide with --text examples
- [ ] Build + test + verify on localhost:5173

## Files to Modify
- `src/main.rs` - Add --text to Click subcommand (~line 303)
- `src/interact.rs` - Update InteractCommand enum and interact_click (~line 40, 146)
- `src/cdp.rs` - Add click_by_text() method
- `AGENTIC_AI_DOMGUARD_GUIDE.md` - Document --text option

## Acceptance Criteria
- [ ] `domguard interact click --text "MSFT"` clicks element containing "MSFT"
- [ ] `domguard interact click --text "Generate Strategy"` clicks button
- [ ] `domguard interact click --text "Button" --nth 1` clicks second matching element
- [ ] Error message if no element contains the text
- [ ] Works with dynamic dropdown options (Radix/Shadcn)

## Verification
```bash
# Build
cargo build --release && cargo install --path . --force

# Test on FortuitaSolutions (localhost:5173)
# 1. Type into search to show dropdown
domguard interact type "input[placeholder*='Search']" "MSFT"
# 2. Click the dropdown option by text
domguard interact click --text "Microsoft Corporation"
# 3. Verify MSFT is selected
domguard interact screenshot -o /tmp/test-text-click.png
```

---
**Session Handoff** (fill when done):
- Changed: `src/main.rs`, `src/interact.rs`, `src/cdp.rs`
- Causality: --text finds elements by textContent using TreeWalker
- Verify: Click MSFT in dropdown on FortuitaSolutions
- Next: Consider --text for other commands (hover, wait)
