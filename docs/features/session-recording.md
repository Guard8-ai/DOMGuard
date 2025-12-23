# Session Recording

Capture browser actions for replay and analysis.

## Start Recording

```bash
# Begin new session
domguard session start

# Named session
domguard session start --name "checkout-flow"
```

## Check Status

```bash
domguard session status
```

Shows:
- Session ID
- Start time
- Number of actions recorded

## Stop Recording

```bash
domguard session stop
```

Saves the session to `.domguard/sessions/`.

## Managing Sessions

### List Sessions

```bash
domguard session list
```

### View Session Details

```bash
domguard session show <session-id>
```

Shows step-by-step action log with:
- Timestamp
- Action type
- Target element
- Parameters

### Export Session

```bash
domguard session export <session-id> -o session.json
```

### Delete Session

```bash
domguard session delete <session-id>
```

## Session Format

Sessions are stored as JSON with:

```json
{
  "id": "session-abc123",
  "name": "checkout-flow",
  "created": "2025-01-15T10:30:00Z",
  "actions": [
    {
      "timestamp": "2025-01-15T10:30:01Z",
      "type": "navigate",
      "url": "https://shop.example.com"
    },
    {
      "timestamp": "2025-01-15T10:30:05Z",
      "type": "click",
      "selector": "button.add-to-cart"
    },
    {
      "timestamp": "2025-01-15T10:30:08Z",
      "type": "type",
      "selector": "input#email",
      "text": "user@example.com"
    }
  ]
}
```

## Use Cases

1. **Debugging** - Replay issues step-by-step
2. **Testing** - Create test cases from manual exploration
3. **Documentation** - Record workflows for training
4. **Automation** - Convert sessions to reusable workflows
