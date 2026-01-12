# Workflow Syntax Reference

Workflows are reusable automation scripts saved as TOML files.

## File Location

Workflows are stored in `.domguard/workflows/` as `.toml` files.

## Basic Structure

```toml
id = "login-flow"
name = "Login Flow"
description = "Automate login process"
domain = "example.com"
tags = ["auth", "login"]

[[parameters]]
name = "username"
description = "Login username"
required = true
param_type = "text"

[[parameters]]
name = "password"
description = "Login password"
required = true
param_type = "password"

[[steps]]
name = "Navigate to login"
action = "navigate"
target = "https://example.com/login"

[[steps]]
name = "Enter username"
action = "type"
target = "#username"
value = "{{username}}"

[[steps]]
name = "Enter password"
action = "type"
target = "#password"
value = "{{password}}"

[[steps]]
name = "Click submit"
action = "click"
target = "[type=submit]"

[[steps]]
name = "Wait for dashboard"
action = "wait"
target = ".dashboard"
timeout_ms = 10000
```

## Workflow Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier |
| `name` | string | Yes | Human-readable name |
| `description` | string | No | What the workflow does |
| `domain` | string | No | Domain this applies to |
| `tags` | array | No | Tags for organization |
| `parameters` | array | No | Input parameters |
| `steps` | array | Yes | Steps to execute |

## Parameter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Parameter name (used as `{{name}}`) |
| `description` | string | No | Parameter description |
| `default` | string | No | Default value if not provided |
| `required` | bool | No | Whether parameter is required |
| `param_type` | string | No | Type hint: `text`, `password`, `url`, `number`, `file` |

## Step Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | No | Step name for display |
| `action` | string | Yes | Action type (see below) |
| `target` | string | No | CSS selector or URL |
| `value` | string | No | Value for type/input actions |
| `timeout_ms` | int | No | Step timeout in milliseconds |
| `required` | bool | No | Stop workflow on failure (default: true) |
| `retry_count` | int | No | Number of retries on failure |

## Supported Actions

### Navigation
```toml
[[steps]]
action = "navigate"
target = "https://example.com"

[[steps]]
action = "back"

[[steps]]
action = "refresh"
```

### Clicking
```toml
[[steps]]
action = "click"
target = "#button"

[[steps]]
action = "click"
target = "button"
value = "2"  # nth element (0-indexed)
```

### Typing
```toml
[[steps]]
action = "type"
target = "#input"
value = "Hello World"

[[steps]]
action = "type"
value = "text"  # Type into focused element
```

### Keyboard
```toml
[[steps]]
action = "key"
value = "Enter"

[[steps]]
action = "key"
value = "cmd+k"  # Shortcuts
```

### Waiting
```toml
[[steps]]
action = "wait"
target = ".loading"
value = "gone"  # Wait for removal

[[steps]]
action = "wait"
target = "#content"
value = "visible"

[[steps]]
action = "wait"
value = "2000"  # Wait milliseconds
```

### Screenshots
```toml
[[steps]]
action = "screenshot"
target = "full"  # or "viewport"
value = "screenshot.png"

[[steps]]
action = "screenshot"
target = "#element"  # Element screenshot
```

### Select/Dropdown
```toml
[[steps]]
action = "select"
target = "#country"
value = "US"
```

### Scrolling
```toml
[[steps]]
action = "scroll"
value = "500"  # Down 500px

[[steps]]
action = "scroll"
target = ".footer"  # Scroll to element
```

## Parameter Substitution

Use `{{param_name}}` syntax to insert parameters:

```toml
[[parameters]]
name = "search_query"
default = "domguard"

[[steps]]
action = "type"
target = "#search"
value = "{{search_query}}"
```

## Running Workflows

```bash
# Run with default parameters
domguard workflow run login-flow

# Pass parameters
domguard workflow run login-flow --param username=john --param password=secret

# Dry run (show steps without executing)
domguard workflow run login-flow --dry-run
```

## Creating Workflows

```bash
# Create from session recording
domguard workflow create --from-session <session-id> --name "My Workflow"

# Create empty template
domguard workflow create --name "My Workflow"

# List workflows
domguard workflow list

# Show workflow details
domguard workflow show <workflow-id>
```

## Example: Form Submission

```toml
id = "submit-form"
name = "Submit Contact Form"
description = "Fill and submit the contact form"
domain = "example.com"
tags = ["form", "contact"]

[[parameters]]
name = "name"
required = true

[[parameters]]
name = "email"
required = true
param_type = "text"

[[parameters]]
name = "message"
default = "Hello from DOMGuard!"

[[steps]]
name = "Go to contact page"
action = "navigate"
target = "https://example.com/contact"

[[steps]]
name = "Fill name"
action = "type"
target = "#name"
value = "{{name}}"

[[steps]]
name = "Fill email"
action = "type"
target = "#email"
value = "{{email}}"

[[steps]]
name = "Fill message"
action = "type"
target = "#message"
value = "{{message}}"

[[steps]]
name = "Submit"
action = "click"
target = "[type=submit]"

[[steps]]
name = "Verify success"
action = "wait"
target = ".success-message"
timeout_ms = 5000
```
