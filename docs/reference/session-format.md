# Session File Format

Sessions are recorded browser interactions saved as JSON files.

## File Location

Sessions are stored in `.domguard/sessions/` as `.json` files.

## Session Structure

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Login Test",
  "started_at": "2024-01-15T10:30:00Z",
  "ended_at": "2024-01-15T10:32:15Z",
  "initial_url": "https://example.com/login",
  "status": "completed",
  "metadata": {
    "description": "Test login flow",
    "tags": ["auth", "login"],
    "browser_version": "Chrome/120.0",
    "viewport": [1920, 1080]
  },
  "actions": [
    {
      "timestamp": "2024-01-15T10:30:01Z",
      "duration_ms": 150,
      "command": "navigate",
      "args": {"url": "https://example.com/login"},
      "status": "success",
      "page_url": "https://example.com/login"
    },
    {
      "timestamp": "2024-01-15T10:30:05Z",
      "duration_ms": 50,
      "command": "type",
      "args": {"value": "user@example.com"},
      "status": "success",
      "selector": "#email",
      "page_url": "https://example.com/login"
    }
  ]
}
```

## Session Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique session identifier (UUID) |
| `name` | string? | Optional session name |
| `started_at` | datetime | ISO 8601 timestamp when recording started |
| `ended_at` | datetime? | ISO 8601 timestamp when recording ended |
| `initial_url` | string? | URL when session started |
| `status` | enum | Session status (see below) |
| `metadata` | object | Additional session info |
| `actions` | array | List of recorded actions |

## Session Status

| Status | Description |
|--------|-------------|
| `recording` | Session is actively recording |
| `paused` | Recording is paused |
| `completed` | Session finished successfully |
| `failed` | Session ended with error |

## Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | string? | Session description |
| `tags` | array | Tags for categorization |
| `browser_version` | string? | Chrome version |
| `viewport` | [int, int]? | Viewport width and height |

## Action Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | datetime | When action was executed |
| `duration_ms` | int | Execution time in milliseconds |
| `command` | string | Action type (click, type, navigate, etc.) |
| `args` | object | Command arguments |
| `status` | enum | `success` or `failed` |
| `screenshot` | string? | Path to screenshot taken after action |
| `error` | string? | Error message if failed |
| `page_url` | string? | URL at time of action |
| `selector` | string? | CSS selector for element actions |

## Action Types

### Navigation

```json
{
  "command": "navigate",
  "args": {"url": "https://example.com"}
}
```

### Click

```json
{
  "command": "click",
  "args": {"coords": [100, 200], "nth": 0},
  "selector": "#button"
}
```

### Type

```json
{
  "command": "type",
  "args": {"value": "Hello World"},
  "selector": "#input"
}
```

### Key Press

```json
{
  "command": "key",
  "args": {"keys": "Enter"}
}
```

### Wait

```json
{
  "command": "wait",
  "args": {"text": "Success"},
  "selector": ".message"
}
```

### Screenshot

```json
{
  "command": "screenshot",
  "args": {},
  "screenshot": ".domguard/screenshots/screenshot_1705316415.png"
}
```

## Working with Sessions

### Recording

```bash
# Start recording
domguard session start --name "My Session"

# Perform actions (they get recorded)
domguard interact click "#button"
domguard interact type "#input" "hello"

# Stop recording
domguard session stop
```

### Managing Sessions

```bash
# List all sessions
domguard session list

# Show session details
domguard session show <session-id>

# Export to file
domguard session export <session-id> -o session.json

# Delete session
domguard session delete <session-id>
```

### Converting to Workflow

```bash
# Create workflow from session
domguard workflow create --from-session <session-id> --name "My Workflow"
```

## Example: Complete Session

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "Product Search",
  "started_at": "2024-01-15T14:00:00Z",
  "ended_at": "2024-01-15T14:02:30Z",
  "initial_url": "https://shop.example.com",
  "status": "completed",
  "metadata": {
    "description": "Search for products and add to cart",
    "tags": ["e-commerce", "search", "cart"],
    "browser_version": "Chrome/120.0.6099.109",
    "viewport": [1440, 900]
  },
  "actions": [
    {
      "timestamp": "2024-01-15T14:00:01Z",
      "duration_ms": 1250,
      "command": "navigate",
      "args": {"url": "https://shop.example.com"},
      "status": "success",
      "page_url": "https://shop.example.com"
    },
    {
      "timestamp": "2024-01-15T14:00:05Z",
      "duration_ms": 45,
      "command": "type",
      "args": {"value": "laptop"},
      "status": "success",
      "selector": "#search-input",
      "page_url": "https://shop.example.com"
    },
    {
      "timestamp": "2024-01-15T14:00:06Z",
      "duration_ms": 30,
      "command": "key",
      "args": {"keys": "Enter"},
      "status": "success",
      "page_url": "https://shop.example.com"
    },
    {
      "timestamp": "2024-01-15T14:00:10Z",
      "duration_ms": 2500,
      "command": "wait",
      "args": {},
      "status": "success",
      "selector": ".product-grid",
      "page_url": "https://shop.example.com/search?q=laptop"
    },
    {
      "timestamp": "2024-01-15T14:00:15Z",
      "duration_ms": 150,
      "command": "click",
      "args": {"nth": 0},
      "status": "success",
      "selector": ".product-card .add-to-cart",
      "page_url": "https://shop.example.com/search?q=laptop"
    },
    {
      "timestamp": "2024-01-15T14:00:18Z",
      "duration_ms": 1000,
      "command": "wait",
      "args": {"text": "Added to cart"},
      "status": "success",
      "page_url": "https://shop.example.com/search?q=laptop"
    },
    {
      "timestamp": "2024-01-15T14:00:20Z",
      "duration_ms": 200,
      "command": "screenshot",
      "args": {},
      "status": "success",
      "screenshot": ".domguard/screenshots/screenshot_1705327220.png",
      "page_url": "https://shop.example.com/search?q=laptop"
    }
  ]
}
```
