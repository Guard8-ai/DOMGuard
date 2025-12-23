# Contributing to DOMGuard

Thank you for your interest in contributing to DOMGuard!

## Development Setup

### Prerequisites

- Rust 1.70+
- Chrome/Chromium browser
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/Guard8-ai/DOMGuard.git
cd DOMGuard

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- status
```

## Code Quality

Before submitting a PR, ensure:

```bash
# Format code
cargo fmt

# Lint (zero warnings required)
cargo clippy -- -D warnings

# Run tests
cargo test
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/my-feature`
3. **Make changes** following our code standards
4. **Test** your changes thoroughly
5. **Commit** using conventional commits:
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation
   - `refactor:` Code restructuring
   - `test:` Adding tests
6. **Push**: `git push origin feature/my-feature`
7. **Open a Pull Request**

### PR Requirements

- [ ] CI passes (build, test, clippy, fmt)
- [ ] Code is formatted with `cargo fmt`
- [ ] No Clippy warnings
- [ ] Tests pass
- [ ] Commit messages follow convention

## Reporting Issues

### Bug Reports

Include:
- DOMGuard version (`domguard --version`)
- OS and Chrome version
- Steps to reproduce
- Expected vs actual behavior
- Error messages

### Feature Requests

Describe:
- The problem you're trying to solve
- Your proposed solution
- Alternative approaches considered

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Open an issue or discussion on GitHub.

---

Built with [TaskGuard](https://github.com/Guard8-ai/TaskGuard)
