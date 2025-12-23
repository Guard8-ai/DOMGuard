# Prerequisites

## Required

### Rust 1.70+

DOMGuard is built with Rust. Install via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version
# rustc 1.70.0 or higher
```

### Chrome/Chromium

Any Chromium-based browser with remote debugging support:

- Google Chrome
- Chromium
- Microsoft Edge
- Brave

## Starting Chrome with Debugging

Launch Chrome with the remote debugging port enabled:

=== "Linux"
    ```bash
    google-chrome --remote-debugging-port=9222
    # or
    chromium --remote-debugging-port=9222
    ```

=== "macOS"
    ```bash
    /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222
    ```

=== "Windows"
    ```powershell
    "C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
    ```

!!! tip "Headless Mode"
    For server environments, add `--headless=new`:
    ```bash
    chrome --remote-debugging-port=9222 --headless=new
    ```

## Verify Setup

After installing DOMGuard and starting Chrome:

```bash
domguard status
```

Expected output:
```
✓ Connected to Chrome at 127.0.0.1:9222
  Browser: Chrome/120.0.0.0
  Target: https://example.com
```
