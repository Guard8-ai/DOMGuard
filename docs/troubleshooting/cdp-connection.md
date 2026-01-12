# CDP Connection Troubleshooting

Guide to resolving Chrome DevTools Protocol connection issues.

## Quick Diagnosis

```bash
# Check if DOMGuard can connect
domguard status

# Expected output:
# Connected to Chrome on 127.0.0.1:9222
# Active tab: https://example.com
```

## Starting Chrome Correctly

### Linux

```bash
# Standard
google-chrome --remote-debugging-port=9222

# Chromium
chromium --remote-debugging-port=9222

# With specific profile
google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
```

### macOS

```bash
# Google Chrome
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222

# Chromium
/Applications/Chromium.app/Contents/MacOS/Chromium --remote-debugging-port=9222

# Chrome Canary
/Applications/Google\ Chrome\ Canary.app/Contents/MacOS/Google\ Chrome\ Canary --remote-debugging-port=9222
```

### Windows

```powershell
# Google Chrome
& "C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222

# Chrome (x86)
& "C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
```

## Common Issues

### Port Already in Use

**Symptom:** Chrome fails to start or DOMGuard can't connect

**Solution:**
```bash
# Find process using port 9222
lsof -i :9222  # Linux/macOS
netstat -ano | findstr :9222  # Windows

# Kill existing Chrome instances
pkill -f chrome  # Linux/macOS
taskkill /F /IM chrome.exe  # Windows

# Restart Chrome
chrome --remote-debugging-port=9222
```

### Multiple Chrome Instances

**Symptom:** Connection works but wrong tabs/windows

**Solution:**
```bash
# Use a dedicated profile for debugging
chrome --remote-debugging-port=9222 --user-data-dir=/tmp/domguard-profile

# This creates an isolated Chrome instance
```

### Firewall Blocking

**Symptom:** Connection refused on remote machines

**Solution:**
```bash
# DOMGuard only connects to localhost by default
# To connect to remote Chrome:
domguard --host 192.168.1.100 --port 9222 status

# On the Chrome machine, allow external connections:
chrome --remote-debugging-port=9222 --remote-debugging-address=0.0.0.0
```

### Chrome Extensions Interfering

**Symptom:** Commands fail or behave unexpectedly

**Solution:**
```bash
# Start Chrome without extensions
chrome --remote-debugging-port=9222 --disable-extensions
```

### Headless Mode Issues

**Symptom:** Can't see what's happening

**Solution:**
```bash
# Use new headless mode (Chrome 112+)
chrome --remote-debugging-port=9222 --headless=new

# Old headless (deprecated)
chrome --remote-debugging-port=9222 --headless
```

## Connection Configuration

### Config File

```toml
# .domguard/config.toml
[chrome]
host = "127.0.0.1"
port = 9222
```

### Command Line

```bash
# Override host/port per command
domguard --host localhost --port 9333 status
```

## Verifying CDP Endpoint

```bash
# Check Chrome's CDP endpoint directly
curl http://127.0.0.1:9222/json/version

# Expected response:
# {
#   "Browser": "Chrome/XXX",
#   "Protocol-Version": "1.3",
#   "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/..."
# }

# List available tabs
curl http://127.0.0.1:9222/json/list
```

## Tab Management

```bash
# List all tabs
domguard debug tabs list

# Switch to specific tab
domguard debug tabs switch <tab-id>

# Open new tab
domguard debug tabs new "https://example.com"
```

## Debugging Connection Issues

### Enable Verbose Output

```bash
# Set Rust log level
RUST_LOG=debug domguard status
```

### Check Chrome Logs

```bash
# Start Chrome with logging
chrome --remote-debugging-port=9222 --enable-logging --v=1
```

## Platform-Specific Notes

### Docker

```dockerfile
# Chrome in Docker needs extra flags
chrome --remote-debugging-port=9222 \
       --no-sandbox \
       --disable-gpu \
       --disable-dev-shm-usage
```

### WSL2 (Windows Subsystem for Linux)

```bash
# Chrome runs on Windows, connect from WSL
# Find Windows host IP
export WINDOWS_HOST=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')

# Connect to Windows Chrome
domguard --host $WINDOWS_HOST --port 9222 status
```

### CI/CD Environments

```yaml
# GitHub Actions example
- name: Start Chrome
  run: |
    google-chrome --remote-debugging-port=9222 --headless=new &
    sleep 2
    domguard status
```
