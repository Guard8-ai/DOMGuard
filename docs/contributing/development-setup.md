# Development Setup

Set up your development environment for contributing to DOMGuard.

## Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **Chrome/Chromium** - For testing
- **Git** - For version control

## Clone Repository

```bash
git clone https://github.com/Guard8-ai/DOMGuard.git
cd DOMGuard
```

## Build

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Run Tests

```bash
cargo test
```

## Run Locally

```bash
# Start Chrome with debugging
chrome --remote-debugging-port=9222

# Run from source
cargo run -- status
cargo run -- debug dom
cargo run -- interact screenshot
```

## Code Quality

Before committing, run:

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# All checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Project Structure

```
DOMGuard/
├── src/
│   ├── main.rs         # CLI entry point
│   ├── cdp.rs          # Chrome DevTools Protocol
│   ├── debug.rs        # Debug commands
│   ├── interact.rs     # Interact commands
│   ├── session.rs      # Session recording
│   ├── workflow.rs     # Workflow management
│   ├── security.rs     # Security features
│   ├── captcha.rs      # CAPTCHA detection
│   ├── correction.rs   # Self-correction
│   ├── takeover.rs     # User takeover
│   ├── inspire.rs      # Design extraction
│   ├── config.rs       # Configuration
│   └── output.rs       # Output formatting
├── docs/               # Documentation
├── tasks/              # TaskGuard tasks
├── .github/workflows/  # CI/CD
├── Cargo.toml          # Dependencies
└── README.md           # Project overview
```

## IDE Setup

### VS Code

Recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML

### IntelliJ/CLion

Install the Rust plugin.

## Debugging

### VS Code launch.json

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug domguard",
      "cargo": {
        "args": ["build", "--bin=domguard"],
        "filter": {
          "name": "domguard",
          "kind": "bin"
        }
      },
      "args": ["status"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Making Changes

1. Create a branch: `git checkout -b feature/my-feature`
2. Make changes
3. Run checks: `cargo fmt && cargo clippy && cargo test`
4. Commit: `git commit -m "feat: add my feature"`
5. Push: `git push origin feature/my-feature`
6. Open Pull Request
