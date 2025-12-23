# Debug Mode

Inspect page state without modifying it.

## DOM Inspection

```bash
# Full DOM tree
domguard debug dom

# Specific element
domguard debug dom "div.container"
domguard debug dom "#main-content"
```

## Accessibility Tree (ARIA)

```bash
# Full ARIA tree
domguard debug aria

# Specific element
domguard debug aria "nav"
```

!!! tip "For AI Agents"
    The ARIA tree is often more useful than raw DOM for understanding page structure and finding interactive elements.

## Console Messages

```bash
# View all messages
domguard debug console

# Stream live
domguard debug console --follow

# Filter by type
domguard debug console --filter "error"
domguard debug console --filter "warning"
```

## Network Requests

```bash
# All requests
domguard debug network

# Filter by URL
domguard debug network --filter "api"
domguard debug network --filter ".json"
```

## Storage

```bash
# localStorage and sessionStorage
domguard debug storage

# Cookies
domguard debug cookies
```

## Styles

```bash
# Computed styles for element
domguard debug styles "button.primary"
```

## JavaScript Evaluation

```bash
# Execute JS and get result
domguard debug eval "document.title"
domguard debug eval "window.location.href"
domguard debug eval "document.querySelectorAll('a').length"
```

## Tab Management

```bash
# List all tabs
domguard debug tabs list

# Create new tab
domguard debug tabs new "https://example.com"

# Switch to tab
domguard debug tabs switch <tab-id>

# Close tab
domguard debug tabs close <tab-id>
```

## Performance Metrics

```bash
# Core Web Vitals, timing, heap size
domguard debug performance
```

## Throttling

```bash
# CPU throttling (4x slowdown)
domguard debug throttle-cpu 4
domguard debug throttle-cpu --disable

# Network throttling
domguard debug throttle-network slow-3g
domguard debug throttle-network 3g
domguard debug throttle-network offline
domguard debug throttle-network --disable
```

## DOM Snapshot

```bash
# Export full DOM to file
domguard debug snapshot -o page.html
```
