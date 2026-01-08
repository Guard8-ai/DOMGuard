# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | :white_check_mark: |
| < 0.4   | :x:                |

## Security Model

DOMGuard is designed with a **local-first, privacy-first** security model:

### Default Configuration (Secure)

By default, DOMGuard only connects to **localhost** Chrome instances:

```toml
[chrome]
host = "127.0.0.1"  # localhost only
port = 9222
```

This means:
- No data leaves your machine
- No network exposure
- No authentication required (Chrome trusts localhost)

### Remote Connections (Use with Caution)

DOMGuard supports connecting to remote Chrome instances, but this introduces security risks:

```toml
[chrome]
host = "192.168.1.100"  # Remote connection - USE WITH CAUTION
port = 9222
```

**Risks of remote connections:**

| Risk | Description | Mitigation |
|------|-------------|------------|
| **No encryption** | CDP uses WebSocket without TLS | Use SSH tunnel or VPN |
| **No authentication** | Chrome DevTools has no built-in auth | Firewall + network isolation |
| **Command injection** | Remote attacker could execute arbitrary JS | Restrict to trusted networks |
| **Data exposure** | Page content transmitted in cleartext | Encrypt network layer |

**Recommended setup for remote debugging:**

```bash
# Option 1: SSH tunnel (recommended)
ssh -L 9222:localhost:9222 user@remote-host
# Then connect to localhost:9222

# Option 2: VPN
# Ensure Chrome is only listening on VPN interface

# Option 3: Firewall rules
# Restrict port 9222 to specific IPs
iptables -A INPUT -p tcp --dport 9222 -s 192.168.1.0/24 -j ACCEPT
iptables -A INPUT -p tcp --dport 9222 -j DROP
```

## Security Features

### Credential Masking

DOMGuard automatically masks sensitive data in output:

```bash
# Input fields with type="password" are masked
domguard interact type "#password" "secret123"
# Output shows: Typed ******** into #password
```

### CAPTCHA Detection

DOMGuard detects CAPTCHAs and warns before automation:

- reCAPTCHA v2/v3
- hCaptcha
- Cloudflare Turnstile
- Generic CAPTCHA patterns

### Blocked Sites

Configure sites that should never be automated:

```toml
# .domguard/blocked_sites.toml
[[sites]]
pattern = "bank.example.com"
reason = "Financial institution - manual access only"
```

### Input Validation

- CSS selectors are validated before execution
- Coordinates are bounds-checked
- URLs are validated for format
- File paths are sanitized

## Best Practices

### For Local Development

1. Always use the default localhost configuration
2. Start Chrome with `--remote-debugging-port=9222`
3. Don't expose port 9222 to the network

### For CI/CD Environments

1. Use headless Chrome in isolated containers
2. Don't persist session data between runs
3. Use ephemeral browser profiles
4. Rotate any stored credentials

### For Production Automation

1. Never automate financial or sensitive sites
2. Respect robots.txt and terms of service
3. Implement rate limiting
4. Log all automation actions
5. Review automation scripts before deployment

## Reporting a Vulnerability

If you discover a security vulnerability in DOMGuard:

1. **Do not** open a public GitHub issue
2. Email: security@guard8.ai
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to address the issue.

## Security Checklist

Before going to production, verify:

- [ ] Chrome is only accessible from localhost (or properly tunneled)
- [ ] No sensitive credentials are hardcoded in workflows
- [ ] Session recordings don't contain sensitive data
- [ ] Blocked sites list includes all sensitive domains
- [ ] Network traffic is encrypted (SSH tunnel or VPN for remote)
- [ ] Automation logs are stored securely
- [ ] Access to DOMGuard is restricted to authorized users

## Changelog

### 0.4.2
- Added proper error handling for path operations
- Enhanced credential masking in output
- Added remote connection security warnings to config

### 0.4.0
- Initial security documentation
- Localhost-only default configuration
- CAPTCHA detection
- Blocked sites feature
