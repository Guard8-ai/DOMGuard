# Performance Features

Monitor and test performance characteristics.

## Performance Metrics

```bash
domguard debug performance
```

Returns:
- **Core Web Vitals**: LCP, FID, CLS
- **Timing**: DOM load, full load, TTFB
- **Memory**: JS heap size, DOM nodes
- **Resources**: Total size, request count

Example output:
```
Performance Metrics
───────────────────
Timing:
  DOM Content Loaded: 245ms
  Full Load: 1.2s
  Time to First Byte: 89ms

Core Web Vitals:
  Largest Contentful Paint: 1.1s
  First Input Delay: 12ms
  Cumulative Layout Shift: 0.05

Memory:
  JS Heap Size: 12.4 MB
  DOM Nodes: 1,247

Resources:
  Total Requests: 45
  Total Size: 2.3 MB
```

## CPU Throttling

Simulate slower CPUs:

```bash
# 4x slowdown
domguard debug throttle-cpu 4

# 6x slowdown (mobile-like)
domguard debug throttle-cpu 6

# Disable throttling
domguard debug throttle-cpu --disable
```

Use cases:
- Test on low-end devices
- Identify performance bottlenecks
- Verify graceful degradation

## Network Throttling

Simulate network conditions:

```bash
# Presets
domguard debug throttle-network slow-3g   # 500ms latency, 500 Kbps
domguard debug throttle-network 3g        # 100ms latency, 1.5 Mbps
domguard debug throttle-network offline   # No connection

# Disable
domguard debug throttle-network --disable
```

### Custom Throttling

```bash
domguard debug throttle-network custom --latency 200 --download 1000 --upload 500
```

Parameters:
- `--latency`: Round-trip time in ms
- `--download`: Download speed in Kbps
- `--upload`: Upload speed in Kbps

## DOM Snapshot

Export full page state:

```bash
domguard debug snapshot -o page.html
```

Captures:
- Complete DOM tree
- Inline styles
- Current state (form values, etc.)

## Network Monitoring

```bash
# All requests
domguard debug network

# Filter by URL pattern
domguard debug network --filter "api"
domguard debug network --filter ".js"
```

Shows:
- Request URL
- Method (GET, POST, etc.)
- Status code
- Response time
- Size

## Testing Workflow

1. **Baseline**: Capture metrics on fast connection
2. **Throttle**: Apply CPU/network throttling
3. **Measure**: Compare metrics
4. **Optimize**: Fix issues found
5. **Verify**: Re-test with throttling

```bash
# Baseline
domguard debug performance > baseline.txt

# Throttled test
domguard debug throttle-cpu 4
domguard debug throttle-network 3g
domguard debug performance > throttled.txt

# Cleanup
domguard debug throttle-cpu --disable
domguard debug throttle-network --disable
```
