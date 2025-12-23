# Security Features

Safety features for browser automation.

## Security Check

```bash
# Full security scan
domguard security check

# CAPTCHA detection only
domguard security check --captcha

# Sensitive field detection
domguard security check --sensitive
```

### CAPTCHA Detection

Detects common CAPTCHA providers:
- reCAPTCHA (v2 and v3)
- hCaptcha
- Cloudflare Turnstile
- FunCaptcha
- Custom image CAPTCHAs

### Sensitive Field Detection

Identifies:
- Password fields
- Credit card inputs
- SSN/ID fields
- API key inputs

## Blocked Sites

Prevent automation on specific sites:

```bash
# Block a site
domguard security block "phishing-site.com"

# Unblock a site
domguard security unblock "safe-site.com"

# List blocked sites
domguard security blocked
```

## User Takeover

Hand control back to a human when needed:

```bash
# Request human intervention
domguard takeover request
domguard takeover request --reason "CAPTCHA detected"

# Check takeover status
domguard takeover status

# Signal completion (human done)
domguard takeover done
```

### When to Use Takeover

1. **CAPTCHA encountered** - Human solves CAPTCHA
2. **2FA required** - Human enters code
3. **Complex interaction** - Human handles edge case
4. **Verification needed** - Human confirms action

## Self-Correction

Automatic error recovery:

```bash
# Configure correction behavior
domguard correction config --max-retries 3
domguard correction config --strategy "adaptive"

# View settings
domguard correction config --show

# Analyze page for potential issues
domguard correction analyze

# Manual retry with correction
domguard correction retry
```

### Correction Strategies

| Strategy | Description |
|----------|-------------|
| `simple` | Retry with same action |
| `adaptive` | Adjust timing and selectors |
| `fallback` | Try alternative approaches |

## Per-Site Instructions

Configure site-specific behaviors:

```bash
# Create site config
domguard sites create "example.com" --instructions "Always click cookie accept"

# List configured sites
domguard sites list

# View site config
domguard sites show "example.com"

# Update instructions
domguard sites update "example.com" --instructions "New behavior"

# Delete site config
domguard sites delete "example.com"
```

## Credential Masking

```bash
# Enable masking in output
domguard security mask --enable

# Disable masking
domguard security mask --disable
```

When enabled, sensitive data is masked:
- Passwords: `********`
- API keys: `sk-****...****`
- Tokens: `[MASKED]`

## Best Practices

1. **Always check security** before login/payment forms
2. **Use takeover** for CAPTCHAs instead of bypassing
3. **Block known bad sites** proactively
4. **Enable masking** when logging is visible
5. **Configure per-site rules** for consistent behavior
