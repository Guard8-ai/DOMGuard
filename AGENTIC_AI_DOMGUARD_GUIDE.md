# DOMGuard for AI Agents

## Important: CSS Selectors Only

DOMGuard uses **standard CSS selectors** (what `document.querySelector()` accepts).

**WRONG** (Playwright/Puppeteer syntax):
```bash
domguard interact click "text=Generate"           # ❌ Not CSS
domguard interact click "button:has-text('Go')"   # ❌ Not CSS
domguard interact type --selector "input" --text "hi"  # ❌ No --selector flag
```

**CORRECT** (Standard CSS):
```bash
domguard interact click "button"                  # ✓ Tag selector
domguard interact click "#submit-btn"             # ✓ ID selector
domguard interact click ".btn-primary"            # ✓ Class selector
domguard interact click "[data-testid='submit']"  # ✓ Attribute selector
domguard interact type "textarea" "hello"         # ✓ Positional args
```

**To click by text content**, use `--text`:
```bash
domguard interact click --text "Generate Strategy"   # ✓ Click by visible text
domguard interact click --text "MSFT"                # ✓ Click dropdown option
domguard interact click --text "Submit" --nth 1      # ✓ Second element with text
```

## Quick Reference

```bash
# Setup (requires Chrome with --remote-debugging-port=9222)
domguard init                                    # Initialize project
domguard status                                  # Check Chrome connection

# Debug Mode - Inspect page state
domguard debug dom                               # Full DOM tree
domguard debug dom ".selector"                   # Specific element
domguard debug styles ".button"                  # Computed styles
domguard debug console                           # Console messages
domguard debug console --follow                  # Stream live
domguard debug network                           # Network requests
domguard debug eval "window.location.href"       # Execute JS
domguard debug storage                           # localStorage/sessionStorage
domguard debug cookies                           # All cookies
domguard debug aria                              # Accessibility tree
domguard debug tabs list                         # List browser tabs
domguard debug tabs new/switch/close <id>        # Tab management
domguard debug performance                       # Core Web Vitals
domguard debug snapshot -o page.html             # Export full DOM
domguard debug highlight ".button" --color red   # Highlight element
domguard debug captcha                           # Detect CAPTCHAs

# Interact Mode - Control browser
domguard interact click ".btn"                   # Click element
domguard interact click "button" --nth 1         # Click second button (0-indexed)
domguard interact click "button" --nth -1        # Click last button
domguard interact click --text "Submit"          # Click by visible text
domguard interact click --coords 450,320         # Click coordinates
domguard interact type "#input" "text"           # Type into element
domguard interact type --focused "text"          # Type into focused element
domguard interact key "Enter"                    # Single key
domguard interact key "cmd+k"                    # Shortcut
domguard interact key "Tab Tab Enter"            # Sequence
domguard interact hover ".menu"                  # Hover element
domguard interact scroll --down 500              # Scroll pixels
domguard interact scroll --to ".footer"          # Scroll to element
domguard interact screenshot                     # Viewport capture
domguard interact screenshot --full              # Full page
domguard interact screenshot --element ".card"   # Element only
domguard interact navigate "https://url.com"     # Go to URL
domguard interact back                           # Browser back
domguard interact refresh                        # Refresh page
domguard interact wait ".loading" --gone         # Wait for removal
domguard interact wait "#content" --visible      # Wait for visible
domguard interact wait --text "Success"          # Wait for text
domguard interact wait-duration 2000             # Wait N milliseconds
domguard interact select "#country" "US"         # Select by value
domguard interact select "#country" "USA" --by-label  # By visible text
domguard interact upload "#file" ./doc.pdf       # File upload
domguard interact dialog --accept                # Accept alert/confirm
domguard interact drag --from "#src" --to "#dst" # Drag element
domguard interact resize 1920 1080               # Resize viewport
domguard interact pdf -o page.pdf                # Export to PDF

# Advanced Mouse
domguard interact mouse-move 100,200             # Move cursor (no click)
domguard interact triple-click ".paragraph"      # Select paragraph
domguard interact mouse-down left                # Press mouse button
domguard interact mouse-up left                  # Release mouse button

# Session Recording
domguard session start --name "My Task"          # Start recording
domguard session stop                            # Stop and save
domguard session list                            # List saved sessions
domguard session replay <id>                     # Replay session

# User Takeover (for CAPTCHA, 2FA, etc.)
domguard takeover request captcha                # Request user help
domguard takeover done                           # Resume automation

# Self-Correction
domguard correction dismiss-overlay              # Dismiss blocking modals
domguard correction wait-stable                  # Wait for page to stabilize
```

## Output Formats

```bash
domguard --json debug dom                        # Machine-readable JSON
domguard --json interact screenshot              # Returns base64 + path
```

## Chrome Setup

```bash
# Linux
chrome --remote-debugging-port=9222

# macOS
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222

# Windows
"C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
```

## Common Workflows

```bash
# Fill and submit a form
domguard interact type "#email" "test@test.com"
domguard interact type "#password" "secret"
domguard interact click "[type=submit]"
domguard interact wait ".dashboard" --visible

# Select from dropdown by text
domguard interact type "input.search" "MSFT"
domguard interact click --text "Microsoft Corporation"

# Handle CAPTCHA
domguard takeover request captcha -i "Please solve the CAPTCHA"
# User solves manually...
domguard takeover done
```

## Error Messages

| Error | Solution |
|-------|----------|
| Cannot connect to Chrome | Start Chrome with `--remote-debugging-port=9222` |
| No element matches selector | Check selector, element may not exist yet |
| Timeout waiting for element | Increase `--timeout` or check if element appears |

## Security Notes

- Only connects to localhost by default
- Credentials from `interact type` are not logged
- File uploads validate local file existence

---
**Version**: 0.4.2 | **Config**: `.domguard/config.toml`
