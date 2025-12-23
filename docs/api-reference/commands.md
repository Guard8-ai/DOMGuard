# Commands Reference

Complete list of DOMGuard commands.

## Global Options

```bash
domguard [OPTIONS] <COMMAND>

Options:
  --json               Output in JSON format
  --host <HOST>        Chrome DevTools host (default: 127.0.0.1)
  --port <PORT>        Chrome DevTools port (default: 9222)
  --timeout <TIMEOUT>  Command timeout in milliseconds
  -h, --help           Print help
  -V, --version        Print version
```

## Commands

### Setup

| Command | Description |
|---------|-------------|
| `init` | Initialize DOMGuard in current directory |
| `status` | Check Chrome connection status |

### Debug

| Command | Description |
|---------|-------------|
| `debug dom [selector]` | Inspect DOM tree |
| `debug aria [selector]` | Accessibility tree |
| `debug console` | View console messages |
| `debug console --follow` | Stream console live |
| `debug network` | View network requests |
| `debug storage` | View localStorage/sessionStorage |
| `debug cookies` | View cookies |
| `debug styles <selector>` | Computed styles |
| `debug eval <js>` | Execute JavaScript |
| `debug performance` | Performance metrics |
| `debug snapshot -o <file>` | Export DOM snapshot |
| `debug throttle-cpu <rate>` | CPU throttling |
| `debug throttle-network <preset>` | Network throttling |
| `debug tabs list` | List browser tabs |
| `debug tabs new <url>` | Create new tab |
| `debug tabs switch <id>` | Switch to tab |
| `debug tabs close <id>` | Close tab |

### Interact

| Command | Description |
|---------|-------------|
| `interact click <selector>` | Click element |
| `interact click --coords <x,y>` | Click coordinates |
| `interact hover <selector>` | Hover element |
| `interact type <selector> <text>` | Type text |
| `interact type --focused <text>` | Type to focused |
| `interact key <key>` | Press key |
| `interact hold-key <key> --duration <ms>` | Hold key |
| `interact navigate <url>` | Go to URL |
| `interact back` | Go back |
| `interact forward` | Go forward |
| `interact refresh` | Refresh page |
| `interact scroll --down <px>` | Scroll down |
| `interact scroll --to <selector>` | Scroll to element |
| `interact screenshot` | Take screenshot |
| `interact screenshot --full` | Full page screenshot |
| `interact screenshot-region <x,y,w,h>` | Screenshot region |
| `interact pdf -o <file>` | Export PDF |
| `interact select <selector> <value>` | Select dropdown |
| `interact upload <selector> <file>` | Upload file |
| `interact dialog --accept` | Accept dialog |
| `interact wait <selector>` | Wait for element |
| `interact wait --text <text>` | Wait for text |
| `interact wait-duration <ms>` | Wait fixed time |
| `interact resize <w> <h>` | Resize viewport |
| `interact mouse-move <x,y>` | Move cursor |
| `interact cursor-position` | Get cursor position |
| `interact triple-click <selector>` | Select paragraph |
| `interact mouse-down <button>` | Press mouse button |
| `interact mouse-up <button>` | Release mouse button |
| `interact drag --from <sel> --to <sel>` | Drag and drop |

### Session

| Command | Description |
|---------|-------------|
| `session start` | Start recording |
| `session stop` | Stop recording |
| `session status` | Current session info |
| `session list` | List all sessions |
| `session show <id>` | View session details |
| `session export <id> -o <file>` | Export session |
| `session delete <id>` | Delete session |

### Workflow

| Command | Description |
|---------|-------------|
| `workflow create <name> --from-session <id>` | Create from session |
| `workflow create <name> --file <yaml>` | Create from file |
| `workflow list` | List workflows |
| `workflow run <name>` | Run workflow |
| `workflow run <name> --dry-run` | Preview workflow |
| `workflow show <name>` | View workflow |
| `workflow delete <name>` | Delete workflow |

### Security

| Command | Description |
|---------|-------------|
| `security check` | Full security scan |
| `security check --captcha` | CAPTCHA detection |
| `security check --sensitive` | Sensitive fields |
| `security block <domain>` | Block site |
| `security unblock <domain>` | Unblock site |
| `security blocked` | List blocked sites |
| `security mask --enable` | Enable credential masking |
| `security mask --disable` | Disable masking |

### Takeover

| Command | Description |
|---------|-------------|
| `takeover request` | Request human control |
| `takeover request --reason <text>` | Request with reason |
| `takeover status` | Check takeover status |
| `takeover done` | Signal completion |

### Correction

| Command | Description |
|---------|-------------|
| `correction config --show` | View settings |
| `correction config --max-retries <n>` | Set retry limit |
| `correction config --strategy <name>` | Set strategy |
| `correction analyze` | Analyze page issues |
| `correction retry` | Manual retry |

### Sites

| Command | Description |
|---------|-------------|
| `sites create <domain> --instructions <text>` | Create site config |
| `sites list` | List configured sites |
| `sites show <domain>` | View site config |
| `sites update <domain> --instructions <text>` | Update config |
| `sites delete <domain>` | Delete config |

### Inspire

| Command | Description |
|---------|-------------|
| `inspire <url>` | Extract design patterns |
| `inspire <url> --component <selector>` | Focus on component |
| `inspire <url> --save <name>` | Save for reference |
