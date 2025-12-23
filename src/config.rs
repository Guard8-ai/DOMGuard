//! Configuration file handling for DOMGuard
//!
//! Loads configuration from .domguard/config.toml (project-local, like TaskGuard)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The embedded AI guide content
pub const AI_GUIDE_CONTENT: &str = include_str!("../AGENTIC_AI_DOMGUARD_GUIDE.md");

/// Chrome connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeConfig {
    /// Chrome DevTools port (default: 9222)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Chrome host (default: 127.0.0.1)
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

fn default_port() -> u16 {
    9222
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

/// Default settings for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Timeout in milliseconds (default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Screenshot format (default: png)
    #[serde(default = "default_screenshot_format")]
    pub screenshot_format: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout(),
            screenshot_format: default_screenshot_format(),
        }
    }
}

fn default_timeout() -> u64 {
    5000
}

fn default_screenshot_format() -> String {
    "png".to_string()
}

/// Inspire mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspireConfig {
    /// Directory to save inspirations (relative to .domguard/)
    #[serde(default = "default_save_dir")]
    pub save_dir: PathBuf,
}

impl Default for InspireConfig {
    fn default() -> Self {
        Self {
            save_dir: default_save_dir(),
        }
    }
}

fn default_save_dir() -> PathBuf {
    PathBuf::from("inspirations")
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub chrome: ChromeConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub inspire: InspireConfig,
}

impl Config {
    /// Find the .domguard directory by walking up from current dir
    pub fn find_domguard_dir() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let domguard_dir = current.join(".domguard");
            if domguard_dir.is_dir() {
                return Some(domguard_dir);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Get the .domguard directory path (in current directory)
    pub fn domguard_dir() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".domguard")
    }

    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::find_domguard_dir()
            .unwrap_or_else(Self::domguard_dir)
            .join("config.toml")
    }

    /// Get the inspirations directory path
    pub fn inspirations_dir(&self) -> PathBuf {
        Self::find_domguard_dir()
            .unwrap_or_else(Self::domguard_dir)
            .join(&self.inspire.save_dir)
    }

    /// Load configuration from .domguard/config.toml
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;
            let config: Config =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save configuration to .domguard/config.toml
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Get the WebSocket URL for Chrome DevTools
    pub fn ws_url(&self) -> String {
        format!("http://{}:{}", self.chrome.host, self.chrome.port)
    }

    /// Check if host is localhost (security check)
    pub fn is_localhost(&self) -> bool {
        matches!(self.chrome.host.as_str(), "127.0.0.1" | "localhost" | "::1")
    }

    /// Check if DOMGuard is initialized in current directory tree
    pub fn is_initialized() -> bool {
        Self::find_domguard_dir().is_some()
    }
}

/// Initialize DOMGuard in the current directory
pub fn init_domguard() -> Result<InitResult> {
    let domguard_dir = Config::domguard_dir();

    if domguard_dir.exists() {
        return Ok(InitResult {
            already_exists: true,
            domguard_dir,
            guide_path: None,
        });
    }

    // Create .domguard directory
    std::fs::create_dir_all(&domguard_dir)
        .with_context(|| "Failed to create .domguard directory")?;

    // Create subdirectories
    std::fs::create_dir_all(domguard_dir.join("inspirations"))
        .with_context(|| "Failed to create inspirations directory")?;
    std::fs::create_dir_all(domguard_dir.join("screenshots"))
        .with_context(|| "Failed to create screenshots directory")?;

    // Create default config
    let config = Config::default();
    config.save()?;

    // Copy AI guide to project root
    let guide_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("AGENTIC_AI_DOMGUARD_GUIDE.md");

    std::fs::write(&guide_path, AI_GUIDE_CONTENT).with_context(|| "Failed to write AI guide")?;

    Ok(InitResult {
        already_exists: false,
        domguard_dir,
        guide_path: Some(guide_path),
    })
}

/// Result of initialization
pub struct InitResult {
    pub already_exists: bool,
    pub domguard_dir: PathBuf,
    pub guide_path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.chrome.port, 9222);
        assert_eq!(config.chrome.host, "127.0.0.1");
        assert_eq!(config.defaults.timeout_ms, 5000);
        assert_eq!(config.defaults.screenshot_format, "png");
    }

    #[test]
    fn test_ws_url() {
        let config = Config::default();
        assert_eq!(config.ws_url(), "http://127.0.0.1:9222");
    }

    #[test]
    fn test_is_localhost() {
        let mut config = Config::default();
        assert!(config.is_localhost());

        config.chrome.host = "localhost".to_string();
        assert!(config.is_localhost());

        config.chrome.host = "::1".to_string();
        assert!(config.is_localhost());

        config.chrome.host = "192.168.1.1".to_string();
        assert!(!config.is_localhost());
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[chrome]
port = 9333
host = "127.0.0.1"

[defaults]
timeout_ms = 10000
screenshot_format = "jpeg"

[inspire]
save_dir = "my-inspirations"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.chrome.port, 9333);
        assert_eq!(config.defaults.timeout_ms, 10000);
        assert_eq!(config.defaults.screenshot_format, "jpeg");
        assert_eq!(config.inspire.save_dir, PathBuf::from("my-inspirations"));
    }
}
