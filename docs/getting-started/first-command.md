# First Command

A quick tutorial to get started with DOMGuard.

## 1. Start Chrome

```bash
chrome --remote-debugging-port=9222
```

Navigate to any website (e.g., https://example.com).

## 2. Check Connection

```bash
domguard status
```

You should see:
```
✓ Connected to Chrome at 127.0.0.1:9222
```

## 3. Inspect the Page

### View DOM Tree

```bash
domguard debug dom
```

### View Accessibility Tree

```bash
domguard debug aria
```

This is especially useful for AI agents to understand page structure.

### View Console Messages

```bash
domguard debug console
```

## 4. Interact with the Page

### Take a Screenshot

```bash
domguard interact screenshot
```

Saves to `.domguard/screenshots/`.

### Click an Element

```bash
domguard interact click "a"
```

### Type Text

```bash
domguard interact type "input[type=search]" "hello world"
```

### Navigate

```bash
domguard interact navigate "https://github.com"
```

## 5. Use JSON Output

For programmatic use, add `--json`:

```bash
domguard --json debug dom
domguard --json interact screenshot
```

## Next Steps

- [Debug Mode](../features/debug-mode.md) - Full inspection capabilities
- [Interact Mode](../features/interact-mode.md) - Browser control
- [Commands Reference](../api-reference/commands.md) - All commands
