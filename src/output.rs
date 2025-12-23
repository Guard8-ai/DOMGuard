//! Output formatting for DOMGuard
//!
//! Provides human-readable and JSON output modes

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output format mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Result wrapper for consistent output
#[derive(Debug, Serialize)]
pub struct CommandResult<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing_ms: Option<u64>,
}

impl<T: Serialize> CommandResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timing_ms: None,
        }
    }

    pub fn with_timing(mut self, ms: u64) -> Self {
        self.timing_ms = Some(ms);
        self
    }
}

impl CommandResult<()> {
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
            timing_ms: None,
        }
    }
}

/// Output formatter
pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    pub fn new(json: bool) -> Self {
        Self {
            format: if json {
                OutputFormat::Json
            } else {
                OutputFormat::Human
            },
        }
    }

    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    /// Output a result
    pub fn output<T: Serialize + std::fmt::Display>(&self, result: &CommandResult<T>) {
        match self.format {
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(result).unwrap_or_default()
                );
            }
            OutputFormat::Human => {
                if result.success {
                    if let Some(data) = &result.data {
                        println!("{}", data);
                    }
                    if let Some(ms) = result.timing_ms {
                        println!("{} {}ms", "Completed in".dimmed(), ms);
                    }
                } else if let Some(err) = &result.error {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                }
            }
        }
    }

    /// Output raw JSON data
    pub fn output_json<T: Serialize>(&self, data: &T) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
            }
            OutputFormat::Human => {
                println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
            }
        }
    }

    /// Print a section header
    pub fn header(&self, text: &str) {
        if self.format == OutputFormat::Human {
            println!("\n{}", text.cyan().bold());
            println!("{}", "─".repeat(text.len()).dimmed());
        }
    }

    /// Print a key-value pair
    pub fn kv(&self, key: &str, value: &str) {
        if self.format == OutputFormat::Human {
            println!("  {}: {}", key.dimmed(), value);
        }
    }

    /// Print a list item
    pub fn item(&self, text: &str) {
        if self.format == OutputFormat::Human {
            println!("  • {}", text);
        }
    }

    /// Print success message
    pub fn success(&self, msg: &str) {
        if self.format == OutputFormat::Human {
            println!("{} {}", "✓".green(), msg);
        }
    }

    /// Print warning message
    pub fn warning(&self, msg: &str) {
        if self.format == OutputFormat::Human {
            eprintln!("{} {}", "⚠".yellow(), msg);
        }
    }

    /// Print error message
    pub fn error(&self, msg: &str) {
        match self.format {
            OutputFormat::Json => {
                let result: CommandResult<()> = CommandResult::error(msg);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                );
            }
            OutputFormat::Human => {
                eprintln!("{} {}", "Error:".red().bold(), msg);
            }
        }
    }

    /// Print a hint message
    pub fn hint(&self, msg: &str) {
        if self.format == OutputFormat::Human {
            eprintln!("{} {}", "Hint:".blue(), msg);
        }
    }
}

/// DOM node representation for output
#[derive(Debug, Serialize)]
pub struct DomNode {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<DomNode>,
}

impl std::fmt::Display for DomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indent(f, 0)
    }
}

impl DomNode {
    fn fmt_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let pad = "  ".repeat(indent);

        // Build tag with id and classes
        let mut tag_str = format!("<{}", self.tag);
        if let Some(id) = &self.id {
            tag_str.push_str(&format!(" id=\"{}\"", id));
        }
        if let Some(classes) = &self.classes {
            if !classes.is_empty() {
                tag_str.push_str(&format!(" class=\"{}\"", classes.join(" ")));
            }
        }
        tag_str.push('>');

        writeln!(f, "{}{}", pad, tag_str)?;

        // Text content
        if let Some(text) = &self.text {
            let trimmed = text.trim();
            if !trimmed.is_empty() && trimmed.len() < 100 {
                writeln!(f, "{}  {}", pad, trimmed.dimmed())?;
            }
        }

        // Children
        for child in &self.children {
            child.fmt_indent(f, indent + 1)?;
        }

        Ok(())
    }
}

/// Console message for output
#[derive(Debug, Serialize)]
pub struct ConsoleMessage {
    pub level: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

impl std::fmt::Display for ConsoleMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_colored = match self.level.as_str() {
            "error" => self.level.red().to_string(),
            "warning" => self.level.yellow().to_string(),
            "info" => self.level.blue().to_string(),
            _ => self.level.clone(),
        };

        write!(f, "[{}] {}", level_colored, self.text)?;

        if let (Some(url), Some(line)) = (&self.url, &self.line) {
            write!(f, " ({}:{})", url.dimmed(), line)?;
        }

        Ok(())
    }
}

/// Network request for output
#[derive(Debug, Serialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

impl std::fmt::Display for NetworkRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = self
            .status
            .map(|s| {
                if (200..300).contains(&s) {
                    s.to_string().green().to_string()
                } else if s >= 400 {
                    s.to_string().red().to_string()
                } else {
                    s.to_string().yellow().to_string()
                }
            })
            .unwrap_or_else(|| "...".dimmed().to_string());

        write!(f, "{} {} [{}]", self.method.cyan(), self.url, status_str)?;

        if let Some(mime) = &self.mime_type {
            write!(f, " {}", mime.dimmed())?;
        }

        Ok(())
    }
}

/// Design inspiration output
#[derive(Debug, Serialize)]
pub struct DesignInspiration {
    pub url: String,
    pub colors: Vec<ColorInfo>,
    pub typography: Vec<TypographyInfo>,
    pub spacing: SpacingInfo,
    pub layout: LayoutInfo,
    pub animations: AnimationInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorInfo {
    pub hex: String,
    pub usage: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyInfo {
    pub font_family: String,
    pub font_size: String,
    pub font_weight: String,
    pub line_height: String,
    pub usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpacingInfo {
    #[serde(default)]
    pub padding_values: Vec<String>,
    #[serde(default)]
    pub margin_values: Vec<String>,
    #[serde(default)]
    pub gap_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutInfo {
    #[serde(default)]
    pub flex_containers: u32,
    #[serde(default)]
    pub grid_containers: u32,
    #[serde(default)]
    pub flex_directions: Vec<String>,
    #[serde(default)]
    pub grid_templates: Vec<String>,
    #[serde(default)]
    pub justify_content: Vec<String>,
    #[serde(default)]
    pub align_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationInfo {
    #[serde(default)]
    pub timing_functions: Vec<String>,
    #[serde(default)]
    pub durations: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<String>,
}

impl std::fmt::Display for DesignInspiration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", "Design Inspiration".cyan().bold())?;
        writeln!(f, "URL: {}\n", self.url)?;

        writeln!(f, "{}", "Colors:".bold())?;
        for color in &self.colors {
            writeln!(f, "  {} - {} ({}x)", color.hex, color.usage, color.count)?;
        }

        writeln!(f, "\n{}", "Typography:".bold())?;
        for typo in &self.typography {
            writeln!(
                f,
                "  {} {}px/{} {} - {}",
                typo.font_family, typo.font_size, typo.line_height, typo.font_weight, typo.usage
            )?;
        }

        writeln!(f, "\n{}", "Spacing:".bold())?;
        writeln!(f, "  Padding: {}", self.spacing.padding_values.join(", "))?;
        writeln!(f, "  Margin: {}", self.spacing.margin_values.join(", "))?;
        writeln!(f, "  Gap: {}", self.spacing.gap_values.join(", "))?;

        writeln!(f, "\n{}", "Layout:".bold())?;
        writeln!(f, "  Flex containers: {}", self.layout.flex_containers)?;
        writeln!(f, "  Grid containers: {}", self.layout.grid_containers)?;
        if !self.layout.flex_directions.is_empty() {
            writeln!(
                f,
                "  Flex directions: {}",
                self.layout.flex_directions.join(", ")
            )?;
        }
        if !self.layout.grid_templates.is_empty() {
            writeln!(
                f,
                "  Grid templates: {}",
                self.layout.grid_templates.join(", ")
            )?;
        }
        if !self.layout.justify_content.is_empty() {
            writeln!(
                f,
                "  Justify content: {}",
                self.layout.justify_content.join(", ")
            )?;
        }
        if !self.layout.align_items.is_empty() {
            writeln!(f, "  Align items: {}", self.layout.align_items.join(", "))?;
        }

        writeln!(f, "\n{}", "Animations:".bold())?;
        if !self.animations.timing_functions.is_empty() {
            writeln!(
                f,
                "  Timing: {}",
                self.animations.timing_functions.join(", ")
            )?;
        }
        if !self.animations.durations.is_empty() {
            writeln!(f, "  Durations: {}", self.animations.durations.join(", "))?;
        }
        if !self.animations.transitions.is_empty() {
            writeln!(
                f,
                "  Transitions: {}",
                self.animations.transitions.join(", ")
            )?;
        }

        if let Some(path) = &self.screenshot_path {
            writeln!(f, "\nScreenshot saved: {}", path)?;
        }

        Ok(())
    }
}

/// ARIA accessibility node for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AriaNode {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub states: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<AriaNode>,
}

impl std::fmt::Display for AriaNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indent(f, 0)
    }
}

impl AriaNode {
    fn fmt_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let pad = "  ".repeat(indent);

        // Role in cyan
        write!(f, "{}{}", pad, self.role.cyan())?;

        // Name in quotes if present
        if let Some(name) = &self.name {
            if !name.is_empty() {
                write!(f, " \"{}\"", name)?;
            }
        }

        // Value if present
        if let Some(value) = &self.value {
            if !value.is_empty() {
                write!(f, " [{}]", value.dimmed())?;
            }
        }

        // States in yellow
        if !self.states.is_empty() {
            write!(f, " ({})", self.states.join(", ").yellow())?;
        }

        writeln!(f)?;

        // Children
        for child in &self.children {
            child.fmt_indent(f, indent + 1)?;
        }

        Ok(())
    }
}

/// Mask sensitive data in output
pub fn mask_sensitive(text: &str) -> String {
    // Mask common sensitive patterns
    let patterns = [
        (r"password[=:]\s*\S+", "password=****"),
        (r"token[=:]\s*\S+", "token=****"),
        (r"api[_-]?key[=:]\s*\S+", "api_key=****"),
        (r"secret[=:]\s*\S+", "secret=****"),
        (r"bearer\s+\S+", "Bearer ****"),
    ];

    let mut result = text.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", pattern)) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("test data".to_string());
        assert!(result.success);
        assert_eq!(result.data, Some("test data".to_string()));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_command_result_error() {
        let result: CommandResult<()> = CommandResult::error("test error");
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("test error".to_string()));
    }

    #[test]
    fn test_formatter_json_mode() {
        let formatter = Formatter::new(true);
        assert!(formatter.is_json());
    }

    #[test]
    fn test_formatter_human_mode() {
        let formatter = Formatter::new(false);
        assert!(!formatter.is_json());
    }
}
