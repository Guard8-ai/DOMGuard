# DOMGuard

**Local-First Chrome DevTools CLI for AI Agents**

Direct CDP access for AI agents. No middleware, no servers, sub-ms local response.

## Why DOMGuard?

| Feature | DOMGuard | Others |
|---------|:--------:|:------:|
| **Local-first** | ✓ | Cloud/MCP |
| **Any LLM** | ✓ | Locked |
| **Free** | ✓ | $250/mo+ |
| **Offline** | ✓ | No |
| **Open Source** | ✓ | Mixed |

## Quick Start

```bash
# Install
cargo install domguard

# Start Chrome with debugging
chrome --remote-debugging-port=9222

# Check connection
domguard status

# Try it out
domguard debug dom
domguard interact click "button.submit"
domguard interact screenshot
```

## Features

- **Debug Mode** - Inspect DOM, ARIA, console, network, storage, performance
- **Interact Mode** - Click, type, navigate, screenshot, PDF, forms
- **Session Recording** - Capture and replay browser sessions
- **Workflows** - Create reusable automation scripts
- **Security** - CAPTCHA detection, blocked sites, user takeover
- **Performance** - Metrics, CPU/network throttling, snapshots

## For AI Agents

DOMGuard is designed for AI agent integration. Add to your system prompt:

```markdown
You have access to DOMGuard for browser automation via shell commands.

Core commands:
- `domguard status` - Check Chrome connection
- `domguard debug dom` - Inspect DOM tree
- `domguard debug aria` - Get accessibility tree
- `domguard interact click "<selector>"` - Click element
- `domguard interact type "<selector>" "<text>"` - Type text
- `domguard interact screenshot` - Take screenshot
- `domguard --json <command>` - Get JSON output
```

## Links

- [GitHub Repository](https://github.com/Guard8-ai/DOMGuard)
- [Getting Started](getting-started/installation.md)
- [Full Command Reference](api-reference/commands.md)
