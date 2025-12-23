# JSON Output

Use `--json` flag for programmatic output.

## Usage

```bash
domguard --json <command>
```

## Examples

### Status

```bash
domguard --json status
```

```json
{
  "connected": true,
  "host": "127.0.0.1",
  "port": 9222,
  "browser": "Chrome/120.0.0.0",
  "target": {
    "id": "ABC123",
    "type": "page",
    "url": "https://example.com",
    "title": "Example Domain"
  }
}
```

### DOM

```bash
domguard --json debug dom "body"
```

```json
{
  "nodeId": 1,
  "nodeName": "BODY",
  "children": [
    {
      "nodeId": 2,
      "nodeName": "DIV",
      "attributes": {
        "class": "container",
        "id": "main"
      },
      "children": []
    }
  ]
}
```

### Screenshot

```bash
domguard --json interact screenshot
```

```json
{
  "success": true,
  "path": ".domguard/screenshots/screenshot_1234567890.png",
  "width": 1920,
  "height": 1080,
  "format": "png"
}
```

### Click

```bash
domguard --json interact click "button.submit"
```

```json
{
  "success": true,
  "action": "click",
  "selector": "button.submit",
  "element": {
    "tagName": "BUTTON",
    "className": "submit",
    "textContent": "Submit"
  }
}
```

### Performance

```bash
domguard --json debug performance
```

```json
{
  "timing": {
    "domContentLoaded": 245,
    "load": 1200,
    "firstByte": 89
  },
  "webVitals": {
    "lcp": 1100,
    "fid": 12,
    "cls": 0.05
  },
  "memory": {
    "jsHeapSize": 12400000,
    "domNodes": 1247
  },
  "resources": {
    "requestCount": 45,
    "totalSize": 2300000
  }
}
```

### Errors

```bash
domguard --json interact click "nonexistent"
```

```json
{
  "success": false,
  "error": {
    "code": "ELEMENT_NOT_FOUND",
    "message": "No element found matching selector: nonexistent",
    "selector": "nonexistent"
  }
}
```

## Parsing in Scripts

### Bash + jq

```bash
# Get page title
domguard --json status | jq -r '.target.title'

# Check if element exists
domguard --json debug dom "button.submit" | jq '.nodeId != null'

# Get screenshot path
path=$(domguard --json interact screenshot | jq -r '.path')
```

### Python

```python
import subprocess
import json

result = subprocess.run(
    ["domguard", "--json", "status"],
    capture_output=True,
    text=True
)
data = json.loads(result.stdout)
print(f"Connected: {data['connected']}")
print(f"URL: {data['target']['url']}")
```

### Node.js

```javascript
const { execSync } = require('child_process');

const output = execSync('domguard --json status').toString();
const data = JSON.parse(output);
console.log(`Connected: ${data.connected}`);
console.log(`URL: ${data.target.url}`);
```
