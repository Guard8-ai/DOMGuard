//! Custom site instructions for DOMGuard
//!
//! Allows defining per-site behaviors, custom selectors, and automation rules.
//! Instructions are stored in `.domguard/sites/` directory.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Site-specific instructions and behaviors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiteInstructions {
    /// Domain pattern (e.g., "example.com", "*.example.com")
    pub domain: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Login selectors and instructions
    #[serde(default)]
    pub login: Option<LoginInstructions>,

    /// Custom element selectors for common actions
    #[serde(default)]
    pub selectors: HashMap<String, String>,

    /// Pre-action hooks (run before specific actions)
    #[serde(default)]
    pub before_actions: HashMap<String, Vec<ActionStep>>,

    /// Post-action hooks (run after specific actions)
    #[serde(default)]
    pub after_actions: HashMap<String, Vec<ActionStep>>,

    /// Wait conditions before automation
    #[serde(default)]
    pub wait_ready: Option<WaitCondition>,

    /// Cookie consent handling
    #[serde(default)]
    pub cookie_consent: Option<CookieConsentConfig>,

    /// CAPTCHA handling preferences
    #[serde(default)]
    pub captcha: Option<CaptchaConfig>,

    /// Custom timeouts for this site
    #[serde(default)]
    pub timeouts: Option<TimeoutConfig>,

    /// Notes and tips for automation
    #[serde(default)]
    pub notes: Vec<String>,
}

/// Login form instructions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoginInstructions {
    /// Login page URL path
    pub url: Option<String>,

    /// Username/email field selector
    pub username_selector: Option<String>,

    /// Password field selector
    pub password_selector: Option<String>,

    /// Submit button selector
    pub submit_selector: Option<String>,

    /// How to detect successful login
    pub success_indicator: Option<String>,

    /// How to detect failed login
    pub failure_indicator: Option<String>,

    /// Additional steps before/after login
    #[serde(default)]
    pub extra_steps: Vec<ActionStep>,
}

/// Action step for custom workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    /// Action type: click, type, wait, etc.
    pub action: String,

    /// Target selector or value
    #[serde(default)]
    pub target: Option<String>,

    /// Value for type actions
    #[serde(default)]
    pub value: Option<String>,

    /// Timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,

    /// Whether this step is optional (won't fail if element not found)
    #[serde(default)]
    pub optional: bool,

    /// Description for logging
    #[serde(default)]
    pub description: Option<String>,
}

/// Wait condition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitCondition {
    /// Selector to wait for
    #[serde(default)]
    pub selector: Option<String>,

    /// Text to wait for
    #[serde(default)]
    pub text: Option<String>,

    /// Wait for network idle
    #[serde(default)]
    pub network_idle: bool,

    /// Maximum wait time
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Cookie consent handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConsentConfig {
    /// Selector for accept button
    pub accept_selector: Option<String>,

    /// Selector for reject/decline button
    pub reject_selector: Option<String>,

    /// Selector for close button
    pub close_selector: Option<String>,

    /// Preferred action: accept, reject, or dismiss
    #[serde(default = "default_cookie_action")]
    pub action: String,
}

fn default_cookie_action() -> String {
    "accept".to_string()
}

/// CAPTCHA handling preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaConfig {
    /// Whether to auto-pause on CAPTCHA
    #[serde(default = "default_true")]
    pub pause_on_detect: bool,

    /// Custom CAPTCHA indicators for this site
    #[serde(default)]
    pub custom_indicators: Vec<String>,

    /// Notes about CAPTCHA handling for this site
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Custom timeout configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Page load timeout
    pub page_load_ms: Option<u64>,

    /// Element wait timeout
    pub element_ms: Option<u64>,

    /// Navigation timeout
    pub navigation_ms: Option<u64>,

    /// Script execution timeout
    pub script_ms: Option<u64>,
}

/// Site instructions manager
pub struct SiteInstructionsManager {
    /// Directory where site instructions are stored
    sites_dir: PathBuf,
    /// Cached instructions
    cache: HashMap<String, SiteInstructions>,
}

impl SiteInstructionsManager {
    /// Create a new manager with the given sites directory
    pub fn new(sites_dir: PathBuf) -> Self {
        Self {
            sites_dir,
            cache: HashMap::new(),
        }
    }

    /// Load instructions from disk
    pub fn load_all(&mut self) -> Result<()> {
        self.cache.clear();

        if !self.sites_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.sites_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(instructions) = toml::from_str::<SiteInstructions>(&content) {
                        self.cache.insert(instructions.domain.clone(), instructions);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get instructions for a URL
    pub fn get_for_url(&self, url: &str) -> Option<&SiteInstructions> {
        // Extract domain from URL
        let domain = extract_domain(url)?;

        // Try exact match first
        if let Some(instructions) = self.cache.get(&domain) {
            return Some(instructions);
        }

        // Try wildcard matches
        for (pattern, instructions) in &self.cache {
            if matches_domain_pattern(pattern, &domain) {
                return Some(instructions);
            }
        }

        None
    }

    /// Save instructions for a site
    pub fn save(&self, instructions: &SiteInstructions) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.sites_dir)?;

        // Create filename from domain
        let filename = instructions.domain.replace(['*', '.'], "_") + ".toml";
        let path = self.sites_dir.join(filename);

        let content = toml::to_string_pretty(instructions)?;
        std::fs::write(&path, content)?;

        Ok(path)
    }

    /// List all saved site instructions
    pub fn list(&self) -> Vec<&SiteInstructions> {
        self.cache.values().collect()
    }

    /// Delete instructions for a domain
    pub fn delete(&mut self, domain: &str) -> Result<bool> {
        if self.cache.remove(domain).is_some() {
            // Also delete file
            let filename = domain.replace(['*', '.'], "_") + ".toml";
            let path = self.sites_dir.join(filename);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create example/template instructions
    pub fn create_template(domain: &str) -> SiteInstructions {
        SiteInstructions {
            domain: domain.to_string(),
            description: Some(format!("Custom instructions for {}", domain)),
            login: Some(LoginInstructions {
                url: Some("/login".to_string()),
                username_selector: Some("#username, #email, [name='email']".to_string()),
                password_selector: Some("#password, [type='password']".to_string()),
                submit_selector: Some("[type='submit'], button.login".to_string()),
                success_indicator: Some(".dashboard, .welcome".to_string()),
                failure_indicator: Some(".error, .alert-danger".to_string()),
                extra_steps: vec![],
            }),
            selectors: {
                let mut map = HashMap::new();
                map.insert("search".to_string(), "#search, [name='q']".to_string());
                map.insert("submit".to_string(), "[type='submit']".to_string());
                map
            },
            before_actions: HashMap::new(),
            after_actions: HashMap::new(),
            wait_ready: Some(WaitCondition {
                selector: Some("body".to_string()),
                text: None,
                network_idle: false,
                timeout_ms: Some(5000),
            }),
            cookie_consent: Some(CookieConsentConfig {
                accept_selector: Some("[id*='accept'], [class*='accept'], .cookie-accept".to_string()),
                reject_selector: Some("[id*='reject'], [class*='reject'], .cookie-reject".to_string()),
                close_selector: Some(".cookie-close, .cookie-dismiss".to_string()),
                action: "accept".to_string(),
            }),
            captcha: None,
            timeouts: None,
            notes: vec![
                "Add custom notes about this site here".to_string(),
            ],
        }
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    // Handle URLs with and without protocol
    let url = url.trim();
    let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);

    // Get domain part (before first /)
    let domain = url.split('/').next()?;

    // Remove port if present
    let domain = domain.split(':').next()?;

    Some(domain.to_lowercase())
}

/// Check if a domain pattern matches a domain
fn matches_domain_pattern(pattern: &str, domain: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // Wildcard pattern
        domain.ends_with(suffix) || domain == suffix.trim_start_matches('.')
    } else {
        pattern == domain
    }
}

/// Format site instructions for human-readable output
pub fn format_instructions(instructions: &SiteInstructions) -> String {
    let mut output = String::new();

    output.push_str(&format!("Site: {}\n", instructions.domain));

    if let Some(desc) = &instructions.description {
        output.push_str(&format!("  Description: {}\n", desc));
    }

    if let Some(login) = &instructions.login {
        output.push_str("\n  Login Configuration:\n");
        if let Some(url) = &login.url {
            output.push_str(&format!("    URL: {}\n", url));
        }
        if let Some(sel) = &login.username_selector {
            output.push_str(&format!("    Username: {}\n", sel));
        }
        if let Some(sel) = &login.password_selector {
            output.push_str(&format!("    Password: {}\n", sel));
        }
    }

    if !instructions.selectors.is_empty() {
        output.push_str("\n  Custom Selectors:\n");
        for (name, selector) in &instructions.selectors {
            output.push_str(&format!("    {}: {}\n", name, selector));
        }
    }

    if let Some(cookie) = &instructions.cookie_consent {
        output.push_str("\n  Cookie Consent:\n");
        output.push_str(&format!("    Action: {}\n", cookie.action));
        if let Some(sel) = &cookie.accept_selector {
            output.push_str(&format!("    Accept: {}\n", sel));
        }
    }

    if !instructions.notes.is_empty() {
        output.push_str("\n  Notes:\n");
        for note in &instructions.notes {
            output.push_str(&format!("    - {}\n", note));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://sub.example.com:8080/"), Some("sub.example.com".to_string()));
        assert_eq!(extract_domain("example.com"), Some("example.com".to_string()));
    }

    #[test]
    fn test_domain_pattern_matching() {
        assert!(matches_domain_pattern("example.com", "example.com"));
        assert!(!matches_domain_pattern("example.com", "sub.example.com"));
        assert!(matches_domain_pattern("*.example.com", "sub.example.com"));
        assert!(matches_domain_pattern("*.example.com", "deep.sub.example.com"));
    }

    #[test]
    fn test_create_template() {
        let template = SiteInstructionsManager::create_template("example.com");
        assert_eq!(template.domain, "example.com");
        assert!(template.login.is_some());
        assert!(template.cookie_consent.is_some());
    }

    #[test]
    fn test_serialize_deserialize() {
        let template = SiteInstructionsManager::create_template("test.com");
        let toml = toml::to_string_pretty(&template).unwrap();
        let parsed: SiteInstructions = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.domain, "test.com");
    }
}
