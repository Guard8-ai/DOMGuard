# Configuration

DOMGuard configuration via `.domguard/config.toml`.

## Initialize

```bash
domguard init
```

Creates `.domguard/config.toml` with defaults.

## Configuration File

```toml
# .domguard/config.toml

[chrome]
host = "127.0.0.1"
port = 9222

[defaults]
timeout_ms = 30000
screenshot_format = "png"
screenshot_dir = ".domguard/screenshots"

[security]
allow_remote = false
mask_credentials = true
blocked_sites = []

[correction]
max_retries = 3
strategy = "adaptive"

[session]
auto_record = false
session_dir = ".domguard/sessions"
```

## Options

### Chrome Connection

| Option | Default | Description |
|--------|---------|-------------|
| `chrome.host` | `127.0.0.1` | Chrome DevTools host |
| `chrome.port` | `9222` | Chrome DevTools port |

### Defaults

| Option | Default | Description |
|--------|---------|-------------|
| `defaults.timeout_ms` | `30000` | Command timeout (ms) |
| `defaults.screenshot_format` | `png` | Screenshot format (png, jpeg, webp) |
| `defaults.screenshot_dir` | `.domguard/screenshots` | Screenshot output directory |

### Security

| Option | Default | Description |
|--------|---------|-------------|
| `security.allow_remote` | `false` | Allow non-localhost connections |
| `security.mask_credentials` | `true` | Mask sensitive data in output |
| `security.blocked_sites` | `[]` | List of blocked domains |

### Correction

| Option | Default | Description |
|--------|---------|-------------|
| `correction.max_retries` | `3` | Maximum retry attempts |
| `correction.strategy` | `adaptive` | Correction strategy |

### Session

| Option | Default | Description |
|--------|---------|-------------|
| `session.auto_record` | `false` | Auto-start recording |
| `session.session_dir` | `.domguard/sessions` | Session storage directory |

## Environment Variables

Override config with environment variables:

```bash
DOMGUARD_CHROME_HOST=192.168.1.100 domguard status
DOMGUARD_CHROME_PORT=9223 domguard debug dom
DOMGUARD_TIMEOUT=60000 domguard interact navigate "https://slow-site.com"
```

## Command Line Override

CLI options override config file:

```bash
domguard --host 192.168.1.100 --port 9223 status
domguard --timeout 60000 interact navigate "https://slow-site.com"
```

## Priority

1. Command line options (highest)
2. Environment variables
3. Config file
4. Defaults (lowest)
