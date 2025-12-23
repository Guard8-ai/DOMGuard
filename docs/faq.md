# FAQ

Frequently asked questions about DOMGuard.

## General

### What is DOMGuard?

DOMGuard is a local-first Chrome DevTools CLI designed for AI agents. It provides direct access to the Chrome DevTools Protocol (CDP) without requiring MCP servers or cloud services.

### Why use DOMGuard over Playwright MCP?

- **Local-first**: No server dependencies
- **Any LLM**: Works with Claude, GPT, Gemini, or any AI
- **Offline capable**: Works without internet
- **More features**: Session recording, workflows, CAPTCHA detection

### Is DOMGuard free?

Yes, DOMGuard is open source and free under the MIT License.

## Installation

### How do I install DOMGuard?

```bash
cargo install domguard
```

Or download pre-built binaries from [GitHub Releases](https://github.com/Guard8-ai/DOMGuard/releases).

### What are the requirements?

- Rust 1.70+ (for building from source)
- Chrome/Chromium browser

## Connection

### How do I connect to Chrome?

Start Chrome with remote debugging enabled:

```bash
chrome --remote-debugging-port=9222
```

Then verify connection:

```bash
domguard status
```

### Can I connect to a remote Chrome instance?

By default, DOMGuard only connects to localhost for security. To enable remote connections:

```toml
# .domguard/config.toml
[security]
allow_remote = true
```

Then use:

```bash
domguard --host 192.168.1.100 status
```

### What if Chrome is already running?

Close all Chrome instances first, then restart with the debugging flag. Alternatively, use a separate Chrome profile:

```bash
chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
```

## Usage

### How do I find element selectors?

1. Use `domguard debug dom` to see the DOM tree
2. Use `domguard debug aria` for accessibility tree
3. Use Chrome DevTools Inspector (right-click → Inspect)

### Why is my click not working?

Common issues:

1. **Element not visible**: Use `domguard interact wait "selector"` first
2. **Wrong selector**: Verify with `domguard debug dom "selector"`
3. **Element in iframe**: Not yet supported
4. **Element covered**: Try scrolling first

### How do I handle dynamic content?

Use wait commands:

```bash
# Wait for element
domguard interact wait "div.loaded"

# Wait for text
domguard interact wait --text "Success"

# Wait fixed duration
domguard interact wait-duration 2000
```

### How do I handle CAPTCHAs?

Use the takeover feature for human intervention:

```bash
# Detect CAPTCHA
domguard security check --captcha

# Request human help
domguard takeover request --reason "CAPTCHA detected"

# After human solves it
domguard takeover done
```

## Troubleshooting

### "Connection refused" error

1. Ensure Chrome is running with `--remote-debugging-port=9222`
2. Check if the port is correct: `domguard --port 9222 status`
3. Verify no firewall blocking

### "No target found" error

1. Ensure Chrome has at least one tab open
2. Navigate to a webpage (not chrome:// pages)

### Commands are slow

1. Check network throttling: `domguard debug throttle-network --disable`
2. Check CPU throttling: `domguard debug throttle-cpu --disable`
3. Reduce timeout if appropriate

### Screenshot is blank

1. Wait for page to load: `domguard interact wait "body"`
2. Check if page has content: `domguard debug dom`

## Integration

### How do I use DOMGuard with Claude?

Add to your system prompt:

```markdown
You have access to DOMGuard for browser automation.
Use `domguard --json <command>` for programmatic output.
```

### How do I use JSON output?

```bash
domguard --json status
domguard --json debug dom
domguard --json interact screenshot
```

Parse with jq, Python, or any JSON library.

## Contributing

### How do I contribute?

1. Fork the repository
2. Create a feature branch
3. Make changes
4. Run `cargo fmt && cargo clippy && cargo test`
5. Submit a Pull Request

See [Contributing Guide](contributing/development-setup.md) for details.
