//! Security features for DOMGuard
//!
//! Implements sensitive action detection and blocked site list.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sensitive action types that require extra attention
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveActionType {
    /// Typing into password fields
    PasswordInput,
    /// Typing into fields with sensitive names (ssn, credit card, etc.)
    SensitiveFieldInput,
    /// Submitting forms with payment information
    PaymentSubmission,
    /// Submitting login forms
    LoginSubmission,
    /// File uploads
    FileUpload,
    /// Accessing financial sites
    FinancialSite,
    /// Navigating to blocked sites
    BlockedSite,
    /// Accessing pages with personal data
    PersonalData,
}

/// Detection result for sensitive actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveActionDetection {
    pub detected: bool,
    pub action_type: Option<SensitiveActionType>,
    pub reason: Option<String>,
    pub severity: Severity,
    pub element_info: Option<ElementInfo>,
}

/// Element information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_action: Option<String>,
}

/// Severity level for sensitive actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for SensitiveActionDetection {
    fn default() -> Self {
        Self {
            detected: false,
            action_type: None,
            reason: None,
            severity: Severity::Low,
            element_info: None,
        }
    }
}

/// Blocked sites configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockedSitesConfig {
    /// List of blocked domains/patterns
    pub blocked: Vec<String>,
    /// Whether to block by default and only allow certain sites
    pub default_block: bool,
    /// Allowed sites (when default_block is true)
    pub allowed: Vec<String>,
}

impl BlockedSitesConfig {
    /// Load blocked sites from config file
    pub fn load(config_path: &PathBuf) -> Result<Self> {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: BlockedSitesConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save blocked sites to config file
    pub fn save(&self, config_path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Check if a URL is blocked
    pub fn is_blocked(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();

        if self.default_block {
            // Block by default, check if in allowed list
            !self
                .allowed
                .iter()
                .any(|pattern| url_lower.contains(&pattern.to_lowercase()))
        } else {
            // Allow by default, check if in blocked list
            self.blocked
                .iter()
                .any(|pattern| url_lower.contains(&pattern.to_lowercase()))
        }
    }

    /// Add a site to blocked list
    pub fn block(&mut self, pattern: &str) {
        if !self.blocked.contains(&pattern.to_string()) {
            self.blocked.push(pattern.to_string());
        }
    }

    /// Remove a site from blocked list
    pub fn unblock(&mut self, pattern: &str) {
        self.blocked.retain(|p| p != pattern);
    }

    /// Add a site to allowed list
    pub fn allow(&mut self, pattern: &str) {
        if !self.allowed.contains(&pattern.to_string()) {
            self.allowed.push(pattern.to_string());
        }
    }
}

/// Security checker for detecting sensitive actions
pub struct SecurityChecker {
    blocked_sites: BlockedSitesConfig,
}

impl SecurityChecker {
    pub fn new(blocked_sites: BlockedSitesConfig) -> Self {
        Self { blocked_sites }
    }

    /// Check if typing into a selector is sensitive
    pub fn check_type_action(&self, selector: &str, _text: &str) -> SensitiveActionDetection {
        let selector_lower = selector.to_lowercase();

        // Check for password fields
        if selector_lower.contains("password")
            || selector_lower.contains("[type=password]")
            || selector_lower.contains("[type=\"password\"]")
        {
            return SensitiveActionDetection {
                detected: true,
                action_type: Some(SensitiveActionType::PasswordInput),
                reason: Some("Typing into password field".to_string()),
                severity: Severity::High,
                element_info: Some(ElementInfo {
                    tag: "input".to_string(),
                    id: None,
                    name: None,
                    input_type: Some("password".to_string()),
                    form_action: None,
                }),
            };
        }

        // Check for sensitive field names
        let sensitive_patterns = [
            ("ssn", "Social Security Number"),
            ("social-security", "Social Security Number"),
            ("credit-card", "Credit Card"),
            ("creditcard", "Credit Card"),
            ("card-number", "Credit Card Number"),
            ("cvv", "CVV Code"),
            ("cvc", "CVC Code"),
            ("expiry", "Expiration Date"),
            ("pin", "PIN"),
            ("bank-account", "Bank Account"),
            ("routing", "Routing Number"),
            ("tax-id", "Tax ID"),
            ("passport", "Passport Number"),
            ("license", "License Number"),
        ];

        for (pattern, description) in sensitive_patterns {
            if selector_lower.contains(pattern) {
                return SensitiveActionDetection {
                    detected: true,
                    action_type: Some(SensitiveActionType::SensitiveFieldInput),
                    reason: Some(format!("Typing into {} field", description)),
                    severity: Severity::High,
                    element_info: Some(ElementInfo {
                        tag: "input".to_string(),
                        id: None,
                        name: Some(pattern.to_string()),
                        input_type: None,
                        form_action: None,
                    }),
                };
            }
        }

        SensitiveActionDetection::default()
    }

    /// Check if clicking a selector is sensitive
    pub fn check_click_action(&self, selector: &str) -> SensitiveActionDetection {
        let selector_lower = selector.to_lowercase();

        // Check for submit buttons on potentially sensitive forms
        let form_submit_patterns = [
            ("login", "Login form submission"),
            ("signin", "Sign-in form submission"),
            ("sign-in", "Sign-in form submission"),
            ("signup", "Sign-up form submission"),
            ("sign-up", "Sign-up form submission"),
            ("register", "Registration form submission"),
            ("checkout", "Checkout form submission"),
            ("payment", "Payment form submission"),
            ("purchase", "Purchase form submission"),
            ("buy", "Purchase form submission"),
            ("transfer", "Transfer form submission"),
            ("send-money", "Money transfer"),
            ("confirm", "Confirmation action"),
            ("delete", "Delete action"),
            ("remove", "Remove action"),
        ];

        for (pattern, description) in form_submit_patterns {
            if selector_lower.contains(pattern)
                && (selector_lower.contains("submit")
                    || selector_lower.contains("button")
                    || selector_lower.contains("btn"))
            {
                let severity = if pattern.contains("delete")
                    || pattern.contains("remove")
                    || pattern.contains("payment")
                    || pattern.contains("checkout")
                    || pattern.contains("transfer")
                {
                    Severity::Critical
                } else {
                    Severity::Medium
                };

                return SensitiveActionDetection {
                    detected: true,
                    action_type: Some(SensitiveActionType::PaymentSubmission),
                    reason: Some(description.to_string()),
                    severity,
                    element_info: Some(ElementInfo {
                        tag: "button".to_string(),
                        id: None,
                        name: None,
                        input_type: None,
                        form_action: None,
                    }),
                };
            }
        }

        SensitiveActionDetection::default()
    }

    /// Check if navigation to a URL is sensitive/blocked
    pub fn check_navigation(&self, url: &str) -> SensitiveActionDetection {
        // Check blocked sites
        if self.blocked_sites.is_blocked(url) {
            return SensitiveActionDetection {
                detected: true,
                action_type: Some(SensitiveActionType::BlockedSite),
                reason: Some(format!("Navigation to blocked site: {}", url)),
                severity: Severity::Critical,
                element_info: None,
            };
        }

        // Check for financial sites
        let financial_patterns = [
            "bank",
            "paypal",
            "venmo",
            "chase",
            "wellsfargo",
            "bankofamerica",
            "citibank",
            "capitalone",
            "usbank",
            "pnc",
            "td.com",
            "schwab",
            "fidelity",
            "vanguard",
            "robinhood",
            "coinbase",
            "binance",
            "crypto.com",
            "kraken",
            "trading",
            "broker",
            "invest",
        ];

        let url_lower = url.to_lowercase();
        for pattern in financial_patterns {
            if url_lower.contains(pattern) {
                return SensitiveActionDetection {
                    detected: true,
                    action_type: Some(SensitiveActionType::FinancialSite),
                    reason: Some("Navigating to financial/banking site".to_string()),
                    severity: Severity::High,
                    element_info: None,
                };
            }
        }

        // Check for personal data sites
        let personal_data_patterns = [
            "healthcare",
            "medical",
            "health.gov",
            "irs.gov",
            "ssa.gov",
            "social-security",
            "medicare",
            "medicaid",
            "insurance",
        ];

        for pattern in personal_data_patterns {
            if url_lower.contains(pattern) {
                return SensitiveActionDetection {
                    detected: true,
                    action_type: Some(SensitiveActionType::PersonalData),
                    reason: Some("Navigating to site with personal/health data".to_string()),
                    severity: Severity::High,
                    element_info: None,
                };
            }
        }

        SensitiveActionDetection::default()
    }

    /// Check if file upload is sensitive
    pub fn check_upload(&self, files: &[PathBuf]) -> SensitiveActionDetection {
        // Check file extensions for sensitive documents
        let sensitive_extensions = [
            "pdf", "doc", "docx", "xls", "xlsx", "csv", "txt", "key", "pem", "crt", "p12", "pfx",
            "env", "conf", "config", "ini", "json", "yaml", "yml",
        ];

        for file in files {
            if let Some(ext) = file.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if sensitive_extensions.contains(&ext_str.as_str()) {
                    return SensitiveActionDetection {
                        detected: true,
                        action_type: Some(SensitiveActionType::FileUpload),
                        reason: Some(format!(
                            "Uploading potentially sensitive file: {}",
                            file.display()
                        )),
                        severity: Severity::Medium,
                        element_info: None,
                    };
                }
            }

            // Check filename for sensitive patterns
            let filename = file
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            let sensitive_filenames = [
                "passport",
                "license",
                "ssn",
                "tax",
                "w2",
                "1099",
                "bank",
                "statement",
                "credential",
                "secret",
                "key",
                "password",
            ];

            for pattern in sensitive_filenames {
                if filename.contains(pattern) {
                    return SensitiveActionDetection {
                        detected: true,
                        action_type: Some(SensitiveActionType::FileUpload),
                        reason: Some(format!(
                            "Uploading potentially sensitive file: {}",
                            file.display()
                        )),
                        severity: Severity::High,
                        element_info: None,
                    };
                }
            }
        }

        SensitiveActionDetection::default()
    }
}

/// Format security warning for display
pub fn format_security_warning(detection: &SensitiveActionDetection) -> String {
    if !detection.detected {
        return String::new();
    }

    let severity_icon = match detection.severity {
        Severity::Low => "ℹ",
        Severity::Medium => "⚠",
        Severity::High => "⚠️",
        Severity::Critical => "🚨",
    };

    let mut warning = format!(
        "{} SECURITY WARNING: {}\n",
        severity_icon,
        detection
            .reason
            .as_deref()
            .unwrap_or("Sensitive action detected")
    );

    if let Some(action_type) = &detection.action_type {
        warning.push_str(&format!("   Type: {:?}\n", action_type));
    }
    warning.push_str(&format!("   Severity: {:?}\n", detection.severity));

    if let Some(elem) = &detection.element_info {
        warning.push_str(&format!("   Element: <{}>\n", elem.tag));
    }

    warning
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_detection() {
        let checker = SecurityChecker::new(BlockedSitesConfig::default());
        let result = checker.check_type_action("[type=password]", "secret");
        assert!(result.detected);
        assert_eq!(result.action_type, Some(SensitiveActionType::PasswordInput));
        assert_eq!(result.severity, Severity::High);
    }

    #[test]
    fn test_credit_card_detection() {
        let checker = SecurityChecker::new(BlockedSitesConfig::default());
        let result = checker.check_type_action("#credit-card-number", "1234");
        assert!(result.detected);
        assert_eq!(
            result.action_type,
            Some(SensitiveActionType::SensitiveFieldInput)
        );
    }

    #[test]
    fn test_blocked_site() {
        let mut config = BlockedSitesConfig::default();
        config.blocked.push("malicious-site.com".to_string());

        let checker = SecurityChecker::new(config);
        let result = checker.check_navigation("https://malicious-site.com/phishing");
        assert!(result.detected);
        assert_eq!(result.action_type, Some(SensitiveActionType::BlockedSite));
    }

    #[test]
    fn test_financial_site_detection() {
        let checker = SecurityChecker::new(BlockedSitesConfig::default());
        let result = checker.check_navigation("https://www.bankofamerica.com/");
        assert!(result.detected);
        assert_eq!(result.action_type, Some(SensitiveActionType::FinancialSite));
    }

    #[test]
    fn test_upload_detection() {
        let checker = SecurityChecker::new(BlockedSitesConfig::default());
        let result = checker.check_upload(&[PathBuf::from("/home/user/passport.pdf")]);
        assert!(result.detected);
        assert_eq!(result.action_type, Some(SensitiveActionType::FileUpload));
    }

    #[test]
    fn test_normal_action() {
        let checker = SecurityChecker::new(BlockedSitesConfig::default());
        let result = checker.check_type_action("#search-input", "hello");
        assert!(!result.detected);
    }
}
