# Code Standards

Guidelines for contributing code to DOMGuard.

## Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

CI will fail if code is not formatted.

## Linting

Zero warnings policy with Clippy:

```bash
cargo clippy -- -D warnings
```

All Clippy warnings are treated as errors in CI.

## Testing

All tests must pass:

```bash
cargo test
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `get_dom_tree()` |
| Variables | snake_case | `node_id` |
| Types/Structs | PascalCase | `CdpConnection` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT` |
| Modules | snake_case | `session_recording` |

## Error Handling

Use `anyhow` for application errors:

```rust
use anyhow::{Result, Context};

fn connect() -> Result<Connection> {
    let conn = establish_connection()
        .context("Failed to connect to Chrome")?;
    Ok(conn)
}
```

## Documentation

Document public APIs:

```rust
/// Connects to Chrome DevTools Protocol.
///
/// # Arguments
///
/// * `host` - Chrome host address
/// * `port` - Chrome debugging port
///
/// # Returns
///
/// A `CdpConnection` on success.
///
/// # Errors
///
/// Returns an error if connection fails.
pub fn connect(host: &str, port: u16) -> Result<CdpConnection> {
    // ...
}
```

## Commit Messages

Follow Conventional Commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(interact): add triple-click support
fix(cdp): handle connection timeout
docs(readme): add workflow examples
```

## Pull Requests

### Before Opening

- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] Commit messages follow convention

### PR Description

```markdown
## Summary
Brief description of changes.

## Changes
- Change 1
- Change 2

## Testing
How to test the changes.

## Related Issues
Closes #123
```

## Code Review

PRs require:
1. CI passing
2. At least one approval
3. No merge conflicts

## Security

- Never commit credentials
- Use environment variables for secrets
- Validate all user input
- Sanitize output (credential masking)
