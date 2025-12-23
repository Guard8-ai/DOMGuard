# DOMGuard: Local-First Chrome DevTools CLI

> Rust CLI for Chrome DevTools Protocol. Let AI use browsers like humans.

## Why DOMGuard?

MCP-UI, Playwright-MCP, and similar tools add protocol overhead, network dependencies, and abstraction layers. DOMGuard is direct CDP access - no middleware, no servers, sub-ms local response.

| Approach | Latency | Dependencies | Control |
|----------|---------|--------------|---------|
| MCP-UI | 100-500ms | MCP server, iframe sandbox | Limited |
| Playwright-MCP | 50-200ms | Node.js, MCP protocol | Abstracted |
| **DOMGuard** | <10ms | Chrome only | Full CDP |

## Core Concept

```
Chrome (--remote-debugging-port=9222)
    ↑
    │ WebSocket (CDP)
    ↓
DOMGuard CLI (Rust)
    ↑
    │ stdout/stdin
    ↓
Claude Code / AI Agent
```

No daemon required. Each command connects, executes, disconnects. Stateless by default.

## Three Modes

### 1. Inspire Mode
Extract design patterns from websites.

```bash
domguard inspire https://linear.app
domguard inspire https://stripe.com/pricing --component "pricing-card"
domguard inspire --save notion https://notion.so
```

**Outputs:**
- Color palette (computed styles)
- Typography (fonts, weights, line-heights)
- Spacing system (padding/margin patterns)
- Layout structure (flex/grid usage)
- Animation timing functions
- Screenshot at current viewport

### 2. Debug Mode
Inspect page state.

```bash
domguard debug dom                              # Full DOM tree
domguard debug dom ".sidebar"                   # Specific selector
domguard debug styles ".button"                 # Computed styles
domguard debug console                          # Console messages
domguard debug console --follow                 # Stream console
domguard debug network                          # Network requests
domguard debug network --filter "api"           # Filter requests
domguard debug eval "window.location.href"      # Execute JS
domguard debug storage                          # localStorage/sessionStorage
domguard debug cookies                          # All cookies
```

### 3. Interact Mode
Control browser like a human.

```bash
domguard interact click ".submit-btn"           # Click element
domguard interact click --coords 450,320        # Click coordinates
domguard interact type "#email" "test@test.com" # Type into input
domguard interact type --focused "hello"        # Type into focused element
domguard interact key "Enter"                   # Single key
domguard interact key "cmd+k"                   # Keyboard shortcut
domguard interact key "Tab Tab Enter"           # Key sequence
domguard interact hover ".dropdown"             # Hover element
domguard interact scroll --down 500             # Scroll pixels
domguard interact scroll --to ".footer"         # Scroll to element
domguard interact screenshot                    # Capture viewport
domguard interact screenshot --full             # Full page
domguard interact screenshot --element ".card"  # Element only
domguard interact navigate "https://example.com"# Go to URL
domguard interact back                          # Browser back
domguard interact refresh                       # Refresh page
domguard interact wait ".loading" --gone        # Wait for element removal
domguard interact wait "#content" --visible     # Wait for visibility
```

## Installation

```bash
# Requires Chrome/Chromium
# Start Chrome with debugging enabled:
chrome --remote-debugging-port=9222

# Or headless:
chrome --headless --remote-debugging-port=9222

# Install DOMGuard
cargo install domguard
```

## Configuration

```toml
# ~/.config/domguard/config.toml

[chrome]
port = 9222
host = "127.0.0.1"

[defaults]
timeout_ms = 5000
screenshot_format = "png"  # png, jpeg, webp

[inspire]
save_dir = "~/.config/domguard/inspirations"
```

## Output Formats

All commands support `--json` for machine-readable output:

```bash
domguard debug dom ".nav" --json
domguard inspire https://site.com --json
domguard interact click ".btn" --json  # Returns success/failure + timing
```

Default output is human-readable for Claude Code context.

## CDP Methods Used

### Inspire
- `DOM.getDocument`
- `CSS.getComputedStyleForNode`
- `CSS.getMatchedStylesForNode`
- `Page.captureScreenshot`
- `DOM.querySelectorAll`

### Debug
- `DOM.getDocument`
- `DOM.describeNode`
- `Runtime.evaluate`
- `Network.enable` / `Network.requestWillBeSent`
- `Console.enable` / `Console.messageAdded`
- `Storage.getCookies`
- `DOMStorage.getDOMStorageItems`

### Interact
- `Input.dispatchMouseEvent`
- `Input.dispatchKeyEvent`
- `Input.insertText`
- `DOM.scrollIntoViewIfNeeded`
- `Page.navigate`
- `Page.captureScreenshot`
- `Runtime.evaluate` (for element location)

## Rust Dependencies

```toml
[package]
name = "domguard"
version = "0.1.0"
edition = "2021"

[dependencies]
chromiumoxide = "0.7"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
dirs = "5"
toml = "0.8"
colored = "2"
base64 = "0.22"
```

## Project Structure

```
domguard/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry, clap commands
│   ├── cdp.rs            # Chrome connection handling
│   ├── inspire.rs        # Design extraction logic
│   ├── debug.rs          # DOM/console/network inspection
│   ├── interact.rs       # Mouse/keyboard/navigation
│   ├── output.rs         # Human vs JSON formatting
│   └── config.rs         # Config file handling
└── GUIDE.md
```

## Example Session

```bash
# Start Chrome
chrome --remote-debugging-port=9222 &

# Navigate and inspect
domguard interact navigate "https://github.com/login"
domguard debug dom "#login_field"
domguard interact type "#login_field" "myuser"
domguard interact type "#password" "mypass"
domguard interact click "[type=submit]"
domguard interact wait ".dashboard" --visible --timeout 10000
domguard debug dom ".dashboard" --json
```

## Claude Code Usage

Claude Code calls DOMGuard directly via bash:

```bash
# Claude sees a website and wants to understand its design
domguard inspire https://target-site.com

# Claude debugs why a button isn't working
domguard debug dom "#submit-btn"
domguard debug console --filter "error"

# Claude fills out a form
domguard interact type "#name" "John Doe"
domguard interact click "#submit"
domguard interact wait ".success-message"
```

No slash command needed - it's just a CLI tool Claude Code can invoke.

## Error Handling

```bash
# Chrome not running
$ domguard debug dom
Error: Cannot connect to Chrome at 127.0.0.1:9222
Hint: Start Chrome with --remote-debugging-port=9222

# Element not found
$ domguard interact click ".nonexistent"
Error: No element matches selector ".nonexistent"

# Timeout
$ domguard interact wait ".slow-element" --timeout 1000
Error: Timeout waiting for ".slow-element" (1000ms)
```

## Why Not Playwright/Puppeteer?

| Feature | Playwright | DOMGuard |
|---------|------------|----------|
| Runtime | Node.js | Native binary |
| Startup | ~500ms | ~10ms |
| Binary size | ~200MB | ~5MB |
| Protocol | Custom wrapper | Direct CDP |
| Use case | Testing frameworks | AI agent control |

DOMGuard is a scalpel, not a swiss army knife.

## Future: Tree-sitter Integration

Optional `--parse` flag for JavaScript analysis:

```bash
domguard debug scripts --parse              # Parse inline scripts
domguard debug scripts --parse --complexity # Complexity metrics
```

Uses tree-sitter-javascript for AST analysis. Low priority - FlowGuard handles this for local files.

## Security Notes

- DOMGuard only connects to localhost by default
- No remote Chrome connections without explicit `--host` flag
- Credentials typed via `interact type` are not logged
- `--json` output excludes sensitive data by default

## License

MIT (or Apache-2.0, TBD with Guard8 licensing strategy)

---

**Status**: Design Document  
**Version**: 0.1.0  
**Last Updated**: December 2024
