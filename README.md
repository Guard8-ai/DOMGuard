# DOMGuard

<p align="center">
  <img src="docs/assets/logo.png" alt="DOMGuard Logo" width="200">
</p>

<p align="center">
  <a href="https://github.com/Guard8-ai/DOMGuard/actions/workflows/ci.yml"><img src="https://github.com/Guard8-ai/DOMGuard/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/Guard8-ai/DOMGuard/releases/latest"><img src="https://img.shields.io/github/release/Guard8-ai/DOMGuard.svg" alt="Release"></a>
  <a href="https://domguard.readthedocs.io/en/latest/"><img src="https://readthedocs.org/projects/domguard/badge/?version=latest" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <strong>Local-First Chrome DevTools CLI for AI Agents</strong>
</p>

Direct CDP access for AI agents. No middleware, no servers, sub-ms local response.

## Why DOMGuard?

| Feature | DOMGuard | Playwright MCP | Chrome DevTools MCP | Project Mariner | OpenAI Operator |
|---------|----------|----------------|---------------------|-----------------|-----------------|
| **Architecture** | Local CLI | MCP Server | MCP Server | Cloud VM | Cloud VM |
| **Latency** | Sub-ms | Network RTT | Network RTT | High (cloud) | High (cloud) |
| **Privacy** | 100% local | Server-dependent | Server-dependent | Cloud processing | Cloud processing |
| **Cost** | Free | Free | Free | $249.99/mo | ChatGPT Pro |
| **Offline** | Yes | No | No | No | No |
| **AI Integration** | Any LLM | Claude only | Claude/Cursor | Gemini only | GPT-4o only |
| **Open Source** | Yes | Yes | Yes | No | No |

## Quick Start

### Download Pre-built Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/Guard8-ai/DOMGuard/releases/latest):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `domguard-linux-x86_64` |
| macOS ARM64 (Apple Silicon) | `domguard-macos-aarch64` |
| Windows x86_64 | `domguard-windows-x86_64.exe` |

```bash
# Linux/macOS: Make executable and move to PATH
chmod +x domguard-*
sudo mv domguard-* /usr/local/bin/domguard

# Windows: Add to PATH or run directly
```

### Install from Source

```bash
cargo install domguard
```

### Get Started

```bash
# Initialize in your project
domguard init

# Start Chrome with debugging enabled
chrome --remote-debugging-port=9222

# Check connection
domguard status

# Try it out
domguard debug dom
domguard interact click "button.submit"
domguard inspire https://example.com
```

## Features

### Debug Mode - Inspect Page State

```bash
# DOM inspection
domguard debug dom                          # Full DOM tree
domguard debug dom "div.container"          # Specific element

# Styles
domguard debug styles "button.primary"      # Computed styles

# Console
domguard debug console                      # View messages
domguard debug console --follow             # Stream live
domguard debug console --filter "error"     # Filter messages

# Network
domguard debug network                      # All requests
domguard debug network --filter "api"       # Filter by URL

# JavaScript
domguard debug eval "document.title"        # Execute JS

# Storage
domguard debug storage                      # localStorage/sessionStorage
domguard debug cookies                      # View cookies

# Accessibility
domguard debug aria                         # Full ARIA tree
domguard debug aria "nav"                   # Specific element

# Tab Management
domguard debug tabs list                    # List all tabs
domguard debug tabs new "https://example.com"  # Create tab
domguard debug tabs switch <tab-id>         # Switch to tab
domguard debug tabs close <tab-id>          # Close tab
```

### Interact Mode - Control Browser

```bash
# Mouse Actions
domguard interact click "button.submit"           # Click element
domguard interact click --coords 100,200          # Click coordinates
domguard interact hover "div.menu"                # Hover element
domguard interact drag --from "#source" --to "#target"  # Drag and drop

# Keyboard
domguard interact type "input.search" "hello"     # Type text
domguard interact type --focused "hello world"    # Type to focused
domguard interact key "Enter"                     # Press key
domguard interact key "ctrl+a ctrl+c"             # Key sequence

# Forms
domguard interact select "select#country" "US"              # By value
domguard interact select "select#country" "United States" --by-label
domguard interact upload "input[type=file]" ./doc.pdf       # File upload

# Navigation
domguard interact navigate "https://example.com"  # Go to URL
domguard interact back                            # Go back
domguard interact refresh                         # Refresh page
domguard interact scroll --down 500               # Scroll pixels
domguard interact scroll --to "footer"            # Scroll to element

# Screenshots & PDF
domguard interact screenshot                      # Viewport
domguard interact screenshot --full               # Full page
domguard interact screenshot --element "div.hero" # Element
domguard interact pdf -o page.pdf                 # Export PDF
domguard interact pdf --landscape                 # Landscape PDF

# Dialogs
domguard interact dialog --accept                 # Accept alert
domguard interact dialog --accept --text "yes"    # Prompt response

# Viewport
domguard interact resize 1920 1080                # Resize viewport

# Wait Conditions
domguard interact wait "div.loaded"               # Wait for element
domguard interact wait "div.spinner" --gone       # Wait until gone
domguard interact wait --text "Success"           # Wait for text
domguard interact wait --text-gone "Loading..."   # Wait text gone

# Advanced Mouse Control (Anthropic Computer Use)
domguard interact mouse-move 100,200              # Move cursor without click
domguard interact cursor-position                 # Get current cursor coords
domguard interact triple-click "p.content"        # Select paragraph
domguard interact triple-click --coords 100,200   # Triple-click at coords
domguard interact mouse-down left                 # Press mouse button
domguard interact mouse-up left                   # Release mouse button

# Advanced Keyboard (Anthropic Computer Use)
domguard interact hold-key Shift --duration 1000  # Hold key for 1 second

# Screenshot Region (zoom/crop)
domguard interact screenshot-region 0,0,800,600   # Capture region x,y,w,h
domguard interact screenshot-region 100,100,400,300 -o crop.png

# Wait Duration
domguard interact wait-duration 2000              # Wait 2 seconds
```

### Inspire Mode - Extract Design Patterns

```bash
# Extract design system from any website
domguard inspire https://stripe.com

# Focus on specific component
domguard inspire https://stripe.com --component "nav.header"

# Save for later reference
domguard inspire https://stripe.com --save "stripe-nav"
```

Extracts:
- Color palette with usage frequency
- Typography (fonts, sizes, weights, line-heights)
- Spacing system (padding, margin, gap values)
- Layout patterns (flex/grid usage)
- Animation timing functions

### Session Recording - Capture Action History

```bash
# Start recording actions
domguard session start                    # Begin new session
domguard session start --name "checkout"  # Named session

# Check recording status
domguard session status                   # Show current session info

# Stop and save recording
domguard session stop                     # Stop current session

# List recorded sessions
domguard session list                     # Show all sessions

# View session details
domguard session show <session-id>        # Detailed action log

# Export session
domguard session export <session-id> -o session.json

# Delete session
domguard session delete <session-id>
```

### Workflow Mode - Reusable Automation

```bash
# Create workflow from recorded session
domguard workflow create "login-flow" --from-session <session-id>

# Create workflow from YAML file
domguard workflow create "checkout" --file workflow.yaml

# List all workflows
domguard workflow list

# Run a workflow
domguard workflow run "login-flow"
domguard workflow run "login-flow" --dry-run    # Preview without executing

# View workflow details
domguard workflow show "login-flow"

# Delete workflow
domguard workflow delete "login-flow"
```

### User Takeover - Human-in-the-Loop

```bash
# Request human takeover (pauses automation)
domguard takeover request                 # Pause for human intervention
domguard takeover request --reason "CAPTCHA detected"

# Check takeover status
domguard takeover status

# Signal completion (resume automation)
domguard takeover done
```

### Self-Correction - Automatic Error Recovery

```bash
# Configure correction behavior
domguard correction config --max-retries 3
domguard correction config --strategy "adaptive"

# View correction settings
domguard correction config --show

# Analyze page for potential issues
domguard correction analyze

# Manual retry with correction
domguard correction retry
```

### Site Instructions - Per-Site Behaviors

```bash
# Create site-specific instructions
domguard sites create "example.com" --instructions "Always click cookie accept"

# List configured sites
domguard sites list

# View site instructions
domguard sites show "example.com"

# Update instructions
domguard sites update "example.com" --instructions "New behavior"

# Delete site config
domguard sites delete "example.com"

# Block a site
domguard sites block "malicious-site.com"
domguard sites unblock "malicious-site.com"
```

### Security Commands - Safety Features

```bash
# Check page for security concerns
domguard security check                   # Full security scan
domguard security check --captcha         # CAPTCHA detection only
domguard security check --sensitive       # Sensitive field detection

# Block/unblock sites
domguard security block "phishing-site.com"
domguard security unblock "safe-site.com"
domguard security blocked                 # List blocked sites

# Credential masking
domguard security mask --enable           # Enable credential masking
domguard security mask --disable          # Disable masking
```

### Performance & Throttling

```bash
# Get performance metrics
domguard debug performance                # Core Web Vitals, timing

# CPU throttling
domguard debug throttle-cpu 4             # 4x slowdown
domguard debug throttle-cpu --disable     # Disable throttling

# Network throttling
domguard debug throttle-network slow-3g   # Slow 3G preset
domguard debug throttle-network 3g        # Regular 3G
domguard debug throttle-network offline   # Offline mode
domguard debug throttle-network --disable # Disable throttling

# DOM snapshot export
domguard debug snapshot -o page.html      # Export full DOM
```

## Feature Comparison

| Capability | DOMGuard | Playwright MCP | DevTools MCP | Mariner | Operator |
|------------|:--------:|:--------------:|:------------:|:-------:|:--------:|
| **Architecture** | Local CLI | MCP Server | MCP Server | Cloud | Cloud |
| **Cost** | Free | Free | Free | $250/mo | Pro |
| **Privacy** | Full | Partial | Partial | Cloud | Cloud |
| **Any LLM** | ✓ | | | | |
| **Open Source** | ✓ | ✓ | ✓ | | |
| **Offline** | ✓ | | | | |
| Click/Type/Navigate | ✓ | ✓ | | ✓ | ✓ |
| Screenshots | ✓ | ✓ | ✓ | ✓ | ✓ |
| Screenshot Region | ✓ | | | ✓ | ✓ |
| PDF Export | ✓ | ✓ | | | |
| Tab Management | ✓ | ~ | | ✓ | ✓ |
| DOM Inspection | ✓ | ✓ | ✓ | | |
| ARIA/Accessibility | ✓ | ✓ | | | |
| Console Messages | ✓ | ✓ | ✓ | | |
| Network Monitoring | ✓ | ✓ | ✓ | | |
| Performance Metrics | ✓ | | ✓ | | |
| CPU/Network Throttling | ✓ | | ✓ | | |
| Triple-click | ✓ | | | ✓ | ✓ |
| Mouse Down/Up | ✓ | | | ✓ | ✓ |
| Hold Key | ✓ | | | ✓ | ✓ |
| Session Recording | ✓ | | | ✓ | ✓ |
| Reusable Workflows | ✓ | | | ✓ | ✓ |
| Self-correction | ✓ | | | ✓ | ✓ |
| User Takeover | ✓ | | | ✓ | ✓ |
| CAPTCHA Detection | ✓ | | | ✓ | ✓ |
| Per-site Instructions | ✓ | | | ✓ | ✓ |
| Blocked Sites | ✓ | | | ✓ | ✓ |
| Design Extraction | ✓ | | | | |

✓ = supported, ~ = limited

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   AI Agent      │     │     Chrome      │
│  (Any LLM)      │     │   Browser       │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │ CLI calls             │ CDP WebSocket
         │                       │ (localhost:9222)
         ▼                       │
┌─────────────────┐              │
│    DOMGuard     │◄─────────────┘
│   (Local CLI)   │
└─────────────────┘
```

**Key Benefits:**
- **Zero latency**: Direct CDP connection, no middleware
- **Full privacy**: Everything runs locally
- **Any LLM**: Not locked to specific AI provider
- **Offline capable**: Works without internet
- **Simple integration**: Just shell commands

## Output Formats

```bash
# Human-readable (default)
domguard debug dom

# JSON for programmatic use
domguard --json debug dom
domguard --json interact screenshot
```

## Configuration

```bash
# Initialize creates .domguard/config.toml
domguard init
```

```toml
# .domguard/config.toml
[chrome]
host = "127.0.0.1"
port = 9222

[defaults]
timeout_ms = 30000
screenshot_format = "png"

[security]
allow_remote = false  # Only localhost by default
```

## CLI Options

```bash
domguard [OPTIONS] <COMMAND>

Options:
  --json               Output in JSON format
  --host <HOST>        Chrome DevTools host (default: 127.0.0.1)
  --port <PORT>        Chrome DevTools port (default: 9222)
  --timeout <TIMEOUT>  Command timeout in milliseconds
  -h, --help           Print help
  -V, --version        Print version

Commands:
  init       Initialize DOMGuard in current directory
  status     Check Chrome connection status
  inspire    Extract design patterns from websites
  debug      Inspect page state (DOM, console, network, storage, performance)
  interact   Control browser (click, type, navigate, screenshot)
  session    Record and manage browser sessions
  workflow   Create and run reusable automation workflows
  takeover   Human-in-the-loop control handoff
  correction Self-correction and error recovery settings
  sites      Per-site instructions and blocked sites
  security   Security checks, CAPTCHA detection, credential masking
```

## Security

- **Localhost only by default**: Won't connect to remote Chrome instances
- **Credential masking**: Sensitive data (tokens, passwords) masked in output
- **No data collection**: Everything stays on your machine
- **Open source**: Full code transparency

## For AI Agents

DOMGuard is designed for AI agent integration. See [AGENTIC_AI_DOMGUARD_GUIDE.md](AGENTIC_AI_DOMGUARD_GUIDE.md) for the complete quick reference.

### System Prompt Example

```markdown
You have access to DOMGuard for browser automation via shell commands.

## Core Commands
- `domguard status` - Check Chrome connection
- `domguard debug dom [selector]` - Inspect DOM tree
- `domguard debug aria` - Get accessibility tree (useful for understanding page structure)
- `domguard interact click "<selector>"` - Click element
- `domguard interact type "<selector>" "<text>"` - Type text
- `domguard interact screenshot` - Take screenshot
- `domguard interact wait "<selector>"` - Wait for element
- `domguard --json <command>` - Get JSON output for parsing

## Advanced Commands
- `domguard interact mouse-move <x>,<y>` - Move cursor without clicking
- `domguard interact triple-click "<selector>"` - Select paragraph
- `domguard interact screenshot-region <x>,<y>,<w>,<h>` - Capture region
- `domguard interact hold-key <key> --duration <ms>` - Hold key
- `domguard debug tabs list` - List browser tabs

## Session & Workflow Commands
- `domguard session start` - Start recording actions
- `domguard session stop` - Stop and save recording
- `domguard workflow run "<name>"` - Execute saved workflow
- `domguard takeover request --reason "<reason>"` - Request human intervention

## Safety Commands
- `domguard security check` - Check for CAPTCHAs, sensitive fields
- `domguard security check --captcha` - CAPTCHA detection only
- `domguard correction analyze` - Analyze page for potential issues

## Best Practices
1. Always check `domguard status` before automation
2. Use `--json` for programmatic parsing
3. Use `debug aria` to understand page structure for accessibility
4. Use `wait` commands before interacting with dynamic elements
5. Use `screenshot` to verify visual state when needed
6. Use `security check` before interacting with login/payment forms
7. Use `takeover request` when encountering CAPTCHAs or complex interactions
```

### Complete Command Reference

| Category | Command | Description |
|----------|---------|-------------|
| **Setup** | `init`, `status` | Initialize and check connection |
| **Inspect** | `debug dom`, `debug aria`, `debug console`, `debug network` | Page inspection |
| **Navigate** | `interact navigate`, `back`, `refresh` | Browser navigation |
| **Click** | `interact click`, `hover`, `triple-click` | Mouse clicks |
| **Type** | `interact type`, `key`, `hold-key` | Keyboard input |
| **Wait** | `interact wait`, `wait-duration` | Synchronization |
| **Forms** | `interact select`, `upload`, `dialog` | Form interaction |
| **Capture** | `interact screenshot`, `screenshot-region`, `pdf` | Page capture |
| **Advanced** | `mouse-move`, `mouse-down`, `mouse-up`, `cursor-position` | Precise control |
| **Tabs** | `debug tabs list/new/switch/close` | Tab management |
| **Design** | `inspire` | Extract design patterns |
| **Recording** | `session start/stop/status/list/show/export` | Session recording |
| **Workflows** | `workflow create/list/run/show/delete` | Reusable automation |
| **Takeover** | `takeover request/status/done` | Human-in-the-loop |
| **Correction** | `correction config/analyze/retry` | Error recovery |
| **Sites** | `sites create/list/show/update/delete/block` | Per-site config |
| **Security** | `security check/block/unblock/blocked/mask` | Safety features |
| **Performance** | `debug performance`, `throttle-cpu`, `throttle-network` | Performance testing |

## Installation

### From Source

```bash
git clone https://github.com/Guard8-ai/DOMGuard.git
cd DOMGuard
cargo build --release
```

### Requirements

- Rust 1.70+
- Chrome/Chromium with `--remote-debugging-port=9222`

## License

MIT License - see [LICENSE](LICENSE)

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Built with [TaskGuard](https://github.com/Guard8-ai/TaskGuard) in 2 days.

---

**DOMGuard** - Local-first browser automation for AI agents.
