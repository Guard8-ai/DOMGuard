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

## Command Overview

| Category | Commands |
|----------|----------|
| **Setup** | `init`, `status` |
| **Inspect** | `debug dom`, `aria`, `console`, `network`, `storage`, `cookies`, `styles`, `eval`, `performance`, `tabs` |
| **Navigate** | `interact navigate`, `back`, `forward`, `refresh` |
| **Mouse** | `interact click`, `hover`, `triple-click`, `mouse-move`, `mouse-down`, `mouse-up`, `drag`, `cursor-position` |
| **Keyboard** | `interact type`, `key`, `hold-key` |
| **Wait** | `interact wait`, `wait-duration` |
| **Capture** | `interact screenshot`, `screenshot-region`, `pdf` |
| **Forms** | `interact select`, `upload`, `dialog`, `resize` |
| **Recording** | `session start`, `stop`, `status`, `list`, `show`, `export`, `delete` |
| **Automation** | `workflow create`, `list`, `run`, `show`, `delete` |
| **Security** | `security check`, `block`, `unblock`, `blocked`, `mask` |
| **Takeover** | `takeover request`, `status`, `done` |
| **Correction** | `correction config`, `analyze`, `retry` |
| **Sites** | `sites create`, `list`, `show`, `update`, `delete`, `block` |
| **Performance** | `debug throttle-cpu`, `throttle-network`, `snapshot` |
| **Design** | `inspire` |

## Modes

### 1. Debug Mode - Inspect Page State

```bash
# DOM & Accessibility
domguard debug dom                          # Full DOM tree
domguard debug dom ".sidebar"               # Specific selector
domguard debug aria                         # Accessibility tree
domguard debug styles ".button"             # Computed styles

# Console & Network
domguard debug console                      # Console messages
domguard debug console --follow             # Stream live
domguard debug network                      # Network requests
domguard debug network --filter "api"       # Filter requests

# Storage
domguard debug storage                      # localStorage/sessionStorage
domguard debug cookies                      # All cookies
domguard debug eval "window.location.href"  # Execute JS

# Tabs
domguard debug tabs list                    # List all tabs
domguard debug tabs new "https://..."       # Create tab
domguard debug tabs switch <id>             # Switch tab
domguard debug tabs close <id>              # Close tab

# Performance
domguard debug performance                  # Core Web Vitals
domguard debug throttle-cpu 4               # CPU slowdown
domguard debug throttle-network slow-3g     # Network throttling
domguard debug snapshot -o page.html        # DOM export
```

### 2. Interact Mode - Control Browser

```bash
# Mouse
domguard interact click ".submit-btn"       # Click element
domguard interact click --coords 450,320    # Click coordinates
domguard interact hover ".dropdown"         # Hover
domguard interact triple-click "p.content"  # Select paragraph
domguard interact mouse-move 100,200        # Move cursor
domguard interact mouse-down left           # Press button
domguard interact mouse-up left             # Release button
domguard interact drag --from "#a" --to "#b"# Drag and drop

# Keyboard
domguard interact type "#email" "test@x.com"# Type into input
domguard interact type --focused "hello"    # Type to focused
domguard interact key "Enter"               # Single key
domguard interact key "cmd+k"               # Shortcut
domguard interact hold-key Shift --duration 1000

# Navigation
domguard interact navigate "https://..."    # Go to URL
domguard interact back                      # Browser back
domguard interact forward                   # Browser forward
domguard interact refresh                   # Refresh

# Scroll
domguard interact scroll --down 500         # Scroll pixels
domguard interact scroll --to ".footer"     # Scroll to element

# Screenshots & PDF
domguard interact screenshot                # Viewport
domguard interact screenshot --full         # Full page
domguard interact screenshot --element ".x" # Element only
domguard interact screenshot-region 0,0,800,600
domguard interact pdf -o page.pdf           # PDF export

# Forms
domguard interact select "#country" "US"    # Dropdown
domguard interact upload "input" ./file.pdf # File upload
domguard interact dialog --accept           # Accept alert
domguard interact resize 1920 1080          # Viewport size

# Wait
domguard interact wait ".loading" --gone    # Wait for removal
domguard interact wait "#content"           # Wait for element
domguard interact wait --text "Success"     # Wait for text
domguard interact wait-duration 2000        # Wait fixed time
```

### 3. Session Recording

```bash
domguard session start                      # Start recording
domguard session start --name "login"       # Named session
domguard session status                     # Current session
domguard session stop                       # Stop and save
domguard session list                       # List sessions
domguard session show <id>                  # View details
domguard session export <id> -o file.json   # Export
domguard session delete <id>                # Delete
```

### 4. Workflows - Reusable Automation

```bash
domguard workflow create "login" --from-session <id>
domguard workflow create "test" --file workflow.yaml
domguard workflow list                      # List workflows
domguard workflow run "login"               # Execute
domguard workflow run "login" --dry-run     # Preview
domguard workflow show "login"              # View details
domguard workflow delete "login"            # Delete
```

### 5. Security Features

```bash
# Security checks
domguard security check                     # Full scan
domguard security check --captcha           # CAPTCHA detection
domguard security check --sensitive         # Sensitive fields

# Site blocking
domguard security block "bad-site.com"      # Block site
domguard security unblock "site.com"        # Unblock
domguard security blocked                   # List blocked

# Credential masking
domguard security mask --enable             # Enable masking
domguard security mask --disable            # Disable
```

### 6. User Takeover

```bash
domguard takeover request                   # Request human control
domguard takeover request --reason "CAPTCHA"# With reason
domguard takeover status                    # Check status
domguard takeover done                      # Human done
```

### 7. Self-Correction

```bash
domguard correction config --show           # View settings
domguard correction config --max-retries 3  # Set retries
domguard correction config --strategy adaptive
domguard correction analyze                 # Analyze page
domguard correction retry                   # Manual retry
```

### 8. Per-Site Instructions

```bash
domguard sites create "x.com" --instructions "..."
domguard sites list                         # List sites
domguard sites show "x.com"                 # View config
domguard sites update "x.com" --instructions "..."
domguard sites delete "x.com"               # Delete
domguard sites block "x.com"                # Block site
```

### 9. Inspire Mode - Design Extraction

```bash
domguard inspire https://stripe.com         # Extract design
domguard inspire https://x.com --component "nav"
domguard inspire https://x.com --save "design-name"
```

Extracts: colors, typography, spacing, layout, animations.

## Installation

```bash
# Install
cargo install domguard

# Start Chrome with debugging
chrome --remote-debugging-port=9222

# Initialize project
domguard init

# Check connection
domguard status
```

## Configuration

```toml
# .domguard/config.toml

[chrome]
port = 9222
host = "127.0.0.1"

[defaults]
timeout_ms = 30000
screenshot_format = "png"

[security]
allow_remote = false
mask_credentials = true

[correction]
max_retries = 3
strategy = "adaptive"
```

## Output Formats

```bash
# Human-readable (default)
domguard debug dom

# JSON for programmatic use
domguard --json debug dom
domguard --json interact screenshot
```

## Project Structure

```
domguard/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry, clap commands
│   ├── cdp.rs            # Chrome connection handling
│   ├── config.rs         # Config file handling
│   ├── debug.rs          # DOM/console/network/tabs inspection
│   ├── interact.rs       # Mouse/keyboard/navigation
│   ├── session.rs        # Session recording
│   ├── workflow.rs       # Workflow management
│   ├── security.rs       # Security checks
│   ├── captcha.rs        # CAPTCHA detection
│   ├── takeover.rs       # User takeover
│   ├── correction.rs     # Self-correction
│   ├── site_instructions.rs # Per-site config
│   ├── explanation.rs    # Action explanation
│   ├── inspire.rs        # Design extraction
│   └── output.rs         # Human vs JSON formatting
├── docs/                 # MkDocs documentation
├── tasks/                # TaskGuard tasks
└── .github/workflows/    # CI/CD
```

## Security Notes

- Localhost only by default (`security.allow_remote = false`)
- Credential masking in output
- CAPTCHA detection (reCAPTCHA, hCaptcha, Cloudflare, etc.)
- Blocked site management
- Sensitive field detection
- User takeover for human intervention

## License

MIT License - see [LICENSE](LICENSE)

---

**Version**: 0.1.0
**Repository**: https://github.com/Guard8-ai/DOMGuard
**Built with**: [TaskGuard](https://github.com/Guard8-ai/TaskGuard) in 2 days
