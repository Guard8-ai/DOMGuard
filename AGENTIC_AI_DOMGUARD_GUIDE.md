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
domguard interact click "button[type='submit']"   # ✓ Attribute match
domguard interact type "textarea" "hello"         # ✓ Positional args
```

**To click by text content**, use `debug eval`:
```bash
domguard debug eval "document.evaluate(\"//button[contains(text(),'Generate')]\", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue?.click()"
```

## Quick Reference

```bash
# Setup (requires Chrome with --remote-debugging-port=9222)
domguard init                                    # Initialize project
domguard status                                  # Check Chrome connection

# Inspire Mode - Extract design patterns
domguard inspire https://site.com                # Full design analysis
domguard inspire https://site.com --component ".card"  # Specific component
domguard inspire --save mysite https://site.com  # Save to .domguard/inspirations/

# Debug Mode - Inspect page state
domguard debug dom                               # Full DOM tree
domguard debug dom ".selector"                   # Specific element
domguard debug styles ".button"                  # Computed styles
domguard debug console                           # Console messages
domguard debug console --follow                  # Stream live
domguard debug network                           # Network requests
domguard debug network --filter "api"            # Filter requests
domguard debug eval "window.location.href"       # Execute JS
domguard debug storage                           # localStorage/sessionStorage
domguard debug cookies                           # All cookies
domguard debug aria                              # Accessibility tree
domguard debug aria ".nav"                       # Element ARIA snapshot
domguard debug tabs list                         # List browser tabs
domguard debug tabs new "https://url.com"        # Open new tab
domguard debug tabs switch <id>                  # Switch to tab
domguard debug tabs close <id>                   # Close tab

# Performance & DevTools (Chrome DevTools MCP features)
domguard debug performance                       # Core Web Vitals + metrics
domguard debug snapshot -o page.html             # Export full DOM as HTML
domguard debug network-details                   # Detailed request info
domguard debug network-details --filter "api"    # Filter by URL
domguard debug throttle cpu 4                    # 4x CPU slowdown
domguard debug throttle network3g                # Simulate 3G network
domguard debug throttle network-slow3g           # Simulate slow 3G
domguard debug throttle network-offline          # Simulate offline
domguard debug throttle off                      # Disable throttling

# Interact Mode - Control browser
domguard interact click ".btn"                   # Click element (first match)
domguard interact click "button" --nth 1         # Click second button
domguard interact click "button" --nth -1        # Click last button
domguard interact click --coords 450,320         # Click coordinates
domguard interact type "#input" "text"           # Type into element
domguard interact type --focused "text"          # Type into focused
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
domguard interact wait --text-gone "Loading"     # Wait text gone

# Forms
domguard interact select "#country" "US"         # Select by value
domguard interact select "#country" "USA" --by-label  # By visible text
domguard interact upload "#file" ./doc.pdf       # File upload
domguard interact dialog --accept                # Accept alert/confirm
domguard interact dialog --accept --text "yes"   # Respond to prompt

# Drag & Drop
domguard interact drag --from "#src" --to "#dst" # Drag element
domguard interact drag --from-coords 100,100 --to-coords 300,300

# Viewport & Export
domguard interact resize 1920 1080               # Resize viewport
domguard interact pdf -o page.pdf                # Export to PDF
domguard interact pdf --landscape                # Landscape PDF

# Advanced Mouse (Anthropic Computer Use)
domguard interact mouse-move 100,200             # Move cursor (no click)
domguard interact cursor-position                # Get cursor coords
domguard interact triple-click ".paragraph"      # Select paragraph
domguard interact triple-click --coords 100,200  # Triple-click at point
domguard interact mouse-down left                # Press mouse button
domguard interact mouse-up left                  # Release mouse button

# Advanced Keyboard
domguard interact hold-key Shift --duration 1000 # Hold key for 1s

# Screenshot Region (crop/zoom)
domguard interact screenshot-region 0,0,800,600  # x,y,width,height
domguard interact screenshot-region 100,100,400,300 -o crop.png

# Timing
domguard interact wait-duration 2000             # Wait 2 seconds

# Session Recording
domguard session start --name "My Task"          # Start recording
domguard session status                          # Show current session
domguard session pause                           # Pause recording
domguard session resume                          # Resume recording
domguard session stop                            # Stop and save session
domguard session list                            # List saved sessions
domguard session show <id>                       # Show session details
domguard session export <id> --format bash       # Export as bash script
domguard session export <id> --format markdown   # Export as markdown
domguard session replay <id> --delay 500         # Replay session

# Element Highlighting
domguard debug highlight ".button" --color red   # Highlight element
domguard debug highlight "a" --all --color blue  # Highlight all with labels
domguard debug highlight ".card" --duration 3000 # Auto-clear after 3s
domguard debug clear-highlights                  # Clear all highlights

# Security
domguard security check type "[type=password]" -v "test"  # Check sensitive input
domguard security check navigate "https://bank.com"       # Check navigation
domguard security check upload "/path/to/file.pdf"        # Check file upload
domguard security block "malicious-site.com"    # Block a site pattern
domguard security unblock "example.com"         # Unblock a site
domguard security list-blocked                  # List blocked sites
domguard security config                        # Show security config

# User Takeover Mode
domguard takeover request captcha               # Request takeover for CAPTCHA
domguard takeover request auth -m "Please login" # Request for authentication
domguard takeover request sensitive -i "Review before proceeding"  # Sensitive action
domguard takeover request 2fa                   # Request for 2FA
domguard takeover request payment               # Request for payment confirmation
domguard takeover request error -m "Error occurred"  # Request on error
domguard takeover status                        # Check takeover status
domguard takeover done                          # Mark complete and resume
domguard takeover done --success false          # Mark as failed
domguard takeover cancel                        # Cancel takeover
domguard takeover history                       # View takeover history

# Self-Correction
domguard correction config                      # Show correction settings
domguard correction enable                      # Enable auto-recovery
domguard correction disable                     # Disable auto-recovery
domguard correction analyze "element not found" # Analyze error and show strategies
domguard correction strategies not-found        # Show strategies for error type
domguard correction strategies captcha          # Show strategies for CAPTCHA
domguard correction test scroll -t ".button"    # Test scroll strategy
domguard correction test dismiss-overlay        # Test overlay dismissal
domguard correction dismiss-overlay             # Dismiss page overlays
domguard correction wait-stable                 # Wait for page to stabilize
```

## Output Formats

```bash
domguard --json debug dom                        # Machine-readable JSON
domguard --json interact screenshot              # Returns base64 + path
domguard --json inspire https://site.com         # Structured design data
```

Default: Human-readable for Claude Code context.

## Chrome Setup

```bash
# Start Chrome with debugging
chrome --remote-debugging-port=9222

# Or headless (required for PDF export)
chrome --headless --remote-debugging-port=9222

# macOS
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222

# Windows
"C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
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
```

## Common Workflows

### Analyze a website's design
```bash
domguard interact navigate "https://target.com"
domguard inspire https://target.com
```

### Debug a form submission
```bash
domguard debug dom "#submit-btn"
domguard debug console --filter "error"
domguard interact click "#submit-btn"
domguard interact wait ".success" --visible
```

### Fill and submit a form
```bash
domguard interact type "#email" "test@test.com"
domguard interact type "#password" "secret"
domguard interact click "[type=submit]"
domguard interact wait ".dashboard" --visible --timeout 10000
```

### Multi-tab workflow
```bash
domguard debug tabs new "https://docs.example.com"
# ... work in new tab ...
domguard debug tabs list
domguard debug tabs switch <original-tab-id>
```

### Upload file and submit
```bash
domguard interact upload "input[type=file]" ./document.pdf
domguard interact wait --text "Upload complete"
domguard interact click "#submit"
```

### Precise mouse control
```bash
domguard interact mouse-move 500,300            # Position cursor
domguard interact mouse-down left               # Start drag
domguard interact mouse-move 700,400            # Drag to position
domguard interact mouse-up left                 # Release
```

### Select text with triple-click
```bash
domguard interact triple-click ".article p:first-child"
domguard interact key "cmd+c"                   # Copy selected
```

### Capture specific region
```bash
domguard interact screenshot-region 0,0,1200,800 -o hero.png
```

### Handle CAPTCHA with takeover
```bash
domguard debug captcha                          # Detect CAPTCHA
domguard takeover request captcha -i "Please solve the CAPTCHA"
# User solves CAPTCHA manually...
domguard takeover done                          # Resume automation
domguard interact wait ".logged-in" --visible   # Continue workflow
```

### Handle 2FA with takeover
```bash
domguard interact click "#login-btn"
domguard takeover request 2fa -m "Enter 2FA code" -e "Should see dashboard"
# User enters 2FA code...
domguard takeover done -n "2FA completed successfully"
domguard interact wait ".dashboard" --visible
```

## Error Messages

| Error | Solution |
|-------|----------|
| Cannot connect to Chrome | Start Chrome with `--remote-debugging-port=9222` |
| No element matches selector | Check selector, element may not exist yet |
| Timeout waiting for element | Increase `--timeout` or check if element appears |
| Non-localhost connection blocked | Security: only localhost allowed by default |
| PDF export failed | PDF requires Chrome headless mode |
| File not found | Verify file path for upload command |

## Security Notes

- Only connects to localhost by default
- Remote connections require explicit `--host` flag
- Credentials from `interact type` are not logged
- `--json` output excludes sensitive data
- File uploads validate local file existence

## Command Reference

### Debug Commands
| Command | Description |
|---------|-------------|
| `debug dom [selector]` | Inspect DOM tree |
| `debug styles <selector>` | Get computed styles |
| `debug console [--follow] [--filter]` | View console messages |
| `debug network [--filter]` | View network requests |
| `debug network-details [--filter]` | Detailed request info (timing, size) |
| `debug eval <js>` | Execute JavaScript |
| `debug storage` | View localStorage/sessionStorage |
| `debug cookies` | View all cookies |
| `debug aria [selector]` | Accessibility tree |
| `debug tabs list/new/switch/close` | Tab management |
| `debug performance` | Core Web Vitals + runtime metrics |
| `debug snapshot [-o file]` | Export full DOM as HTML |
| `debug throttle cpu/network3g/off` | CPU/network throttling |
| `debug highlight <selector>` | Highlight element(s) |
| `debug clear-highlights` | Clear all highlights |

### Interact Commands
| Command | Description |
|---------|-------------|
| `interact click [--nth N]` | Click element/coordinates (--nth for nth match) |
| `interact type` | Type text into element |
| `interact key` | Press key(s) |
| `interact hover` | Hover over element |
| `interact scroll` | Scroll page |
| `interact screenshot` | Capture viewport/full/element |
| `interact screenshot-region` | Capture cropped region |
| `interact navigate` | Go to URL |
| `interact back` | Browser back |
| `interact refresh` | Refresh page |
| `interact wait` | Wait for element/text |
| `interact wait-duration` | Wait N milliseconds |
| `interact drag` | Drag and drop |
| `interact select` | Select dropdown option |
| `interact upload` | Upload file(s) |
| `interact dialog` | Handle alert/confirm/prompt |
| `interact resize` | Resize viewport |
| `interact pdf` | Export page as PDF |
| `interact mouse-move` | Move cursor (no click) |
| `interact cursor-position` | Get cursor coordinates |
| `interact triple-click` | Select paragraph |
| `interact mouse-down` | Press mouse button |
| `interact mouse-up` | Release mouse button |
| `interact hold-key` | Hold key for duration |

### Session Commands
| Command | Description |
|---------|-------------|
| `session start` | Start recording session |
| `session stop` | Stop and save session |
| `session pause` | Pause recording |
| `session resume` | Resume recording |
| `session status` | Show current session |
| `session list` | List saved sessions |
| `session show <id>` | Show session details |
| `session export <id>` | Export as bash/markdown |
| `session replay <id>` | Replay recorded session |

### Security Commands
| Command | Description |
|---------|-------------|
| `security check <action> <target>` | Check if action is sensitive |
| `security block <pattern>` | Block a site pattern |
| `security unblock <pattern>` | Unblock a site |
| `security list-blocked` | List blocked sites |
| `security config` | Show security config |

### Explain Commands
| Command | Description |
|---------|-------------|
| `explain click <target>` | Explain what a click will do |
| `explain type <target>` | Explain what typing will do |
| `explain key <keys>` | Explain key press effects |
| `explain navigate <url>` | Explain navigation action |
| `explain wait <target>` | Explain wait action |
| `explain interact <cmd> [target]` | Explain any interact command |

### CAPTCHA Detection
| Command | Description |
|---------|-------------|
| `debug captcha` | Detect CAPTCHAs on current page |

### Site Instructions Commands
| Command | Description |
|---------|-------------|
| `sites list` | List all saved site instructions |
| `sites show <domain>` | Show instructions for domain |
| `sites create <domain>` | Create template instructions |
| `sites delete <domain>` | Delete site instructions |
| `sites current` | Get instructions for current page |
| `sites edit <domain>` | Edit in default editor |

### Workflow Commands
| Command | Description |
|---------|-------------|
| `workflow list [--tag X] [--domain X]` | List workflows |
| `workflow show <id>` | Show workflow details |
| `workflow from-session <id> <name>` | Create from session |
| `workflow create <name>` | Create empty workflow |
| `workflow run <id> [-p key=val]` | Run workflow |
| `workflow run <id> --dry-run` | Preview workflow |
| `workflow delete <id>` | Delete workflow |
| `workflow edit <id>` | Edit in default editor |

### Takeover Commands
| Command | Description |
|---------|-------------|
| `takeover request <reason>` | Request user takeover (captcha, auth, sensitive, error, etc.) |
| `takeover request <reason> -m "message"` | Request with custom message |
| `takeover request <reason> -i "instructions"` | Request with instructions |
| `takeover done` | Mark takeover complete and resume |
| `takeover done --success false -n "notes"` | Complete with failure status |
| `takeover cancel` | Cancel takeover without completing |
| `takeover status` | Check if takeover is active |
| `takeover history` | List takeover history |

### Self-Correction Commands
| Command | Description |
|---------|-------------|
| `correction config` | Show self-correction configuration |
| `correction enable` | Enable automatic error recovery |
| `correction disable` | Disable automatic error recovery |
| `correction analyze "error message"` | Classify error and show recovery strategies |
| `correction strategies <type>` | Show strategies for error type (not-found, captcha, etc.) |
| `correction test <strategy> [-t selector]` | Test a recovery strategy |
| `correction dismiss-overlay` | Dismiss blocking overlays/modals |
| `correction wait-stable` | Wait for page to stop changing |

---
**Version**: 0.2.0 | **Config**: `.domguard/config.toml`
