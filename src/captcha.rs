//! CAPTCHA detection for DOMGuard
//!
//! Detects common CAPTCHA implementations and pauses for human intervention.
//! Supports reCAPTCHA, hCaptcha, Cloudflare Turnstile, and other common patterns.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// Types of CAPTCHA detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaType {
    /// Google reCAPTCHA v2 (checkbox)
    RecaptchaV2,
    /// Google reCAPTCHA v3 (invisible)
    RecaptchaV3,
    /// hCaptcha
    Hcaptcha,
    /// Cloudflare Turnstile
    CloudflareTurnstile,
    /// Cloudflare challenge page
    CloudflareChallenge,
    /// Arkose Labs FunCaptcha
    FunCaptcha,
    /// Simple image-based CAPTCHA
    ImageCaptcha,
    /// Audio CAPTCHA
    AudioCaptcha,
    /// Text-based CAPTCHA (math puzzles, etc.)
    TextCaptcha,
    /// Slider/puzzle CAPTCHA
    SliderCaptcha,
    /// Unknown but detected CAPTCHA
    Unknown,
}

/// CAPTCHA detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaDetection {
    /// Whether a CAPTCHA was detected
    pub detected: bool,
    /// Type of CAPTCHA if detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_type: Option<CaptchaType>,
    /// Selector for the CAPTCHA element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the CAPTCHA appears to be solved
    pub appears_solved: bool,
    /// Recommended action
    pub recommendation: CaptchaRecommendation,
}

/// Recommended action for CAPTCHA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaRecommendation {
    /// No action needed (no CAPTCHA or already solved)
    Continue,
    /// Pause and wait for human to solve CAPTCHA
    PauseForHuman,
    /// Wait for CAPTCHA to auto-solve (reCAPTCHA v3)
    WaitForAutoSolve,
    /// Retry the action (Cloudflare challenge)
    Retry,
}

impl Default for CaptchaDetection {
    fn default() -> Self {
        Self {
            detected: false,
            captcha_type: None,
            selector: None,
            description: None,
            appears_solved: false,
            recommendation: CaptchaRecommendation::Continue,
        }
    }
}

/// JavaScript code to detect CAPTCHAs on a page
pub fn captcha_detection_script() -> &'static str {
    r#"
    (function() {
        const result = {
            detected: false,
            captcha_type: null,
            selector: null,
            description: null,
            appears_solved: false
        };

        // Check for reCAPTCHA v2
        const recaptchaV2 = document.querySelector('.g-recaptcha, [data-sitekey], iframe[src*="recaptcha"]');
        if (recaptchaV2) {
            result.detected = true;
            result.captcha_type = 'recaptcha_v2';
            result.selector = '.g-recaptcha';
            result.description = 'Google reCAPTCHA v2 detected';
            // Check if checkbox is checked
            const checkbox = document.querySelector('.recaptcha-checkbox-checked, .recaptcha-checkbox-checkmark[style*="opacity: 1"]');
            result.appears_solved = !!checkbox;
            return result;
        }

        // Check for reCAPTCHA v3 (invisible)
        const recaptchaV3 = document.querySelector('.grecaptcha-badge, script[src*="recaptcha/api.js?render="]');
        if (recaptchaV3) {
            result.detected = true;
            result.captcha_type = 'recaptcha_v3';
            result.selector = '.grecaptcha-badge';
            result.description = 'Google reCAPTCHA v3 (invisible) detected';
            result.appears_solved = true; // v3 is automatic
            return result;
        }

        // Check for hCaptcha
        const hcaptcha = document.querySelector('.h-captcha, iframe[src*="hcaptcha.com"], [data-hcaptcha-widget-id]');
        if (hcaptcha) {
            result.detected = true;
            result.captcha_type = 'hcaptcha';
            result.selector = '.h-captcha';
            result.description = 'hCaptcha detected';
            // Check for success indicator
            const solved = document.querySelector('[data-hcaptcha-response]:not([data-hcaptcha-response=""])');
            result.appears_solved = !!solved;
            return result;
        }

        // Check for Cloudflare Turnstile
        const turnstile = document.querySelector('.cf-turnstile, iframe[src*="challenges.cloudflare.com/turnstile"]');
        if (turnstile) {
            result.detected = true;
            result.captcha_type = 'cloudflare_turnstile';
            result.selector = '.cf-turnstile';
            result.description = 'Cloudflare Turnstile detected';
            const solved = document.querySelector('[name="cf-turnstile-response"]:not([value=""])');
            result.appears_solved = !!solved;
            return result;
        }

        // Check for Cloudflare challenge page
        const cfChallenge = document.querySelector('#cf-challenge-running, .cf-browser-verification, #challenge-form, #challenge-error-text');
        if (cfChallenge || document.title.includes('Just a moment') || document.body.innerHTML.includes('Checking if the site connection is secure')) {
            result.detected = true;
            result.captcha_type = 'cloudflare_challenge';
            result.selector = '#challenge-form';
            result.description = 'Cloudflare challenge page detected';
            result.appears_solved = false;
            return result;
        }

        // Check for FunCaptcha (Arkose Labs)
        const funcaptcha = document.querySelector('#FunCaptcha, iframe[src*="arkoselabs.com"], [data-callback*="funcaptcha"]');
        if (funcaptcha) {
            result.detected = true;
            result.captcha_type = 'fun_captcha';
            result.selector = '#FunCaptcha';
            result.description = 'Arkose Labs FunCaptcha detected';
            result.appears_solved = false;
            return result;
        }

        // Check for slider CAPTCHA patterns
        const slider = document.querySelector('.slider-captcha, .slide-verify, .geetest_slider, [class*="captcha-slider"]');
        if (slider) {
            result.detected = true;
            result.captcha_type = 'slider_captcha';
            result.selector = slider.className.split(' ')[0];
            result.description = 'Slider/puzzle CAPTCHA detected';
            result.appears_solved = false;
            return result;
        }

        // Check for image CAPTCHA
        const imageCaptcha = document.querySelector('img[src*="captcha"], img[alt*="captcha" i], input[name*="captcha" i]');
        if (imageCaptcha) {
            result.detected = true;
            result.captcha_type = 'image_captcha';
            result.selector = imageCaptcha.tagName.toLowerCase();
            result.description = 'Image-based CAPTCHA detected';
            result.appears_solved = false;
            return result;
        }

        // Check for generic CAPTCHA indicators
        const genericCaptcha = document.querySelector('[class*="captcha" i], [id*="captcha" i], [aria-label*="captcha" i]');
        if (genericCaptcha && !result.detected) {
            result.detected = true;
            result.captcha_type = 'unknown';
            result.selector = genericCaptcha.id || genericCaptcha.className.split(' ')[0];
            result.description = 'Unknown CAPTCHA type detected';
            result.appears_solved = false;
            return result;
        }

        return result;
    })()
    "#
}

/// Parse the detection result from JavaScript
pub fn parse_captcha_detection(js_result: &serde_json::Value) -> CaptchaDetection {
    if js_result.is_null() {
        return CaptchaDetection::default();
    }

    let detected = js_result
        .get("detected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !detected {
        return CaptchaDetection::default();
    }

    let captcha_type = js_result
        .get("captcha_type")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "recaptcha_v2" => CaptchaType::RecaptchaV2,
            "recaptcha_v3" => CaptchaType::RecaptchaV3,
            "hcaptcha" => CaptchaType::Hcaptcha,
            "cloudflare_turnstile" => CaptchaType::CloudflareTurnstile,
            "cloudflare_challenge" => CaptchaType::CloudflareChallenge,
            "fun_captcha" => CaptchaType::FunCaptcha,
            "image_captcha" => CaptchaType::ImageCaptcha,
            "audio_captcha" => CaptchaType::AudioCaptcha,
            "text_captcha" => CaptchaType::TextCaptcha,
            "slider_captcha" => CaptchaType::SliderCaptcha,
            _ => CaptchaType::Unknown,
        });

    let selector = js_result
        .get("selector")
        .and_then(|v| v.as_str())
        .map(String::from);

    let description = js_result
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let appears_solved = js_result
        .get("appears_solved")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Determine recommendation based on type and solved state
    let recommendation = if appears_solved {
        CaptchaRecommendation::Continue
    } else {
        match captcha_type {
            Some(CaptchaType::RecaptchaV3) => CaptchaRecommendation::WaitForAutoSolve,
            Some(CaptchaType::CloudflareChallenge) => CaptchaRecommendation::Retry,
            Some(_) => CaptchaRecommendation::PauseForHuman,
            None => CaptchaRecommendation::Continue,
        }
    };

    CaptchaDetection {
        detected,
        captcha_type,
        selector,
        description,
        appears_solved,
        recommendation,
    }
}

/// Format CAPTCHA detection for human-readable output
pub fn format_captcha_detection(detection: &CaptchaDetection) -> String {
    if !detection.detected {
        return "No CAPTCHA detected".to_string();
    }

    let mut output = String::new();

    if let Some(desc) = &detection.description {
        let _ = writeln!(output, "⚠️  {}", desc);
    } else {
        output.push_str("⚠️  CAPTCHA detected\n");
    }

    if let Some(captcha_type) = &detection.captcha_type {
        let _ = writeln!(output, "   Type: {:?}", captcha_type);
    }

    if let Some(selector) = &detection.selector {
        let _ = writeln!(output, "   Element: {}", selector);
    }

    let _ = writeln!(
        output,
        "   Solved: {}",
        if detection.appears_solved {
            "Yes"
        } else {
            "No"
        }
    );

    let action = match detection.recommendation {
        CaptchaRecommendation::Continue => "Continue with automation",
        CaptchaRecommendation::PauseForHuman => "PAUSE - Human intervention required",
        CaptchaRecommendation::WaitForAutoSolve => "Wait for automatic solution",
        CaptchaRecommendation::Retry => "Retry after short delay",
    };
    let _ = writeln!(output, "   Action: {}", action);

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_detection() {
        let detection = CaptchaDetection::default();
        assert!(!detection.detected);
        assert_eq!(detection.recommendation, CaptchaRecommendation::Continue);
    }

    #[test]
    fn test_parse_recaptcha_v2() {
        let result = serde_json::json!({
            "detected": true,
            "captcha_type": "recaptcha_v2",
            "selector": ".g-recaptcha",
            "description": "Google reCAPTCHA v2 detected",
            "appears_solved": false
        });

        let detection = parse_captcha_detection(&result);
        assert!(detection.detected);
        assert_eq!(detection.captcha_type, Some(CaptchaType::RecaptchaV2));
        assert_eq!(
            detection.recommendation,
            CaptchaRecommendation::PauseForHuman
        );
    }

    #[test]
    fn test_parse_cloudflare_challenge() {
        let result = serde_json::json!({
            "detected": true,
            "captcha_type": "cloudflare_challenge",
            "selector": "#challenge-form",
            "description": "Cloudflare challenge page detected",
            "appears_solved": false
        });

        let detection = parse_captcha_detection(&result);
        assert!(detection.detected);
        assert_eq!(
            detection.captcha_type,
            Some(CaptchaType::CloudflareChallenge)
        );
        assert_eq!(detection.recommendation, CaptchaRecommendation::Retry);
    }

    #[test]
    fn test_parse_solved_captcha() {
        let result = serde_json::json!({
            "detected": true,
            "captcha_type": "hcaptcha",
            "selector": ".h-captcha",
            "description": "hCaptcha detected",
            "appears_solved": true
        });

        let detection = parse_captcha_detection(&result);
        assert!(detection.detected);
        assert!(detection.appears_solved);
        assert_eq!(detection.recommendation, CaptchaRecommendation::Continue);
    }

    #[test]
    fn test_parse_no_captcha() {
        let result = serde_json::json!({
            "detected": false
        });

        let detection = parse_captcha_detection(&result);
        assert!(!detection.detected);
    }
}
