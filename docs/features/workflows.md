# Workflows

Create and run reusable automation scripts.

## Create Workflow

### From Session

```bash
domguard workflow create "login-flow" --from-session <session-id>
```

### From YAML File

```bash
domguard workflow create "checkout" --file workflow.yaml
```

## Workflow Format

```yaml
name: login-flow
description: Log into the application

steps:
  - navigate: "https://app.example.com/login"

  - wait: "input#email"

  - type:
      selector: "input#email"
      text: "user@example.com"

  - type:
      selector: "input#password"
      text: "${PASSWORD}"  # Environment variable

  - click: "button[type=submit]"

  - wait:
      text: "Welcome"
```

## Run Workflow

```bash
# Execute workflow
domguard workflow run "login-flow"

# Dry run (preview without executing)
domguard workflow run "login-flow" --dry-run
```

## Managing Workflows

### List Workflows

```bash
domguard workflow list
```

### View Workflow

```bash
domguard workflow show "login-flow"
```

### Delete Workflow

```bash
domguard workflow delete "login-flow"
```

## Variables

Use environment variables in workflows:

```yaml
steps:
  - type:
      selector: "input#api-key"
      text: "${API_KEY}"
```

Run with:
```bash
API_KEY=secret123 domguard workflow run "api-test"
```

## Workflow Steps

| Step | Example |
|------|---------|
| `navigate` | `navigate: "https://..."` |
| `click` | `click: "button.submit"` |
| `type` | `type: { selector: "input", text: "..." }` |
| `wait` | `wait: "div.loaded"` |
| `wait_text` | `wait: { text: "Success" }` |
| `screenshot` | `screenshot: "step1.png"` |
| `scroll` | `scroll: { down: 500 }` |

## Use Cases

1. **Login flows** - Reusable authentication
2. **Data entry** - Form filling automation
3. **Testing** - Repeatable test scenarios
4. **Scraping** - Multi-page data collection
