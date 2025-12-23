# Installation

## From crates.io (Recommended)

```bash
cargo install domguard
```

## From Source

```bash
git clone https://github.com/Guard8-ai/DOMGuard.git
cd DOMGuard
cargo build --release
```

The binary will be at `target/release/domguard`.

## Pre-built Binaries

Download from [GitHub Releases](https://github.com/Guard8-ai/DOMGuard/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `domguard-linux-x86_64` |
| macOS ARM64 | `domguard-macos-aarch64` |
| Windows x86_64 | `domguard-windows-x86_64.exe` |

### Linux/macOS

```bash
# Download
curl -LO https://github.com/Guard8-ai/DOMGuard/releases/latest/download/domguard-linux-x86_64

# Make executable
chmod +x domguard-linux-x86_64

# Move to PATH
sudo mv domguard-linux-x86_64 /usr/local/bin/domguard
```

### Windows

Download `domguard-windows-x86_64.exe` and add to your PATH.

## Initialize Project

```bash
domguard init
```

This creates `.domguard/` with:

- `config.toml` - Configuration settings
- `AGENTIC_AI_GUIDE.md` - AI agent quick reference

## Verify Installation

```bash
domguard --version
domguard status
```
