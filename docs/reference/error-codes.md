# Error Codes Reference

DOMGuard error messages and their solutions.

## Connection Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Cannot connect to Chrome on 127.0.0.1:9222` | Chrome not running with debugging enabled | Start Chrome with `--remote-debugging-port=9222` |
| `Connection refused` | Chrome closed or wrong port | Verify Chrome is running and port is correct |
| `Connection timeout` | Chrome unresponsive | Restart Chrome with debugging flag |
| `WebSocket handshake failed` | Protocol mismatch | Update Chrome to latest version |

## Selector Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `No element matches selector` | Element doesn't exist or wrong selector | Use `debug dom` to verify selector exists |
| `Multiple elements match selector` | Ambiguous selector | Add more specificity or use `--nth` flag |
| `Selector syntax error` | Invalid CSS selector | Use standard CSS selectors only (not Playwright/XPath) |
| `Element not visible` | Element exists but hidden | Wait for element with `--visible` flag |
| `Element not interactable` | Element covered or disabled | Use `debug dom` to check element state |

## Timeout Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Timeout waiting for element` | Element never appeared | Increase `--timeout` or check if element loads |
| `Timeout waiting for navigation` | Page load too slow | Increase timeout or check network |
| `Command timeout exceeded` | Operation took too long | Use `--timeout` flag with higher value |

## File Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `File not found` | Upload file doesn't exist | Check file path is correct |
| `Permission denied` | Can't write screenshot/PDF | Check directory permissions |
| `Invalid file format` | Unsupported file type | Use supported formats (png, pdf) |

## Session Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `No active session` | Trying to stop/pause without start | Use `session start` first |
| `Session already active` | Starting while recording | Stop current session first |
| `Session not found` | Invalid session ID | Use `session list` to find valid IDs |

## Security Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Site is blocked` | URL matches blocked pattern | Use `security unblock` or check config |
| `Sensitive action detected` | Password/payment field detected | Confirm action or use takeover mode |
| `CAPTCHA detected` | Page has CAPTCHA challenge | Use `takeover request captcha` |

## Workflow Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Workflow not found` | Invalid workflow ID | Use `workflow list` to find valid IDs |
| `Invalid workflow syntax` | TOML/YAML parse error | Check workflow file syntax |
| `Missing required parameter` | Workflow parameter not provided | Pass required `--param` values |

## CDP Protocol Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Target closed` | Tab was closed during operation | Ensure tab stays open |
| `Execution context destroyed` | Page navigated during JS eval | Wait for navigation to complete |
| `Object reference not found` | Stale element reference | Re-query the element |

## Common Solutions

### Chrome Won't Connect

```bash
# 1. Kill existing Chrome instances
pkill -f chrome

# 2. Start Chrome with debugging
chrome --remote-debugging-port=9222

# 3. Verify connection
domguard status
```

### Element Not Found

```bash
# 1. Check if element exists
domguard debug dom ".my-selector"

# 2. Wait for element to appear
domguard interact wait ".my-selector" --visible

# 3. Try alternative selectors
domguard debug aria  # Find by accessibility tree
```

### Timeouts

```bash
# Increase global timeout
domguard --timeout 30000 interact click ".slow-button"

# Or set in config
# .domguard/config.toml
[defaults]
timeout_ms = 30000
```
