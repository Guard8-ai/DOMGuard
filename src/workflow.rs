//! Workflow/macro system for DOMGuard
//!
//! Allows saving, editing, and replaying recorded sessions as reusable workflows.
//! Workflows can include parameters, conditions, and loops.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A reusable workflow (macro) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this workflow does
    #[serde(default)]
    pub description: Option<String>,

    /// Domain this workflow applies to (optional)
    #[serde(default)]
    pub domain: Option<String>,

    /// Parameters that can be passed when running
    #[serde(default)]
    pub parameters: Vec<WorkflowParameter>,

    /// The steps to execute
    pub steps: Vec<WorkflowStep>,

    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,

    /// Number of times this workflow has been run
    #[serde(default)]
    pub run_count: u32,

    /// Last run timestamp
    #[serde(default)]
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// Parameter definition for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// Parameter name (used as {{name}} in steps)
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Whether this parameter is required
    #[serde(default)]
    pub required: bool,

    /// Type hint: text, password, url, number, file
    #[serde(default = "default_param_type")]
    pub param_type: String,
}

fn default_param_type() -> String {
    "text".to_string()
}

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name for display
    #[serde(default)]
    pub name: Option<String>,

    /// Action type: click, type, navigate, wait, etc.
    pub action: String,

    /// Target selector or value (can include {{param}} placeholders)
    #[serde(default)]
    pub target: Option<String>,

    /// Value for type/input actions (can include {{param}} placeholders)
    #[serde(default)]
    pub value: Option<String>,

    /// Timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,

    /// Whether failure of this step should stop the workflow
    #[serde(default = "default_true")]
    pub required: bool,

    /// Retry count on failure
    #[serde(default)]
    pub retry_count: u32,

    /// Delay before this step (ms)
    #[serde(default)]
    pub delay_before_ms: Option<u64>,

    /// Delay after this step (ms)
    #[serde(default)]
    pub delay_after_ms: Option<u64>,

    /// Condition to execute this step (selector must exist)
    #[serde(default)]
    pub condition: Option<StepCondition>,

    /// Screenshot after this step
    #[serde(default)]
    pub screenshot_after: bool,
}

fn default_true() -> bool {
    true
}

/// Condition for conditional step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    /// Selector that must exist
    #[serde(default)]
    pub selector_exists: Option<String>,

    /// Selector that must not exist
    #[serde(default)]
    pub selector_not_exists: Option<String>,

    /// Text that must be present
    #[serde(default)]
    pub text_contains: Option<String>,

    /// URL must match pattern
    #[serde(default)]
    pub url_contains: Option<String>,
}

/// Result of running a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Workflow ID
    pub workflow_id: String,

    /// Whether all required steps succeeded
    pub success: bool,

    /// Total duration in milliseconds
    pub duration_ms: u64,

    /// Results of each step
    pub step_results: Vec<StepResult>,

    /// Any error message
    #[serde(default)]
    pub error: Option<String>,

    /// Screenshots captured during execution
    #[serde(default)]
    pub screenshots: Vec<String>,
}

/// Result of a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step index
    pub index: usize,

    /// Step name
    #[serde(default)]
    pub name: Option<String>,

    /// Whether the step succeeded
    pub success: bool,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Whether the step was skipped
    #[serde(default)]
    pub skipped: bool,

    /// Retry count used
    #[serde(default)]
    pub retries: u32,

    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,
}

/// Workflow manager for saving and loading workflows
pub struct WorkflowManager {
    workflows_dir: PathBuf,
    cache: HashMap<String, Workflow>,
}

impl WorkflowManager {
    /// Create a new manager
    pub fn new(workflows_dir: PathBuf) -> Self {
        Self {
            workflows_dir,
            cache: HashMap::new(),
        }
    }

    /// Load all workflows from disk
    pub fn load_all(&mut self) -> Result<()> {
        self.cache.clear();

        if !self.workflows_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.workflows_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(workflow) = toml::from_str::<Workflow>(&content) {
                        self.cache.insert(workflow.id.clone(), workflow);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a workflow by ID
    pub fn get(&self, id: &str) -> Option<&Workflow> {
        self.cache.get(id)
    }

    /// Get a workflow by name (partial match)
    pub fn find_by_name(&self, name: &str) -> Vec<&Workflow> {
        let name_lower = name.to_lowercase();
        self.cache
            .values()
            .filter(|w| w.name.to_lowercase().contains(&name_lower))
            .collect()
    }

    /// List all workflows
    pub fn list(&self) -> Vec<&Workflow> {
        let mut workflows: Vec<_> = self.cache.values().collect();
        workflows.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        workflows
    }

    /// List workflows by tag
    pub fn list_by_tag(&self, tag: &str) -> Vec<&Workflow> {
        self.cache
            .values()
            .filter(|w| w.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    /// List workflows for a domain
    pub fn list_for_domain(&self, domain: &str) -> Vec<&Workflow> {
        let domain_lower = domain.to_lowercase();
        self.cache
            .values()
            .filter(|w| {
                w.domain
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&domain_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Save a workflow
    pub fn save(&mut self, workflow: Workflow) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.workflows_dir)?;

        let filename = format!("{}.toml", workflow.id);
        let path = self.workflows_dir.join(filename);

        let content = toml::to_string_pretty(&workflow)?;
        std::fs::write(&path, content)?;

        self.cache.insert(workflow.id.clone(), workflow);

        Ok(path)
    }

    /// Delete a workflow
    pub fn delete(&mut self, id: &str) -> Result<bool> {
        if self.cache.remove(id).is_some() {
            let filename = format!("{}.toml", id);
            let path = self.workflows_dir.join(filename);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create a workflow from a recorded session
    pub fn from_session(session: &crate::session::Session, name: &str) -> Workflow {
        let now = chrono::Utc::now();

        let steps: Vec<WorkflowStep> = session
            .actions
            .iter()
            .map(|action| WorkflowStep {
                name: Some(action.command.clone()),
                action: action.command.clone(),
                target: action.selector.clone(),
                value: action
                    .args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                timeout_ms: Some(5000),
                required: true,
                retry_count: 0,
                delay_before_ms: None,
                delay_after_ms: Some(200),
                condition: None,
                screenshot_after: false,
            })
            .collect();

        Workflow {
            id: format!("workflow-{}", &session.id[..8]),
            name: name.to_string(),
            description: session.name.clone(),
            domain: session.initial_url.as_ref().and_then(|u| {
                u.split("://")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .map(String::from)
            }),
            parameters: vec![],
            steps,
            tags: vec!["from-session".to_string()],
            created_at: now,
            modified_at: now,
            run_count: 0,
            last_run: None,
        }
    }

    /// Create a new empty workflow
    pub fn create_empty(name: &str) -> Workflow {
        let now = chrono::Utc::now();
        let id = format!("workflow-{}", uuid_simple());

        Workflow {
            id,
            name: name.to_string(),
            description: None,
            domain: None,
            parameters: vec![],
            steps: vec![WorkflowStep {
                name: Some("Navigate to site".to_string()),
                action: "navigate".to_string(),
                target: Some("{{url}}".to_string()),
                value: None,
                timeout_ms: Some(10000),
                required: true,
                retry_count: 0,
                delay_before_ms: None,
                delay_after_ms: Some(1000),
                condition: None,
                screenshot_after: false,
            }],
            tags: vec![],
            created_at: now,
            modified_at: now,
            run_count: 0,
            last_run: None,
        }
    }

    /// Update run statistics
    pub fn record_run(&mut self, id: &str, _success: bool) -> Result<()> {
        if let Some(workflow) = self.cache.get_mut(id) {
            workflow.run_count += 1;
            workflow.last_run = Some(chrono::Utc::now());

            // Save to disk
            let filename = format!("{}.toml", id);
            let path = self.workflows_dir.join(filename);
            let content = toml::to_string_pretty(&workflow)?;
            std::fs::write(&path, content)?;
        }
        Ok(())
    }
}

/// Substitute parameters in a string
pub fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Generate a simple UUID-like ID
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", now)[..12].to_string()
}

/// Format workflow for display
pub fn format_workflow(workflow: &Workflow) -> String {
    let mut output = String::new();

    output.push_str(&format!("Workflow: {} ({})\n", workflow.name, workflow.id));

    if let Some(desc) = &workflow.description {
        output.push_str(&format!("  Description: {}\n", desc));
    }

    if let Some(domain) = &workflow.domain {
        output.push_str(&format!("  Domain: {}\n", domain));
    }

    if !workflow.tags.is_empty() {
        output.push_str(&format!("  Tags: {}\n", workflow.tags.join(", ")));
    }

    output.push_str(&format!("  Steps: {}\n", workflow.steps.len()));
    output.push_str(&format!("  Run count: {}\n", workflow.run_count));

    if !workflow.parameters.is_empty() {
        output.push_str("\n  Parameters:\n");
        for param in &workflow.parameters {
            let req = if param.required { "*" } else { "" };
            output.push_str(&format!(
                "    - {}{}: {} ({})\n",
                param.name,
                req,
                param.description.as_deref().unwrap_or(""),
                param.param_type
            ));
        }
    }

    output.push_str("\n  Steps:\n");
    for (i, step) in workflow.steps.iter().enumerate() {
        let name = step.name.as_deref().unwrap_or(&step.action);
        let target = step.target.as_deref().unwrap_or("");
        output.push_str(&format!("    {}. {} {}\n", i + 1, name, target));
    }

    output
}

/// Format workflow list for display
pub fn format_workflow_list(workflows: &[&Workflow]) -> String {
    let mut output = String::new();

    for workflow in workflows {
        let domain = workflow.domain.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "  {} - {} ({} steps, {})\n",
            workflow.id,
            workflow.name,
            workflow.steps.len(),
            domain
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_params() {
        let mut params = HashMap::new();
        params.insert("username".to_string(), "john".to_string());
        params.insert("password".to_string(), "secret".to_string());

        let result =
            substitute_params("Hello {{username}}, your password is {{password}}", &params);
        assert_eq!(result, "Hello john, your password is secret");
    }

    #[test]
    fn test_create_empty_workflow() {
        let workflow = WorkflowManager::create_empty("Test Workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert!(!workflow.steps.is_empty());
    }

    #[test]
    fn test_serialize_workflow() {
        let workflow = WorkflowManager::create_empty("Test");
        let toml = toml::to_string_pretty(&workflow).unwrap();
        let parsed: Workflow = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.name, "Test");
    }
}
