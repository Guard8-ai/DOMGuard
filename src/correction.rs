//! Self-correction module for DOMGuard
//!
//! Provides automatic error recovery strategies for browser automation.
//! When an action fails, the system can attempt various recovery techniques
//! before giving up or requesting user takeover.

use serde::{Deserialize, Serialize};

/// Types of errors that can occur during automation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationError {
    /// Element not found
    ElementNotFound,
    /// Element not visible
    ElementNotVisible,
    /// Element not interactable
    ElementNotInteractable,
    /// Navigation timeout
    NavigationTimeout,
    /// Network error
    NetworkError,
    /// JavaScript error
    JavaScriptError,
    /// CAPTCHA detected
    CaptchaDetected,
    /// Authentication required
    AuthRequired,
    /// Unexpected dialog
    UnexpectedDialog,
    /// Page changed unexpectedly
    UnexpectedPageChange,
    /// Stale element reference
    StaleElement,
    /// Click intercepted
    ClickIntercepted,
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for AutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomationError::ElementNotFound => write!(f, "Element not found"),
            AutomationError::ElementNotVisible => write!(f, "Element not visible"),
            AutomationError::ElementNotInteractable => write!(f, "Element not interactable"),
            AutomationError::NavigationTimeout => write!(f, "Navigation timeout"),
            AutomationError::NetworkError => write!(f, "Network error"),
            AutomationError::JavaScriptError => write!(f, "JavaScript error"),
            AutomationError::CaptchaDetected => write!(f, "CAPTCHA detected"),
            AutomationError::AuthRequired => write!(f, "Authentication required"),
            AutomationError::UnexpectedDialog => write!(f, "Unexpected dialog"),
            AutomationError::UnexpectedPageChange => write!(f, "Unexpected page change"),
            AutomationError::StaleElement => write!(f, "Stale element reference"),
            AutomationError::ClickIntercepted => write!(f, "Click intercepted by overlay"),
            AutomationError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

/// Recovery strategy to attempt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStrategy {
    /// Wait and retry
    WaitAndRetry { delay_ms: u64 },
    /// Scroll element into view
    ScrollIntoView,
    /// Dismiss any blocking overlay
    DismissOverlay,
    /// Refresh the page
    RefreshPage,
    /// Navigate back and forward
    NavigateBackForward,
    /// Clear cookies and retry
    ClearCookies,
    /// Close unexpected dialog
    CloseDialog,
    /// Wait for page to stabilize
    WaitForStable,
    /// Try alternate selector
    TryAlternateSelector { selectors: Vec<String> },
    /// Scroll and search for element
    ScrollAndSearch,
    /// Click via JavaScript
    ClickViaJs,
    /// Focus then type
    FocusThenType,
    /// Request user takeover
    RequestTakeover { reason: String },
}

impl std::fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryStrategy::WaitAndRetry { delay_ms } => write!(f, "Wait {}ms and retry", delay_ms),
            RecoveryStrategy::ScrollIntoView => write!(f, "Scroll element into view"),
            RecoveryStrategy::DismissOverlay => write!(f, "Dismiss blocking overlay"),
            RecoveryStrategy::RefreshPage => write!(f, "Refresh page"),
            RecoveryStrategy::NavigateBackForward => write!(f, "Navigate back/forward"),
            RecoveryStrategy::ClearCookies => write!(f, "Clear cookies"),
            RecoveryStrategy::CloseDialog => write!(f, "Close dialog"),
            RecoveryStrategy::WaitForStable => write!(f, "Wait for page to stabilize"),
            RecoveryStrategy::TryAlternateSelector { .. } => write!(f, "Try alternate selectors"),
            RecoveryStrategy::ScrollAndSearch => write!(f, "Scroll and search for element"),
            RecoveryStrategy::ClickViaJs => write!(f, "Click via JavaScript"),
            RecoveryStrategy::FocusThenType => write!(f, "Focus element then type"),
            RecoveryStrategy::RequestTakeover { reason } => write!(f, "Request takeover: {}", reason),
        }
    }
}

/// Self-correction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionConfig {
    /// Whether self-correction is enabled
    pub enabled: bool,
    /// Maximum number of retry attempts per error
    pub max_retries: u32,
    /// Base delay between retries (ms)
    pub base_delay_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Maximum total recovery time (ms)
    pub max_recovery_time_ms: u64,
    /// Whether to auto-dismiss dialogs
    pub auto_dismiss_dialogs: bool,
    /// Whether to auto-scroll to elements
    pub auto_scroll: bool,
    /// Whether to request takeover after exhausting strategies
    pub takeover_on_failure: bool,
}

impl Default for CorrectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            base_delay_ms: 500,
            exponential_backoff: true,
            max_recovery_time_ms: 30000,
            auto_dismiss_dialogs: true,
            auto_scroll: true,
            takeover_on_failure: true,
        }
    }
}

/// Get recovery strategies for an error type
pub fn get_recovery_strategies(error: &AutomationError, action: &str) -> Vec<RecoveryStrategy> {
    match error {
        AutomationError::ElementNotFound => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 500 },
                RecoveryStrategy::WaitForStable,
                RecoveryStrategy::ScrollAndSearch,
                RecoveryStrategy::WaitAndRetry { delay_ms: 1000 },
                RecoveryStrategy::RefreshPage,
            ]
        }
        AutomationError::ElementNotVisible => {
            vec![
                RecoveryStrategy::ScrollIntoView,
                RecoveryStrategy::WaitAndRetry { delay_ms: 500 },
                RecoveryStrategy::DismissOverlay,
                RecoveryStrategy::ScrollAndSearch,
            ]
        }
        AutomationError::ElementNotInteractable => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 300 },
                RecoveryStrategy::ScrollIntoView,
                RecoveryStrategy::DismissOverlay,
                if action == "click" {
                    RecoveryStrategy::ClickViaJs
                } else {
                    RecoveryStrategy::FocusThenType
                },
            ]
        }
        AutomationError::NavigationTimeout => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 2000 },
                RecoveryStrategy::RefreshPage,
                RecoveryStrategy::NavigateBackForward,
            ]
        }
        AutomationError::NetworkError => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 1000 },
                RecoveryStrategy::WaitAndRetry { delay_ms: 3000 },
                RecoveryStrategy::RefreshPage,
            ]
        }
        AutomationError::CaptchaDetected => {
            vec![
                RecoveryStrategy::RequestTakeover {
                    reason: "CAPTCHA detected".to_string(),
                },
            ]
        }
        AutomationError::AuthRequired => {
            vec![
                RecoveryStrategy::RequestTakeover {
                    reason: "Authentication required".to_string(),
                },
            ]
        }
        AutomationError::UnexpectedDialog => {
            vec![
                RecoveryStrategy::CloseDialog,
                RecoveryStrategy::WaitAndRetry { delay_ms: 300 },
            ]
        }
        AutomationError::UnexpectedPageChange => {
            vec![
                RecoveryStrategy::NavigateBackForward,
                RecoveryStrategy::WaitForStable,
            ]
        }
        AutomationError::StaleElement => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 200 },
                RecoveryStrategy::WaitForStable,
                RecoveryStrategy::RefreshPage,
            ]
        }
        AutomationError::ClickIntercepted => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 300 },
                RecoveryStrategy::DismissOverlay,
                RecoveryStrategy::ScrollIntoView,
                RecoveryStrategy::ClickViaJs,
            ]
        }
        AutomationError::JavaScriptError | AutomationError::Unknown(_) => {
            vec![
                RecoveryStrategy::WaitAndRetry { delay_ms: 500 },
                RecoveryStrategy::RefreshPage,
            ]
        }
    }
}

/// Classify an error message into an AutomationError type
pub fn classify_error(error_message: &str) -> AutomationError {
    let lower = error_message.to_lowercase();

    if lower.contains("not found") || lower.contains("no element") || lower.contains("could not find") {
        AutomationError::ElementNotFound
    } else if lower.contains("not visible") || lower.contains("hidden") || lower.contains("display: none") {
        AutomationError::ElementNotVisible
    } else if lower.contains("not interactable") || lower.contains("disabled") || lower.contains("readonly") {
        AutomationError::ElementNotInteractable
    } else if lower.contains("navigation") && lower.contains("timeout") {
        AutomationError::NavigationTimeout
    } else if lower.contains("network") || lower.contains("fetch") || lower.contains("connection") {
        AutomationError::NetworkError
    } else if lower.contains("javascript") || lower.contains("script error") || lower.contains("uncaught") {
        AutomationError::JavaScriptError
    } else if lower.contains("captcha") || lower.contains("recaptcha") || lower.contains("hcaptcha") {
        AutomationError::CaptchaDetected
    } else if lower.contains("login") || lower.contains("sign in") || lower.contains("authentication") {
        AutomationError::AuthRequired
    } else if lower.contains("dialog") || lower.contains("alert") || lower.contains("confirm") {
        AutomationError::UnexpectedDialog
    } else if lower.contains("stale") || lower.contains("detached") || lower.contains("removed from dom") {
        AutomationError::StaleElement
    } else if lower.contains("intercepted") || lower.contains("obscured") || lower.contains("overlay") {
        AutomationError::ClickIntercepted
    } else if lower.contains("unexpected") && (lower.contains("page") || lower.contains("url") || lower.contains("navigate")) {
        AutomationError::UnexpectedPageChange
    } else {
        AutomationError::Unknown(error_message.to_string())
    }
}

/// JavaScript for dismissing common overlays
pub fn dismiss_overlay_script() -> &'static str {
    r#"
    (function() {
        // Common overlay selectors
        const overlaySelectors = [
            '.modal-backdrop',
            '.overlay',
            '.popup-overlay',
            '[class*="modal"]',
            '[class*="overlay"]',
            '[class*="backdrop"]',
            '.cookie-banner',
            '.cookie-consent',
            '[class*="cookie"]',
            '.gdpr-consent',
            '.notification-banner',
            '.promo-popup',
            '.subscribe-popup',
            '.newsletter-popup'
        ];

        // Close button selectors
        const closeSelectors = [
            '.close',
            '.close-btn',
            '.close-button',
            '[aria-label="Close"]',
            '[aria-label="close"]',
            'button[class*="close"]',
            '.modal-close',
            '.dismiss',
            '.cancel'
        ];

        let dismissed = false;

        // Try clicking close buttons first
        for (const selector of closeSelectors) {
            const buttons = document.querySelectorAll(selector);
            for (const btn of buttons) {
                if (btn.offsetParent !== null) { // visible
                    try {
                        btn.click();
                        dismissed = true;
                    } catch (e) {}
                }
            }
        }

        // Hide overlays directly
        for (const selector of overlaySelectors) {
            const elements = document.querySelectorAll(selector);
            for (const el of elements) {
                if (el.offsetParent !== null) {
                    try {
                        el.style.display = 'none';
                        dismissed = true;
                    } catch (e) {}
                }
            }
        }

        return dismissed;
    })()
    "#
}

/// JavaScript for waiting until page is stable
pub fn wait_stable_script() -> &'static str {
    r#"
    (async function() {
        const startTime = Date.now();
        const maxWait = 3000;
        const checkInterval = 200;

        let lastHTML = document.documentElement.outerHTML;
        let stableCount = 0;
        const requiredStableChecks = 3;

        while (Date.now() - startTime < maxWait) {
            await new Promise(r => setTimeout(r, checkInterval));

            const currentHTML = document.documentElement.outerHTML;
            if (currentHTML === lastHTML) {
                stableCount++;
                if (stableCount >= requiredStableChecks) {
                    return { stable: true, duration: Date.now() - startTime };
                }
            } else {
                stableCount = 0;
                lastHTML = currentHTML;
            }
        }

        return { stable: false, duration: Date.now() - startTime };
    })()
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_element_not_found() {
        assert_eq!(
            classify_error("Element not found: #missing-button"),
            AutomationError::ElementNotFound
        );
    }

    #[test]
    fn test_classify_captcha() {
        assert_eq!(
            classify_error("reCAPTCHA challenge detected"),
            AutomationError::CaptchaDetected
        );
    }

    #[test]
    fn test_classify_network() {
        assert_eq!(
            classify_error("Network connection failed"),
            AutomationError::NetworkError
        );
    }

    #[test]
    fn test_get_recovery_strategies_element_not_found() {
        let strategies = get_recovery_strategies(&AutomationError::ElementNotFound, "click");
        assert!(!strategies.is_empty());
        // First strategy should be wait and retry
        assert!(matches!(strategies[0], RecoveryStrategy::WaitAndRetry { .. }));
    }

    #[test]
    fn test_get_recovery_strategies_captcha() {
        let strategies = get_recovery_strategies(&AutomationError::CaptchaDetected, "click");
        assert_eq!(strategies.len(), 1);
        assert!(matches!(strategies[0], RecoveryStrategy::RequestTakeover { .. }));
    }
}
