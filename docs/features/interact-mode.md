# Interact Mode

Control the browser programmatically.

## Mouse Actions

### Click

```bash
# Click by selector
domguard interact click "button.submit"
domguard interact click "#login-btn"

# Click by coordinates
domguard interact click --coords 100,200
```

### Hover

```bash
domguard interact hover "div.menu"
```

### Triple-click (Select Paragraph)

```bash
domguard interact triple-click "p.content"
domguard interact triple-click --coords 100,200
```

### Advanced Mouse Control

```bash
# Move cursor without clicking
domguard interact mouse-move 100,200

# Get current cursor position
domguard interact cursor-position

# Press/release mouse button
domguard interact mouse-down left
domguard interact mouse-up left

# Drag and drop
domguard interact drag --from "#source" --to "#target"
```

## Keyboard Actions

### Type Text

```bash
# Type into element
domguard interact type "input.search" "hello world"

# Type to focused element
domguard interact type --focused "hello world"
```

### Press Keys

```bash
# Single key
domguard interact key "Enter"
domguard interact key "Tab"
domguard interact key "Escape"

# Key combinations
domguard interact key "ctrl+a"
domguard interact key "ctrl+c"
domguard interact key "ctrl+a ctrl+c"
```

### Hold Key

```bash
# Hold for duration (ms)
domguard interact hold-key Shift --duration 1000
```

## Navigation

```bash
# Go to URL
domguard interact navigate "https://example.com"

# Back/forward
domguard interact back
domguard interact forward

# Refresh
domguard interact refresh
```

## Scrolling

```bash
# Scroll by pixels
domguard interact scroll --down 500
domguard interact scroll --up 200

# Scroll to element
domguard interact scroll --to "footer"
domguard interact scroll --to "#comments"
```

## Screenshots & PDF

```bash
# Viewport screenshot
domguard interact screenshot

# Full page
domguard interact screenshot --full

# Specific element
domguard interact screenshot --element "div.hero"

# Custom output
domguard interact screenshot -o myshot.png

# Screenshot region (x,y,width,height)
domguard interact screenshot-region 0,0,800,600
domguard interact screenshot-region 100,100,400,300 -o crop.png

# PDF export
domguard interact pdf -o page.pdf
domguard interact pdf --landscape
```

## Forms

### Select Dropdown

```bash
# By value
domguard interact select "select#country" "US"

# By label text
domguard interact select "select#country" "United States" --by-label
```

### File Upload

```bash
domguard interact upload "input[type=file]" ./document.pdf
```

### Dialog Handling

```bash
# Accept alert/confirm
domguard interact dialog --accept

# Dismiss
domguard interact dialog --dismiss

# Prompt with text
domguard interact dialog --accept --text "my response"
```

## Wait Conditions

```bash
# Wait for element to appear
domguard interact wait "div.loaded"

# Wait for element to disappear
domguard interact wait "div.spinner" --gone

# Wait for text to appear
domguard interact wait --text "Success"

# Wait for text to disappear
domguard interact wait --text-gone "Loading..."

# Wait fixed duration (ms)
domguard interact wait-duration 2000
```

## Viewport

```bash
# Resize viewport
domguard interact resize 1920 1080
domguard interact resize 375 667  # Mobile
```
