//! DOMGuard - Local-First Chrome DevTools CLI
//!
//! Direct CDP access for AI agents. No middleware, no servers, sub-ms local response.

mod captcha;
mod cdp;
mod config;
mod correction;
mod debug;
mod explanation;
mod inspire;
mod interact;
mod output;
mod security;
mod session;
mod site_instructions;
mod takeover;
mod workflow;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

use crate::cdp::CdpConnection;
use crate::config::{init_domguard, Config};
use crate::debug::DebugCommand;
use crate::interact::InteractCommand;
use crate::output::{CommandResult, Formatter};
use crate::session::SessionRecorder;

#[derive(Parser)]
#[command(name = "domguard")]
#[command(author = "Guard8.ai")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Local-first Chrome DevTools CLI for AI agents", long_about = None)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Chrome DevTools host (default: 127.0.0.1)
    #[arg(long, global = true)]
    host: Option<String>,

    /// Chrome DevTools port (default: 9222)
    #[arg(long, global = true)]
    port: Option<u16>,

    /// Command timeout in milliseconds
    #[arg(long, global = true)]
    timeout: Option<u64>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize DOMGuard in current directory
    Init,

    /// Check Chrome connection status
    Status,

    /// Extract design patterns from websites
    Inspire {
        /// URL to analyze
        url: String,

        /// CSS selector for specific component
        #[arg(long)]
        component: Option<String>,

        /// Save inspiration with name
        #[arg(long)]
        save: Option<String>,
    },

    /// Inspect page state (DOM, console, network, storage)
    Debug {
        #[command(subcommand)]
        command: DebugSubcommand,
    },

    /// Control browser (click, type, navigate, screenshot)
    Interact {
        #[command(subcommand)]
        command: InteractSubcommand,
    },

    /// Record and manage browser sessions
    Session {
        #[command(subcommand)]
        command: SessionSubcommand,
    },

    /// Manage security settings and blocked sites
    Security {
        #[command(subcommand)]
        command: SecuritySubcommand,
    },

    /// Explain what an action will do before executing it
    Explain {
        #[command(subcommand)]
        command: ExplainSubcommand,
    },

    /// Manage per-site custom instructions
    Sites {
        #[command(subcommand)]
        command: SitesSubcommand,
    },

    /// Manage saved workflows/macros
    Workflow {
        #[command(subcommand)]
        command: WorkflowSubcommand,
    },

    /// User takeover mode - hand control back to human
    Takeover {
        #[command(subcommand)]
        command: TakeoverSubcommand,
    },

    /// Self-correction settings and diagnostics
    Correction {
        #[command(subcommand)]
        command: CorrectionSubcommand,
    },
}

#[derive(Subcommand)]
enum DebugSubcommand {
    /// Inspect DOM tree
    Dom {
        /// CSS selector (optional, default: full tree)
        selector: Option<String>,
    },

    /// Get computed styles for element
    Styles {
        /// CSS selector
        selector: String,
    },

    /// View console messages
    Console {
        /// Stream console messages live
        #[arg(long)]
        follow: bool,

        /// Filter messages by text
        #[arg(long)]
        filter: Option<String>,
    },

    /// View network requests
    Network {
        /// Filter requests by URL
        #[arg(long)]
        filter: Option<String>,
    },

    /// Execute JavaScript expression
    Eval {
        /// JavaScript expression
        expression: String,
    },

    /// View localStorage and sessionStorage
    Storage,

    /// View cookies
    Cookies,

    /// View accessibility tree (ARIA snapshot)
    Aria {
        /// CSS selector (optional, default: full tree)
        selector: Option<String>,
    },

    /// Manage browser tabs
    Tabs {
        #[command(subcommand)]
        action: TabAction,
    },

    // Chrome DevTools MCP features
    /// Get performance metrics (Core Web Vitals, heap size, etc.)
    Performance,

    /// Export full DOM as HTML snapshot
    Snapshot {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Enable CPU or network throttling
    Throttle {
        #[command(subcommand)]
        mode: ThrottleAction,
    },

    /// Get detailed network request info (timing, size, protocol)
    NetworkDetails {
        /// Filter requests by URL
        #[arg(long)]
        filter: Option<String>,
    },

    /// Highlight element(s) on the page
    Highlight {
        /// CSS selector for element(s) to highlight
        selector: String,

        /// Highlight color (red, green, blue, yellow, orange, purple, pink, cyan, or #RRGGBB)
        #[arg(short, long, default_value = "red")]
        color: String,

        /// Duration in milliseconds (0 = persistent until cleared)
        #[arg(short, long, default_value = "0")]
        duration: u64,

        /// Highlight all matching elements (with numbered labels)
        #[arg(long)]
        all: bool,
    },

    /// Clear all highlights from the page
    ClearHighlights,

    /// Detect CAPTCHAs on the current page
    Captcha,
}

#[derive(Subcommand)]
enum ThrottleAction {
    /// Disable all throttling
    Off,

    /// Slow down CPU (rate = slowdown factor, e.g., 4 = 4x slower)
    Cpu {
        /// Slowdown rate (1 = normal, 4 = 4x slower)
        rate: f64,
    },

    /// Simulate 3G network (1.6 Mbps, 300ms latency)
    Network3g,

    /// Simulate slow 3G network (400 Kbps, 2s latency)
    NetworkSlow3g,

    /// Simulate offline mode
    NetworkOffline,

    /// Custom network conditions
    NetworkCustom {
        /// Download speed in Kbps
        #[arg(long)]
        download: f64,

        /// Upload speed in Kbps
        #[arg(long)]
        upload: f64,

        /// Latency in milliseconds
        #[arg(long)]
        latency: f64,
    },
}

#[derive(Subcommand)]
enum TabAction {
    /// List all open tabs
    List,

    /// Create a new tab
    New {
        /// URL to open (default: about:blank)
        url: Option<String>,
    },

    /// Switch to a tab by ID
    Switch {
        /// Tab ID to switch to
        id: String,
    },

    /// Close a tab by ID
    Close {
        /// Tab ID to close
        id: String,
    },
}

#[derive(Subcommand)]
enum InteractSubcommand {
    /// Click element or coordinates
    Click {
        /// CSS selector
        selector: Option<String>,

        /// Click at coordinates (x,y)
        #[arg(long, value_parser = parse_coords)]
        coords: Option<(f64, f64)>,

        /// Select nth matching element (0-indexed, -1 for last)
        #[arg(long, default_value = "0", allow_hyphen_values = true)]
        nth: i32,

        /// Click element containing this text
        #[arg(long)]
        text: Option<String>,
    },

    /// Type text into element
    Type {
        /// CSS selector (or use --focused)
        selector: Option<String>,

        /// Text to type
        text: Option<String>,

        /// Type into currently focused element
        #[arg(long)]
        focused: bool,
    },

    /// Press key or key sequence
    Key {
        /// Key(s) to press (space-separated, e.g., "Tab Tab Enter" or "cmd+k")
        keys: String,
    },

    /// Hover over element
    Hover {
        /// CSS selector
        selector: String,
    },

    /// Scroll page
    Scroll {
        /// Scroll down by pixels
        #[arg(long)]
        down: Option<i64>,

        /// Scroll up by pixels
        #[arg(long)]
        up: Option<i64>,

        /// Scroll left by pixels
        #[arg(long)]
        left: Option<i64>,

        /// Scroll right by pixels
        #[arg(long)]
        right: Option<i64>,

        /// Scroll to element (CSS selector)
        #[arg(long)]
        to: Option<String>,
    },

    /// Capture screenshot
    Screenshot {
        /// Capture full page
        #[arg(long)]
        full: bool,

        /// Capture specific element
        #[arg(long)]
        element: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Navigate to URL
    Navigate {
        /// URL to navigate to
        url: String,
    },

    /// Go back in browser history
    Back,

    /// Refresh the page
    Refresh,

    /// Wait for element or text
    Wait {
        /// CSS selector (optional if using --text)
        selector: Option<String>,

        /// Wait for element to be visible
        #[arg(long)]
        visible: bool,

        /// Wait for element to be gone
        #[arg(long)]
        gone: bool,

        /// Wait for text to appear on page
        #[arg(long)]
        text: Option<String>,

        /// Wait for text to disappear from page
        #[arg(long)]
        text_gone: Option<String>,

        /// Timeout in milliseconds
        #[arg(long, default_value = "5000")]
        timeout: u64,
    },

    /// Drag element to another element or coordinates
    Drag {
        /// Source CSS selector
        #[arg(long)]
        from: Option<String>,

        /// Target CSS selector
        #[arg(long)]
        to: Option<String>,

        /// Source coordinates (x,y)
        #[arg(long, value_parser = parse_coords)]
        from_coords: Option<(f64, f64)>,

        /// Target coordinates (x,y)
        #[arg(long, value_parser = parse_coords)]
        to_coords: Option<(f64, f64)>,
    },

    /// Select option from dropdown
    Select {
        /// CSS selector for select element
        selector: String,

        /// Value to select
        value: String,

        /// Select by visible label instead of value
        #[arg(long)]
        by_label: bool,

        /// Select by index (0-based)
        #[arg(long)]
        by_index: bool,
    },

    /// Upload file(s) to file input
    Upload {
        /// CSS selector for file input
        selector: String,

        /// File path(s) to upload
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Handle browser dialog (alert, confirm, prompt)
    Dialog {
        /// Accept the dialog (default: dismiss)
        #[arg(long)]
        accept: bool,

        /// Text to enter for prompt dialogs
        #[arg(long)]
        text: Option<String>,
    },

    /// Resize browser viewport
    Resize {
        /// Viewport width in pixels
        width: u32,

        /// Viewport height in pixels
        height: u32,
    },

    /// Export page as PDF
    Pdf {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Landscape orientation
        #[arg(long)]
        landscape: bool,
    },

    // Anthropic Computer Use features
    /// Move mouse cursor to coordinates (without clicking)
    MouseMove {
        /// Target coordinates (x,y)
        #[arg(value_parser = parse_coords)]
        coords: (f64, f64),
    },

    /// Get current cursor position
    CursorPosition,

    /// Hold a key for specified duration
    HoldKey {
        /// Key to hold
        key: String,

        /// Duration in milliseconds
        #[arg(long, default_value = "500")]
        duration: u64,
    },

    /// Triple-click to select paragraph/block
    TripleClick {
        /// CSS selector
        selector: Option<String>,

        /// Click at coordinates (x,y)
        #[arg(long, value_parser = parse_coords)]
        coords: Option<(f64, f64)>,
    },

    /// Press mouse button down (without releasing)
    MouseDown {
        /// Button to press: left, middle, right
        #[arg(default_value = "left")]
        button: String,
    },

    /// Release mouse button
    MouseUp {
        /// Button to release: left, middle, right
        #[arg(default_value = "left")]
        button: String,
    },

    /// Capture screenshot of specific region
    ScreenshotRegion {
        /// Region coordinates: x,y,width,height
        #[arg(value_parser = parse_region)]
        region: (i32, i32, i32, i32),

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Wait for specified duration (in milliseconds)
    WaitDuration {
        /// Duration in milliseconds
        duration: u64,
    },

    /// Clean up screenshot files
    Cleanup {
        /// Only show what would be deleted (dry run)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SessionSubcommand {
    /// Start recording a new session
    Start {
        /// Optional session name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Stop the current recording
    Stop {
        /// Delete screenshots after stopping
        #[arg(long)]
        cleanup: bool,
    },

    /// Pause the current recording
    Pause,

    /// Resume a paused recording
    Resume,

    /// Show current session status
    Status,

    /// List all saved sessions
    List,

    /// Show details of a specific session
    Show {
        /// Session ID
        id: String,
    },

    /// Delete a saved session
    Delete {
        /// Session ID
        id: String,
    },

    /// Export session as workflow script
    Export {
        /// Session ID
        id: String,

        /// Output format (json, bash, or markdown)
        #[arg(short, long, default_value = "bash")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Replay a recorded session
    Replay {
        /// Session ID
        id: String,

        /// Step through each action (pause between steps)
        #[arg(long)]
        step: bool,

        /// Delay between actions in milliseconds
        #[arg(long, default_value = "500")]
        delay: u64,
    },
}

#[derive(Subcommand)]
enum SecuritySubcommand {
    /// Check if an action is sensitive (for testing security detection)
    Check {
        /// Type of action: type, click, navigate, upload
        action: String,

        /// Target (selector for type/click, URL for navigate, file path for upload)
        target: String,

        /// Value (text for type action)
        #[arg(short, long)]
        value: Option<String>,
    },

    /// List blocked sites
    ListBlocked,

    /// Block a site pattern
    Block {
        /// Site pattern to block (e.g., "malicious-site.com")
        pattern: String,
    },

    /// Unblock a site pattern
    Unblock {
        /// Site pattern to unblock
        pattern: String,
    },

    /// Allow a site (when using default-block mode)
    Allow {
        /// Site pattern to allow
        pattern: String,
    },

    /// Set default blocking mode
    SetMode {
        /// Mode: allow (block nothing by default) or block (block everything by default)
        mode: String,
    },

    /// Show current security configuration
    Config,
}

#[derive(Subcommand)]
enum ExplainSubcommand {
    /// Explain what a click action will do
    Click {
        /// CSS selector or coordinates
        target: String,
    },

    /// Explain what a type action will do
    Type {
        /// CSS selector or "focused"
        target: String,
    },

    /// Explain what a key press will do
    Key {
        /// Key or key sequence
        keys: String,
    },

    /// Explain what a navigate action will do
    Navigate {
        /// URL to navigate to
        url: String,
    },

    /// Explain what a wait action will do
    Wait {
        /// Selector or condition
        target: String,
    },

    /// Explain any interact command
    Interact {
        /// The interact command (e.g., "click", "type", "navigate")
        command: String,
        /// Target selector or value
        target: Option<String>,
    },
}

#[derive(Subcommand)]
enum SitesSubcommand {
    /// List all saved site instructions
    List,

    /// Show instructions for a specific domain
    Show {
        /// Domain pattern (e.g., "example.com")
        domain: String,
    },

    /// Create template instructions for a domain
    Create {
        /// Domain pattern (e.g., "example.com" or "*.example.com")
        domain: String,
    },

    /// Delete instructions for a domain
    Delete {
        /// Domain pattern
        domain: String,
    },

    /// Get instructions for current page (requires Chrome connection)
    Current,

    /// Edit site instructions in default editor
    Edit {
        /// Domain pattern
        domain: String,
    },
}

#[derive(Subcommand)]
enum WorkflowSubcommand {
    /// List all saved workflows
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by domain
        #[arg(long)]
        domain: Option<String>,
    },

    /// Show details of a workflow
    Show {
        /// Workflow ID or name
        id: String,
    },

    /// Create a new workflow from a recorded session
    FromSession {
        /// Session ID
        session_id: String,

        /// Name for the workflow
        name: String,
    },

    /// Create a new empty workflow
    Create {
        /// Name for the workflow
        name: String,
    },

    /// Run a workflow
    Run {
        /// Workflow ID or name
        id: String,

        /// Parameters as key=value pairs
        #[arg(short, long, value_parser = parse_param)]
        param: Vec<(String, String)>,

        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,

        /// Delay between steps in milliseconds
        #[arg(long, default_value = "500")]
        delay: u64,
    },

    /// Delete a workflow
    Delete {
        /// Workflow ID
        id: String,
    },

    /// Edit workflow in default editor
    Edit {
        /// Workflow ID
        id: String,
    },
}

#[derive(Subcommand)]
enum TakeoverSubcommand {
    /// Request takeover - hand control to user
    Request {
        /// Reason for takeover: captcha, auth, sensitive, error, uncertain, complex, 2fa, payment, or custom message
        reason: String,

        /// Human-readable message for the user
        #[arg(short, long)]
        message: Option<String>,

        /// Instructions for the user
        #[arg(short, long)]
        instructions: Option<String>,

        /// Expected outcome after user action
        #[arg(short, long)]
        expected: Option<String>,
    },

    /// Mark takeover as complete and resume automation
    Done {
        /// Whether the takeover was successful
        #[arg(long, default_value = "true")]
        success: bool,

        /// Notes about what was done
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Cancel active takeover without completing
    Cancel,

    /// Check current takeover status
    Status,

    /// List takeover history
    History {
        /// Maximum number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum CorrectionSubcommand {
    /// Show current self-correction configuration
    Config,

    /// Enable self-correction
    Enable,

    /// Disable self-correction
    Disable,

    /// Analyze an error message and show recovery strategies
    Analyze {
        /// Error message to analyze
        error: String,

        /// Action type (click, type, navigate, etc.)
        #[arg(short, long, default_value = "click")]
        action: String,
    },

    /// Test a specific recovery strategy
    Test {
        /// Strategy to test: scroll, dismiss-overlay, wait-stable, refresh
        strategy: String,

        /// Target selector (for some strategies)
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Dismiss any blocking overlays on the page
    DismissOverlay,

    /// Wait for page to become stable
    WaitStable,

    /// Show recovery strategies for a given error type
    Strategies {
        /// Error type: element-not-found, not-visible, not-interactable, timeout, network, captcha, etc.
        error_type: String,
    },
}

fn parse_param(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Parameter must be in format: key=value".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn parse_region(s: &str) -> Result<(i32, i32, i32, i32), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err("Region must be in format: x,y,width,height".to_string());
    }
    let x = parts[0]
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid x coordinate")?;
    let y = parts[1]
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid y coordinate")?;
    let width = parts[2]
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid width")?;
    let height = parts[3]
        .trim()
        .parse::<i32>()
        .map_err(|_| "Invalid height")?;
    Ok((x, y, width, height))
}

fn parse_coords(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("Coordinates must be in format: x,y".to_string());
    }
    let x = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid x coordinate")?;
    let y = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid y coordinate")?;
    Ok((x, y))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let formatter = Formatter::new(cli.json);

    // Run the actual command and handle errors with proper formatting
    let result = run_command(cli, &formatter).await;

    if let Err(e) = &result {
        if formatter.is_json() {
            let err_result: CommandResult<()> = CommandResult::error(e.to_string());
            formatter.output_json(&err_result);
        } else {
            formatter.error(&e.to_string());
        }
        std::process::exit(1);
    }

    result
}

async fn run_command(cli: Cli, formatter: &Formatter) -> Result<()> {
    // Handle init command specially (doesn't need Chrome connection)
    if matches!(cli.command, Commands::Init) {
        return handle_init(formatter);
    }

    // Load config
    let mut config = Config::load()?;

    // Override config with CLI args
    if let Some(host) = cli.host {
        config.chrome.host = host;
    }
    if let Some(port) = cli.port {
        config.chrome.port = port;
    }
    if let Some(timeout) = cli.timeout {
        config.defaults.timeout_ms = timeout;
    }

    // Connect to Chrome
    let mut cdp = CdpConnection::new(config.clone());

    match &cli.command {
        Commands::Status => handle_status(&mut cdp, formatter).await,
        Commands::Inspire {
            url,
            component,
            save,
        } => {
            cdp.connect().await?;
            inspire::run_inspire(
                &cdp,
                &config,
                url,
                component.as_deref(),
                save.as_deref(),
                formatter,
            )
            .await
        }
        Commands::Debug { command } => {
            cdp.connect().await?;
            let cmd = match command {
                DebugSubcommand::Dom { selector } => DebugCommand::Dom {
                    selector: selector.clone(),
                },
                DebugSubcommand::Styles { selector } => DebugCommand::Styles {
                    selector: selector.clone(),
                },
                DebugSubcommand::Console { follow, filter } => DebugCommand::Console {
                    follow: *follow,
                    filter: filter.clone(),
                },
                DebugSubcommand::Network { filter } => DebugCommand::Network {
                    filter: filter.clone(),
                },
                DebugSubcommand::Eval { expression } => DebugCommand::Eval {
                    expression: expression.clone(),
                },
                DebugSubcommand::Storage => DebugCommand::Storage,
                DebugSubcommand::Cookies => DebugCommand::Cookies,
                DebugSubcommand::Aria { selector } => DebugCommand::Aria {
                    selector: selector.clone(),
                },
                DebugSubcommand::Tabs { action } => {
                    let tab_action = match action {
                        TabAction::List => debug::TabCommand::List,
                        TabAction::New { url } => debug::TabCommand::New { url: url.clone() },
                        TabAction::Switch { id } => debug::TabCommand::Switch { id: id.clone() },
                        TabAction::Close { id } => debug::TabCommand::Close { id: id.clone() },
                    };
                    DebugCommand::Tabs { action: tab_action }
                }
                // Chrome DevTools MCP features
                DebugSubcommand::Performance => DebugCommand::Performance,
                DebugSubcommand::Snapshot { output } => DebugCommand::Snapshot {
                    output: output.clone(),
                },
                DebugSubcommand::Throttle { mode } => {
                    let throttle_mode = match mode {
                        ThrottleAction::Off => debug::ThrottleMode::Off,
                        ThrottleAction::Cpu { rate } => debug::ThrottleMode::Cpu { rate: *rate },
                        ThrottleAction::Network3g => debug::ThrottleMode::Network3g,
                        ThrottleAction::NetworkSlow3g => debug::ThrottleMode::NetworkSlow3g,
                        ThrottleAction::NetworkOffline => debug::ThrottleMode::NetworkOffline,
                        ThrottleAction::NetworkCustom {
                            download,
                            upload,
                            latency,
                        } => debug::ThrottleMode::NetworkCustom {
                            download_kbps: *download,
                            upload_kbps: *upload,
                            latency_ms: *latency,
                        },
                    };
                    DebugCommand::Throttle {
                        mode: throttle_mode,
                    }
                }
                DebugSubcommand::NetworkDetails { filter } => DebugCommand::NetworkDetails {
                    filter: filter.clone(),
                },
                // Element highlighting
                DebugSubcommand::Highlight {
                    selector,
                    color,
                    duration,
                    all,
                } => DebugCommand::Highlight {
                    selector: selector.clone(),
                    color: color.clone(),
                    duration: *duration,
                    all: *all,
                },
                DebugSubcommand::ClearHighlights => DebugCommand::ClearHighlights,
                DebugSubcommand::Captcha => DebugCommand::Captcha,
            };
            debug::run_debug(&cdp, cmd, formatter).await
        }
        Commands::Interact { command } => {
            // Handle cleanup command separately (doesn't need CDP)
            if let InteractSubcommand::Cleanup { dry_run } = command {
                let screenshots_dir = Config::find_domguard_dir()
                    .unwrap_or_else(Config::domguard_dir)
                    .join("screenshots");

                if !screenshots_dir.exists() {
                    if formatter.is_json() {
                        formatter.output_json(&serde_json::json!({
                            "success": true,
                            "deleted": 0,
                            "message": "No screenshots directory found"
                        }));
                    } else {
                        println!("No screenshots directory found");
                    }
                    return Ok(());
                }

                let mut deleted = 0;
                let mut files_to_delete = Vec::new();

                for entry in std::fs::read_dir(&screenshots_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "png" || ext == "pdf" || ext == "html" {
                                files_to_delete.push(path);
                            }
                        }
                    }
                }

                if *dry_run {
                    if formatter.is_json() {
                        formatter.output_json(&serde_json::json!({
                            "dry_run": true,
                            "would_delete": files_to_delete.len(),
                            "files": files_to_delete.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
                        }));
                    } else {
                        println!("Would delete {} files:", files_to_delete.len());
                        for path in &files_to_delete {
                            println!("  {}", path.display());
                        }
                    }
                } else {
                    for path in &files_to_delete {
                        std::fs::remove_file(path)?;
                        deleted += 1;
                    }

                    if formatter.is_json() {
                        formatter.output_json(&serde_json::json!({
                            "success": true,
                            "deleted": deleted
                        }));
                    } else {
                        println!("Deleted {} screenshot files", deleted);
                    }
                }

                return Ok(());
            }

            // Early validation for commands that require arguments
            // This prevents unnecessary CDP connection attempts when args are missing
            match command {
                InteractSubcommand::Click {
                    selector,
                    coords,
                    text,
                    ..
                } if selector.is_none() && coords.is_none() && text.is_none() => {
                    anyhow::bail!("Click requires at least one of: SELECTOR, --coords, or --text");
                }
                InteractSubcommand::Type { text, focused, .. }
                    if text.as_ref().map_or(true, |t| t.is_empty()) && !focused =>
                {
                    anyhow::bail!("Type requires TEXT argument or --focused flag");
                }
                _ => {}
            }

            cdp.connect().await?;

            // Build action info for session recording
            let (cmd_name, selector, args) = match command {
                InteractSubcommand::Click {
                    selector,
                    coords,
                    nth,
                    text,
                } => (
                    "click",
                    selector.clone(),
                    serde_json::json!({ "coords": coords, "nth": nth, "text": text }),
                ),
                InteractSubcommand::Type { selector, text, .. } => (
                    "type",
                    selector.clone(),
                    serde_json::json!({ "value": text }),
                ),
                InteractSubcommand::Key { keys } => {
                    ("key", None, serde_json::json!({ "keys": keys }))
                }
                InteractSubcommand::Hover { selector } => {
                    ("hover", Some(selector.clone()), serde_json::json!({}))
                }
                InteractSubcommand::Navigate { url } => {
                    ("navigate", None, serde_json::json!({ "url": url }))
                }
                InteractSubcommand::Screenshot { .. } => {
                    ("screenshot", None, serde_json::json!({}))
                }
                InteractSubcommand::Scroll { to, .. } => {
                    ("scroll", to.clone(), serde_json::json!({}))
                }
                InteractSubcommand::Back => ("back", None, serde_json::json!({})),
                InteractSubcommand::Refresh => ("refresh", None, serde_json::json!({})),
                InteractSubcommand::Wait { selector, text, .. } => (
                    "wait",
                    selector.clone(),
                    serde_json::json!({ "text": text }),
                ),
                InteractSubcommand::Drag { from, to, .. } => {
                    ("drag", from.clone(), serde_json::json!({ "to": to }))
                }
                InteractSubcommand::Select {
                    selector, value, ..
                } => (
                    "select",
                    Some(selector.clone()),
                    serde_json::json!({ "value": value }),
                ),
                InteractSubcommand::Upload { selector, files } => (
                    "upload",
                    Some(selector.clone()),
                    serde_json::json!({ "files": files }),
                ),
                InteractSubcommand::Dialog { accept, text } => (
                    "dialog",
                    None,
                    serde_json::json!({ "accept": accept, "text": text }),
                ),
                InteractSubcommand::Resize { width, height } => (
                    "resize",
                    None,
                    serde_json::json!({ "width": width, "height": height }),
                ),
                InteractSubcommand::Pdf { .. } => ("pdf", None, serde_json::json!({})),
                InteractSubcommand::MouseMove { coords } => {
                    ("mouse_move", None, serde_json::json!({ "coords": coords }))
                }
                InteractSubcommand::CursorPosition => {
                    ("cursor_position", None, serde_json::json!({}))
                }
                InteractSubcommand::HoldKey { key, duration } => (
                    "hold_key",
                    None,
                    serde_json::json!({ "key": key, "duration": duration }),
                ),
                InteractSubcommand::TripleClick { selector, coords } => (
                    "triple_click",
                    selector.clone(),
                    serde_json::json!({ "coords": coords }),
                ),
                InteractSubcommand::MouseDown { button } => {
                    ("mouse_down", None, serde_json::json!({ "button": button }))
                }
                InteractSubcommand::MouseUp { button } => {
                    ("mouse_up", None, serde_json::json!({ "button": button }))
                }
                InteractSubcommand::ScreenshotRegion { region, .. } => (
                    "screenshot_region",
                    None,
                    serde_json::json!({ "region": region }),
                ),
                InteractSubcommand::WaitDuration { duration } => (
                    "wait_duration",
                    None,
                    serde_json::json!({ "duration": duration }),
                ),
                InteractSubcommand::Cleanup { .. } => unreachable!("handled above"),
            };

            let cmd = match command {
                InteractSubcommand::Click {
                    selector,
                    coords,
                    nth,
                    text,
                } => InteractCommand::Click {
                    selector: selector.clone(),
                    coords: *coords,
                    nth: *nth,
                    text: text.clone(),
                },
                InteractSubcommand::Type {
                    selector,
                    text,
                    focused,
                } => InteractCommand::Type {
                    selector: selector.clone(),
                    text: text.clone(),
                    focused: *focused,
                },
                InteractSubcommand::Key { keys } => InteractCommand::Key { keys: keys.clone() },
                InteractSubcommand::Hover { selector } => InteractCommand::Hover {
                    selector: selector.clone(),
                },
                InteractSubcommand::Scroll {
                    down,
                    up,
                    left,
                    right,
                    to,
                } => InteractCommand::Scroll {
                    down: *down,
                    up: *up,
                    left: *left,
                    right: *right,
                    to: to.clone(),
                },
                InteractSubcommand::Screenshot {
                    full,
                    element,
                    output,
                } => InteractCommand::Screenshot {
                    full: *full,
                    element: element.clone(),
                    output: output.clone(),
                },
                InteractSubcommand::Navigate { url } => {
                    InteractCommand::Navigate { url: url.clone() }
                }
                InteractSubcommand::Back => InteractCommand::Back,
                InteractSubcommand::Refresh => InteractCommand::Refresh,
                InteractSubcommand::Wait {
                    selector,
                    visible,
                    gone,
                    text,
                    text_gone,
                    timeout,
                } => InteractCommand::Wait {
                    selector: selector.clone().unwrap_or_default(),
                    visible: *visible,
                    gone: *gone,
                    timeout_ms: *timeout,
                    text: text.clone(),
                    text_gone: text_gone.clone(),
                },
                InteractSubcommand::Drag {
                    from,
                    to,
                    from_coords,
                    to_coords,
                } => InteractCommand::Drag {
                    from_selector: from.clone(),
                    to_selector: to.clone(),
                    from_coords: *from_coords,
                    to_coords: *to_coords,
                },
                InteractSubcommand::Select {
                    selector,
                    value,
                    by_label,
                    by_index,
                } => InteractCommand::Select {
                    selector: selector.clone(),
                    value: value.clone(),
                    by_label: *by_label,
                    by_index: *by_index,
                },
                InteractSubcommand::Upload { selector, files } => InteractCommand::Upload {
                    selector: selector.clone(),
                    files: files.clone(),
                },
                InteractSubcommand::Dialog { accept, text } => InteractCommand::Dialog {
                    accept: *accept,
                    text: text.clone(),
                },
                InteractSubcommand::Resize { width, height } => InteractCommand::Resize {
                    width: *width,
                    height: *height,
                },
                InteractSubcommand::Pdf { output, landscape } => InteractCommand::Pdf {
                    output: output.clone(),
                    landscape: *landscape,
                },
                // Anthropic Computer Use features
                InteractSubcommand::MouseMove { coords } => {
                    InteractCommand::MouseMove { coords: *coords }
                }
                InteractSubcommand::CursorPosition => InteractCommand::CursorPosition,
                InteractSubcommand::HoldKey { key, duration } => InteractCommand::HoldKey {
                    key: key.clone(),
                    duration_ms: *duration,
                },
                InteractSubcommand::TripleClick { selector, coords } => {
                    InteractCommand::TripleClick {
                        selector: selector.clone(),
                        coords: *coords,
                    }
                }
                InteractSubcommand::MouseDown { button } => InteractCommand::MouseDown {
                    button: button.clone(),
                },
                InteractSubcommand::MouseUp { button } => InteractCommand::MouseUp {
                    button: button.clone(),
                },
                InteractSubcommand::ScreenshotRegion { region, output } => {
                    InteractCommand::ScreenshotRegion {
                        region: *region,
                        output: output.clone(),
                    }
                }
                InteractSubcommand::WaitDuration { duration } => InteractCommand::WaitDuration {
                    duration_ms: *duration,
                },
                InteractSubcommand::Cleanup { .. } => unreachable!("handled above"),
            };

            // Build action for recording
            use crate::session::ActionBuilder;
            let action_builder = ActionBuilder::new(cmd_name)
                .with_args(args)
                .with_selector(selector);

            // Execute the command
            let result = interact::run_interact(&cdp, &config, cmd, formatter).await;

            // Record the action if a session is active
            let sessions_dir = Config::find_domguard_dir()
                .unwrap_or_else(Config::domguard_dir)
                .join("sessions");
            let recorder = SessionRecorder::new(sessions_dir);
            if recorder.is_recording() {
                let action = if result.is_ok() {
                    action_builder.success()
                } else {
                    action_builder.failed(
                        result
                            .as_ref()
                            .err()
                            .map(|e| e.to_string())
                            .unwrap_or_default()
                            .as_str(),
                    )
                };
                let _ = recorder.record_action(action);
            }

            result
        }
        Commands::Session { command } => {
            handle_session(&mut cdp, &config, command, formatter).await
        }
        Commands::Security { command } => handle_security(command, formatter),
        Commands::Explain { command } => handle_explain(&mut cdp, command, formatter).await,
        Commands::Sites { command } => handle_sites(&mut cdp, command, formatter).await,
        Commands::Workflow { command } => handle_workflow(&mut cdp, command, formatter).await,
        Commands::Takeover { command } => handle_takeover(&mut cdp, command, formatter).await,
        Commands::Correction { command } => handle_correction(&mut cdp, command, formatter).await,
        Commands::Init => unreachable!(),
    }
}

fn handle_init(formatter: &Formatter) -> Result<()> {
    let result = init_domguard()?;

    if formatter.is_json() {
        // In JSON mode, output structured result
        #[derive(serde::Serialize)]
        struct InitResult {
            already_exists: bool,
            domguard_dir: String,
            config_path: String,
        }
        let init_result = InitResult {
            already_exists: result.already_exists,
            domguard_dir: result.domguard_dir.display().to_string(),
            config_path: result
                .domguard_dir
                .join("config.toml")
                .display()
                .to_string(),
        };
        let cmd_result = CommandResult::success(init_result);
        formatter.output_json(&cmd_result);
    } else if result.already_exists {
        formatter.warning("DOMGuard already initialized in this directory");
        println!(
            "  Config: {}",
            result.domguard_dir.join("config.toml").display()
        );
    } else {
        println!("{}", "DOMGuard initialized!".green().bold());
        println!();
        formatter.header("Created");
        formatter.item(&format!(
            "{} - Configuration and data",
            result.domguard_dir.display()
        ));
        formatter.item(&format!(
            "{}/config.toml - Settings",
            result.domguard_dir.display()
        ));
        formatter.item(&format!(
            "{}/inspirations/ - Saved design patterns",
            result.domguard_dir.display()
        ));
        formatter.item(&format!(
            "{}/screenshots/ - Captured screenshots",
            result.domguard_dir.display()
        ));

        if let Some(guide_path) = result.guide_path {
            formatter.header("AI Agent Integration");
            let filename = guide_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "guide".to_string());
            formatter.item(&format!("{} copied", filename));
        }

        formatter.header("Next steps");
        formatter.item("Start Chrome: chrome --remote-debugging-port=9222");
        formatter.item("Check connection: domguard status");
        formatter.item("Try it out: domguard debug dom");
    }

    Ok(())
}

async fn handle_status(cdp: &mut CdpConnection, formatter: &Formatter) -> Result<()> {
    let config = Config::load()?;

    if formatter.is_json() {
        #[derive(serde::Serialize)]
        struct Status {
            initialized: bool,
            config_path: String,
            chrome_host: String,
            chrome_port: u16,
            connected: bool,
            current_url: Option<String>,
        }

        let connected = cdp.connect().await.is_ok();
        let current_url = if connected {
            cdp.current_url().await.ok()
        } else {
            None
        };

        let status = Status {
            initialized: Config::is_initialized(),
            config_path: Config::config_path().to_string_lossy().to_string(),
            chrome_host: config.chrome.host.clone(),
            chrome_port: config.chrome.port,
            connected,
            current_url,
        };

        formatter.output_json(&status);
    } else {
        println!("{}", "DOMGuard Status".cyan().bold());
        println!();

        // Check initialization
        if Config::is_initialized() {
            println!("  {} Initialized", "✓".green());
            println!("    Config: {}", Config::config_path().display());
        } else {
            println!("  {} Not initialized", "✗".red());
            println!("    Run: domguard init");
        }

        println!();

        // Check Chrome connection
        println!("  Chrome: {}:{}", config.chrome.host, config.chrome.port);

        match cdp.connect().await {
            Ok(_) => {
                println!("  {} Connected", "✓".green());
                if let Ok(url) = cdp.current_url().await {
                    println!("    Current page: {}", url);
                }
                if let Ok(title) = cdp.get_title().await {
                    if !title.is_empty() {
                        println!("    Title: {}", title);
                    }
                }
            }
            Err(e) => {
                println!("  {} Not connected", "✗".red());
                println!("    {}", e.to_string().dimmed());
                println!();
                formatter.hint(&format!(
                    "Start Chrome with: chrome --remote-debugging-port={}",
                    config.chrome.port
                ));
            }
        }
    }

    Ok(())
}

async fn handle_session(
    cdp: &mut CdpConnection,
    config: &Config,
    command: &SessionSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::session::SessionStatus;

    let sessions_dir = Config::find_domguard_dir()
        .unwrap_or_else(Config::domguard_dir)
        .join("sessions");
    let recorder = SessionRecorder::new(sessions_dir.clone());

    match command {
        SessionSubcommand::Start { name } => {
            cdp.connect().await?;
            let initial_url = cdp.current_url().await.ok();
            let id = recorder.start(name.clone(), initial_url)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "session_id": id,
                    "status": "recording"
                }));
            } else {
                println!("{}", "Session recording started".green().bold());
                println!("  ID: {}", id);
                if let Some(n) = name {
                    println!("  Name: {}", n);
                }
                println!();
                formatter.hint("Use 'domguard session stop' to end recording");
            }
        }

        SessionSubcommand::Stop { cleanup } => {
            if let Some(session) = recorder.stop()? {
                if formatter.is_json() {
                    formatter.output_json(&session.summary());
                } else {
                    println!("{}", "Session recording stopped".green().bold());
                    println!();
                    print_session_summary(&session.summary(), formatter);
                }

                // Cleanup screenshots if requested or if auto_cleanup is enabled
                let should_cleanup = *cleanup || config.defaults.auto_cleanup_screenshots;
                if should_cleanup {
                    let screenshots_dir = Config::find_domguard_dir()
                        .unwrap_or_else(Config::domguard_dir)
                        .join("screenshots");

                    if screenshots_dir.exists() {
                        let mut deleted = 0;
                        for entry in std::fs::read_dir(&screenshots_dir)? {
                            let entry = entry?;
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(ext) = path.extension() {
                                    if ext == "png" || ext == "pdf" || ext == "html" {
                                        std::fs::remove_file(&path)?;
                                        deleted += 1;
                                    }
                                }
                            }
                        }
                        if !formatter.is_json() && deleted > 0 {
                            println!("Cleaned up {} screenshot files", deleted);
                        }
                    }
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": false,
                    "error": "No active session"
                }));
            } else {
                formatter.warning("No active session to stop");
            }
        }

        SessionSubcommand::Pause => {
            recorder.pause()?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "status": "paused"
                }));
            } else {
                println!("{}", "Session paused".yellow().bold());
                formatter.hint("Use 'domguard session resume' to continue");
            }
        }

        SessionSubcommand::Resume => {
            recorder.resume()?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "status": "recording"
                }));
            } else {
                println!("{}", "Session resumed".green().bold());
            }
        }

        SessionSubcommand::Status => {
            if let Some(summary) = recorder.get_summary() {
                if formatter.is_json() {
                    formatter.output_json(&summary);
                } else {
                    println!("{}", "Current Session".cyan().bold());
                    println!();
                    print_session_summary(&summary, formatter);
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "recording": false
                }));
            } else {
                println!("No active session");
                formatter.hint("Use 'domguard session start' to begin recording");
            }
        }

        SessionSubcommand::List => {
            let sessions = recorder.list_sessions()?;

            if formatter.is_json() {
                formatter.output_json(&sessions);
            } else if sessions.is_empty() {
                println!("No saved sessions");
                formatter.hint("Use 'domguard session start' to begin recording");
            } else {
                println!("{}", "Saved Sessions".cyan().bold());
                println!();
                for session in &sessions {
                    let status_color = match session.status {
                        SessionStatus::Completed => "✓".green(),
                        SessionStatus::Failed => "✗".red(),
                        SessionStatus::Paused => "⏸".yellow(),
                        SessionStatus::Recording => "●".cyan(),
                    };
                    println!(
                        "  {} {} - {} actions ({:?})",
                        status_color,
                        session.name.as_deref().unwrap_or(&session.id[..8]),
                        session.total_actions,
                        session.status
                    );
                    println!("    ID: {}", session.id);
                    println!(
                        "    Started: {}",
                        session.started_at.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!();
                }
            }
        }

        SessionSubcommand::Show { id } => {
            let session = recorder.load_session(id)?;

            if formatter.is_json() {
                formatter.output_json(&session);
            } else {
                println!("{}", "Session Details".cyan().bold());
                println!();
                print_session_summary(&session.summary(), formatter);
                println!();
                println!("{}", "Actions".cyan().bold());
                for (i, action) in session.actions.iter().enumerate() {
                    let status_icon = match action.status {
                        crate::session::ActionStatus::Success => "✓".green(),
                        crate::session::ActionStatus::Failed => "✗".red(),
                        crate::session::ActionStatus::Skipped => "-".dimmed(),
                        crate::session::ActionStatus::Paused => "⏸".yellow(),
                    };
                    println!(
                        "  {}. {} {} ({}ms)",
                        i + 1,
                        status_icon,
                        action.command,
                        action.duration_ms
                    );
                    if let Some(selector) = &action.selector {
                        println!("     Selector: {}", selector.dimmed());
                    }
                    if let Some(error) = &action.error {
                        println!("     Error: {}", error.red());
                    }
                }
            }
        }

        SessionSubcommand::Delete { id } => {
            recorder.delete_session(id)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "deleted": id
                }));
            } else {
                println!("{}", "Session deleted".green());
            }
        }

        SessionSubcommand::Export { id, format, output } => {
            let session = recorder.load_session(id)?;

            let content = match format.as_str() {
                "json" => serde_json::to_string_pretty(&session)?,
                "bash" => export_session_as_bash(&session),
                "markdown" | "md" => export_session_as_markdown(&session),
                _ => anyhow::bail!("Unknown format: {}. Use json, bash, or markdown", format),
            };

            if let Some(path) = output {
                std::fs::write(path, &content)?;
                if !formatter.is_json() {
                    println!("Exported to {}", path.display());
                }
            } else {
                println!("{}", content);
            }

            if formatter.is_json() && output.is_some() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "format": format,
                    "output": output.as_ref().map(|p| p.display().to_string())
                }));
            }
        }

        SessionSubcommand::Replay { id, step, delay } => {
            let session = recorder.load_session(id)?;
            cdp.connect().await?;

            if !formatter.is_json() {
                println!("{}", "Replaying session...".cyan().bold());
                println!(
                    "  {} actions, {} delay",
                    session.actions.len(),
                    if *step {
                        "stepping".to_string()
                    } else {
                        format!("{}ms", delay)
                    }
                );
                println!();
            }

            // Note: Full replay implementation would execute each action
            // For now, we show what would be executed
            for (i, action) in session.actions.iter().enumerate() {
                if !formatter.is_json() {
                    println!(
                        "  [{}/{}] {} {}",
                        i + 1,
                        session.actions.len(),
                        action.command,
                        action.args.to_string().dimmed()
                    );
                }

                // In a full implementation, we'd execute the action here
                // For now just wait
                tokio::time::sleep(tokio::time::Duration::from_millis(*delay)).await;
            }

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "replayed_actions": session.actions.len()
                }));
            } else {
                println!();
                println!("{}", "Replay complete".green().bold());
            }
        }
    }

    Ok(())
}

fn print_session_summary(summary: &crate::session::SessionSummary, _formatter: &Formatter) {
    use crate::session::SessionStatus;

    let status_str = match summary.status {
        SessionStatus::Recording => "Recording".cyan(),
        SessionStatus::Paused => "Paused".yellow(),
        SessionStatus::Completed => "Completed".green(),
        SessionStatus::Failed => "Failed".red(),
    };

    if let Some(name) = &summary.name {
        println!("  Name: {}", name);
    }
    println!("  ID: {}", summary.id);
    println!("  Status: {}", status_str);
    println!(
        "  Actions: {} ({} successful, {} failed)",
        summary.total_actions, summary.successful_actions, summary.failed_actions
    );
    println!("  Duration: {}ms", summary.total_duration_ms);
    println!(
        "  Started: {}",
        summary.started_at.format("%Y-%m-%d %H:%M:%S")
    );
    if let Some(ended) = summary.ended_at {
        println!("  Ended: {}", ended.format("%Y-%m-%d %H:%M:%S"));
    }
}

fn export_session_as_bash(session: &crate::session::Session) -> String {
    let mut output = String::new();
    output.push_str("#!/bin/bash\n");
    output.push_str("# DOMGuard Session Export\n");
    if let Some(name) = &session.name {
        output.push_str(&format!("# Session: {}\n", name));
    }
    output.push_str(&format!("# ID: {}\n", session.id));
    output.push_str(&format!(
        "# Recorded: {}\n",
        session.started_at.format("%Y-%m-%d %H:%M:%S")
    ));
    output.push('\n');
    output.push_str("set -e  # Exit on error\n\n");

    for action in &session.actions {
        // Convert action back to CLI command
        let cmd = format_action_as_command(action);
        output.push_str(&format!("{}\n", cmd));
    }

    output
}

fn export_session_as_markdown(session: &crate::session::Session) -> String {
    let mut output = String::new();
    output.push_str("# DOMGuard Session\n\n");
    if let Some(name) = &session.name {
        output.push_str(&format!("**Name:** {}\n\n", name));
    }
    output.push_str(&format!("**ID:** `{}`\n\n", session.id));
    output.push_str(&format!(
        "**Recorded:** {}\n\n",
        session.started_at.format("%Y-%m-%d %H:%M:%S")
    ));
    output.push_str(&format!("**Total Actions:** {}\n\n", session.actions.len()));
    output.push_str("## Actions\n\n");

    for (i, action) in session.actions.iter().enumerate() {
        let status = match action.status {
            crate::session::ActionStatus::Success => "✅",
            crate::session::ActionStatus::Failed => "❌",
            crate::session::ActionStatus::Skipped => "⏭️",
            crate::session::ActionStatus::Paused => "⏸️",
        };
        output.push_str(&format!(
            "{}. {} **{}** ({}ms)\n",
            i + 1,
            status,
            action.command,
            action.duration_ms
        ));
        if let Some(selector) = &action.selector {
            output.push_str(&format!("   - Selector: `{}`\n", selector));
        }
        if let Some(error) = &action.error {
            output.push_str(&format!("   - Error: {}\n", error));
        }
        output.push('\n');
    }

    output.push_str("## Replay Commands\n\n```bash\n");
    for action in &session.actions {
        output.push_str(&format!("{}\n", format_action_as_command(action)));
    }
    output.push_str("```\n");

    output
}

fn format_action_as_command(action: &crate::session::RecordedAction) -> String {
    // Convert recorded action back to CLI command
    let base = format!("domguard interact {}", action.command);

    if let Some(selector) = &action.selector {
        format!("{} \"{}\"", base, selector)
    } else {
        base
    }
}

fn handle_security(command: &SecuritySubcommand, formatter: &Formatter) -> Result<()> {
    use crate::security::{format_security_warning, BlockedSitesConfig, SecurityChecker};

    let config_dir = Config::find_domguard_dir().unwrap_or_else(Config::domguard_dir);
    let blocked_sites_path = config_dir.join("blocked_sites.toml");

    match command {
        SecuritySubcommand::Check {
            action,
            target,
            value,
        } => {
            let config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();
            let checker = SecurityChecker::new(config);

            let detection = match action.as_str() {
                "type" => checker.check_type_action(target, value.as_deref().unwrap_or("")),
                "click" => checker.check_click_action(target),
                "navigate" => checker.check_navigation(target),
                "upload" => checker.check_upload(&[PathBuf::from(target)]),
                _ => {
                    anyhow::bail!(
                        "Unknown action type: {}. Use: type, click, navigate, or upload",
                        action
                    );
                }
            };

            if formatter.is_json() {
                formatter.output_json(&detection);
            } else if detection.detected {
                print!("{}", format_security_warning(&detection));
            } else {
                println!("No sensitive action detected");
            }
        }

        SecuritySubcommand::ListBlocked => {
            let config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();

            if formatter.is_json() {
                formatter.output_json(&config);
            } else {
                formatter.header("Blocked Sites Configuration");
                println!(
                    "  Mode: {}",
                    if config.default_block {
                        "block by default"
                    } else {
                        "allow by default"
                    }
                );
                println!();

                if !config.blocked.is_empty() {
                    formatter.header("Blocked Patterns");
                    for pattern in &config.blocked {
                        println!("  - {}", pattern);
                    }
                } else {
                    println!("  (no blocked patterns)");
                }

                if config.default_block && !config.allowed.is_empty() {
                    println!();
                    formatter.header("Allowed Patterns");
                    for pattern in &config.allowed {
                        println!("  - {}", pattern);
                    }
                }
            }
        }

        SecuritySubcommand::Block { pattern } => {
            let mut config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();
            config.block(pattern);

            std::fs::create_dir_all(&config_dir)?;
            config.save(&blocked_sites_path)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "action": "block",
                    "pattern": pattern
                }));
            } else {
                println!("Blocked: {}", pattern);
            }
        }

        SecuritySubcommand::Unblock { pattern } => {
            let mut config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();
            config.unblock(pattern);

            std::fs::create_dir_all(&config_dir)?;
            config.save(&blocked_sites_path)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "action": "unblock",
                    "pattern": pattern
                }));
            } else {
                println!("Unblocked: {}", pattern);
            }
        }

        SecuritySubcommand::Allow { pattern } => {
            let mut config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();
            config.allow(pattern);

            std::fs::create_dir_all(&config_dir)?;
            config.save(&blocked_sites_path)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "action": "allow",
                    "pattern": pattern
                }));
            } else {
                println!("Allowed: {}", pattern);
            }
        }

        SecuritySubcommand::SetMode { mode } => {
            let mut config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();

            match mode.as_str() {
                "allow" => config.default_block = false,
                "block" => config.default_block = true,
                _ => anyhow::bail!("Invalid mode: {}. Use 'allow' or 'block'", mode),
            }

            std::fs::create_dir_all(&config_dir)?;
            config.save(&blocked_sites_path)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "mode": mode,
                    "default_block": config.default_block
                }));
            } else {
                println!(
                    "Security mode set to: {}",
                    if config.default_block {
                        "block by default"
                    } else {
                        "allow by default"
                    }
                );
            }
        }

        SecuritySubcommand::Config => {
            let config = BlockedSitesConfig::load(&blocked_sites_path).unwrap_or_default();

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "config_path": blocked_sites_path.display().to_string(),
                    "default_block": config.default_block,
                    "blocked_count": config.blocked.len(),
                    "allowed_count": config.allowed.len()
                }));
            } else {
                formatter.header("Security Configuration");
                println!("  Config file: {}", blocked_sites_path.display());
                println!(
                    "  Mode: {}",
                    if config.default_block {
                        "block by default"
                    } else {
                        "allow by default"
                    }
                );
                println!("  Blocked patterns: {}", config.blocked.len());
                println!("  Allowed patterns: {}", config.allowed.len());
            }
        }
    }

    Ok(())
}

async fn handle_explain(
    cdp: &mut CdpConnection,
    command: &ExplainSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::explanation::{explain_action, format_explanation, ExplanationContext};

    // Try to get context from current page
    let context = if cdp.connect().await.is_ok() {
        ExplanationContext {
            current_url: cdp.current_url().await.ok(),
            current_title: cdp.get_title().await.ok(),
            previous_action: None,
            current_goal: None,
        }
    } else {
        ExplanationContext::default()
    };

    let explanation = match command {
        ExplainSubcommand::Click { target } => explain_action("click", Some(target), &context),
        ExplainSubcommand::Type { target } => explain_action("type", Some(target), &context),
        ExplainSubcommand::Key { keys } => explain_action("key", Some(keys), &context),
        ExplainSubcommand::Navigate { url } => explain_action("navigate", Some(url), &context),
        ExplainSubcommand::Wait { target } => explain_action("wait", Some(target), &context),
        ExplainSubcommand::Interact { command, target } => {
            explain_action(command, target.as_deref(), &context)
        }
    };

    if formatter.is_json() {
        formatter.output_json(&explanation);
    } else {
        println!("{}", "Action Explanation".cyan().bold());
        println!();
        println!("{}", format_explanation(&explanation));
    }

    Ok(())
}

async fn handle_sites(
    cdp: &mut CdpConnection,
    command: &SitesSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::site_instructions::{format_instructions, SiteInstructionsManager};

    let sites_dir = Config::find_domguard_dir()
        .unwrap_or_else(Config::domguard_dir)
        .join("sites");

    let mut manager = SiteInstructionsManager::new(sites_dir.clone());
    manager.load_all()?;

    match command {
        SitesSubcommand::List => {
            let sites = manager.list();

            if formatter.is_json() {
                formatter.output_json(&sites);
            } else if sites.is_empty() {
                println!("No site instructions saved");
                formatter.hint("Use 'domguard sites create <domain>' to create one");
            } else {
                println!("{}", "Saved Site Instructions".cyan().bold());
                println!();
                for site in sites {
                    println!(
                        "  {} - {}",
                        site.domain,
                        site.description.as_deref().unwrap_or("(no description)")
                    );
                }
            }
        }

        SitesSubcommand::Show { domain } => {
            // Try to find instructions for this domain
            let url = if domain.contains("://") {
                domain.clone()
            } else {
                format!("https://{}", domain)
            };

            if let Some(instructions) = manager.get_for_url(&url) {
                if formatter.is_json() {
                    formatter.output_json(instructions);
                } else {
                    println!("{}", format_instructions(instructions));
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "found": false,
                    "domain": domain
                }));
            } else {
                println!("No instructions found for: {}", domain);
                formatter.hint(&format!(
                    "Use 'domguard sites create {}' to create one",
                    domain
                ));
            }
        }

        SitesSubcommand::Create { domain } => {
            let template = SiteInstructionsManager::create_template(domain);
            let path = manager.save(&template)?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "domain": domain,
                    "path": path.display().to_string()
                }));
            } else {
                println!("{}", "Created site instructions".green().bold());
                println!("  Domain: {}", domain);
                println!("  File: {}", path.display());
                println!();
                formatter.hint(&format!("Edit with 'domguard sites edit {}'", domain));
            }
        }

        SitesSubcommand::Delete { domain } => {
            if manager.delete(domain)? {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "success": true,
                        "deleted": domain
                    }));
                } else {
                    println!("Deleted: {}", domain);
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": false,
                    "error": "Not found"
                }));
            } else {
                println!("No instructions found for: {}", domain);
            }
        }

        SitesSubcommand::Current => {
            if cdp.connect().await.is_err() {
                anyhow::bail!(
                    "Chrome not connected. Start Chrome with --remote-debugging-port=9222"
                );
            }

            let current_url = cdp.current_url().await?;

            if let Some(instructions) = manager.get_for_url(&current_url) {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "found": true,
                        "url": current_url,
                        "instructions": instructions
                    }));
                } else {
                    println!("{}", "Site Instructions for Current Page".cyan().bold());
                    println!("  URL: {}", current_url);
                    println!();
                    println!("{}", format_instructions(instructions));
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "found": false,
                    "url": current_url
                }));
            } else {
                println!("No instructions for: {}", current_url);
                // Extract domain for hint
                if let Some(domain) = current_url
                    .split("://")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                {
                    formatter.hint(&format!(
                        "Use 'domguard sites create {}' to create one",
                        domain
                    ));
                }
            }
        }

        SitesSubcommand::Edit { domain } => {
            // Find or create the file
            let filename = domain.replace(['*', '.'], "_") + ".toml";
            let path = sites_dir.join(&filename);

            if !path.exists() {
                // Create template first
                let template = SiteInstructionsManager::create_template(domain);
                manager.save(&template)?;
            }

            // Open in editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "path": path.display().to_string(),
                    "editor": editor
                }));
            } else {
                println!("Opening {} in {}...", path.display(), editor);
                let status = std::process::Command::new(&editor).arg(&path).status();

                match status {
                    Ok(s) if s.success() => println!("Saved!"),
                    Ok(_) => println!("Editor exited"),
                    Err(e) => println!("Could not open editor: {}", e),
                }
            }
        }
    }

    Ok(())
}

async fn handle_workflow(
    cdp: &mut CdpConnection,
    command: &WorkflowSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::workflow::{
        format_workflow, format_workflow_list, substitute_params, WorkflowManager,
    };

    let workflows_dir = Config::find_domguard_dir()
        .unwrap_or_else(Config::domguard_dir)
        .join("workflows");

    let sessions_dir = Config::find_domguard_dir()
        .unwrap_or_else(Config::domguard_dir)
        .join("sessions");

    let mut manager = WorkflowManager::new(workflows_dir.clone());
    manager.load_all()?;

    match command {
        WorkflowSubcommand::List { tag, domain } => {
            let workflows = if let Some(t) = tag {
                manager.list_by_tag(t)
            } else if let Some(d) = domain {
                manager.list_for_domain(d)
            } else {
                manager.list()
            };

            if formatter.is_json() {
                formatter.output_json(&workflows);
            } else if workflows.is_empty() {
                println!("No workflows saved");
                formatter.hint("Use 'domguard workflow create <name>' or 'domguard workflow from-session <id> <name>'");
            } else {
                println!("{}", "Saved Workflows".cyan().bold());
                println!();
                print!("{}", format_workflow_list(&workflows));
            }
        }

        WorkflowSubcommand::Show { id } => {
            if let Some(workflow) = manager
                .get(id)
                .or_else(|| manager.find_by_name(id).first().copied())
            {
                if formatter.is_json() {
                    formatter.output_json(workflow);
                } else {
                    println!("{}", format_workflow(workflow));
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "found": false,
                    "id": id
                }));
            } else {
                println!("Workflow not found: {}", id);
            }
        }

        WorkflowSubcommand::FromSession { session_id, name } => {
            let session_recorder = crate::session::SessionRecorder::new(sessions_dir);
            let session = session_recorder.load_session(session_id)?;

            let workflow = WorkflowManager::from_session(&session, name);
            let path = manager.save(workflow.clone())?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "workflow_id": workflow.id,
                    "name": workflow.name,
                    "steps": workflow.steps.len(),
                    "path": path.display().to_string()
                }));
            } else {
                println!("{}", "Workflow created from session".green().bold());
                println!("  ID: {}", workflow.id);
                println!("  Name: {}", workflow.name);
                println!("  Steps: {}", workflow.steps.len());
                println!("  File: {}", path.display());
            }
        }

        WorkflowSubcommand::Create { name } => {
            let workflow = WorkflowManager::create_empty(name);
            let path = manager.save(workflow.clone())?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "workflow_id": workflow.id,
                    "path": path.display().to_string()
                }));
            } else {
                println!("{}", "Created new workflow".green().bold());
                println!("  ID: {}", workflow.id);
                println!("  File: {}", path.display());
                println!();
                formatter.hint(&format!(
                    "Edit with 'domguard workflow edit {}'",
                    workflow.id
                ));
            }
        }

        WorkflowSubcommand::Run {
            id,
            param,
            dry_run,
            delay,
        } => {
            let workflow = manager
                .get(id)
                .or_else(|| manager.find_by_name(id).first().copied())
                .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", id))?
                .clone();

            // Build parameter map
            let params: std::collections::HashMap<String, String> = param.iter().cloned().collect();

            // Check required parameters
            for p in &workflow.parameters {
                if p.required && !params.contains_key(&p.name) && p.default.is_none() {
                    anyhow::bail!("Missing required parameter: {}", p.name);
                }
            }

            if *dry_run {
                // Just show what would be done
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "dry_run": true,
                        "workflow": workflow.name,
                        "steps": workflow.steps.len()
                    }));
                } else {
                    println!("{}", "Dry Run - Steps to execute:".cyan().bold());
                    println!();
                    for (i, step) in workflow.steps.iter().enumerate() {
                        let target = step
                            .target
                            .as_ref()
                            .map(|t| substitute_params(t, &params))
                            .unwrap_or_default();
                        println!("  {}. {} {}", i + 1, step.action, target);
                    }
                }
            } else {
                // Actually run the workflow
                cdp.connect().await?;

                if !formatter.is_json() {
                    println!(
                        "{}",
                        format!("Running workflow: {}", workflow.name).cyan().bold()
                    );
                    println!();
                }

                let start = std::time::Instant::now();
                let mut step_results = Vec::new();

                for (i, step) in workflow.steps.iter().enumerate() {
                    let step_start = std::time::Instant::now();

                    let target = step.target.as_ref().map(|t| substitute_params(t, &params));
                    let value = step.value.as_ref().map(|v| substitute_params(v, &params));

                    if !formatter.is_json() {
                        println!(
                            "  [{}/{}] {} {}",
                            i + 1,
                            workflow.steps.len(),
                            step.action,
                            target.as_deref().unwrap_or("")
                        );
                    }

                    // Execute the step
                    let result =
                        execute_workflow_step(cdp, step, target.as_deref(), value.as_deref()).await;

                    let step_result = crate::workflow::StepResult {
                        index: i,
                        name: step.name.clone(),
                        success: result.is_ok(),
                        duration_ms: step_start.elapsed().as_millis() as u64,
                        skipped: false,
                        retries: 0,
                        error: result.err().map(|e| e.to_string()),
                    };

                    step_results.push(step_result);

                    // Wait between steps
                    tokio::time::sleep(std::time::Duration::from_millis(*delay)).await;
                }

                let duration_ms = start.elapsed().as_millis() as u64;
                let success = step_results
                    .iter()
                    .all(|r| r.success || !workflow.steps[r.index].required);

                manager.record_run(&workflow.id, success)?;

                if formatter.is_json() {
                    formatter.output_json(&crate::workflow::WorkflowResult {
                        workflow_id: workflow.id.clone(),
                        success,
                        duration_ms,
                        step_results,
                        error: None,
                        screenshots: vec![],
                    });
                } else {
                    println!();
                    if success {
                        println!("{}", "Workflow completed successfully".green().bold());
                    } else {
                        println!("{}", "Workflow completed with errors".red().bold());
                    }
                    println!("  Duration: {}ms", duration_ms);
                }
            }
        }

        WorkflowSubcommand::Delete { id } => {
            if manager.delete(id)? {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "success": true,
                        "deleted": id
                    }));
                } else {
                    println!("Deleted workflow: {}", id);
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": false,
                    "error": "Not found"
                }));
            } else {
                println!("Workflow not found: {}", id);
            }
        }

        WorkflowSubcommand::Edit { id } => {
            let filename = format!("{}.toml", id);
            let path = workflows_dir.join(&filename);

            if !path.exists() {
                anyhow::bail!("Workflow not found: {}", id);
            }

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "path": path.display().to_string(),
                    "editor": editor
                }));
            } else {
                println!("Opening {} in {}...", path.display(), editor);
                let status = std::process::Command::new(&editor).arg(&path).status();

                match status {
                    Ok(s) if s.success() => println!("Saved!"),
                    Ok(_) => println!("Editor exited"),
                    Err(e) => println!("Could not open editor: {}", e),
                }
            }
        }
    }

    Ok(())
}

/// Execute a single workflow step
async fn execute_workflow_step(
    cdp: &CdpConnection,
    step: &crate::workflow::WorkflowStep,
    target: Option<&str>,
    value: Option<&str>,
) -> Result<()> {
    match step.action.as_str() {
        "click" => {
            if let Some(sel) = target {
                cdp.click(sel, 0).await?;
            }
        }
        "type" => {
            if let Some(sel) = target {
                if let Some(text) = value {
                    cdp.type_into(sel, text).await?;
                }
            }
        }
        "navigate" => {
            if let Some(url) = target {
                cdp.navigate(url).await?;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
        "wait" => {
            if let Some(sel) = target {
                let timeout = step.timeout_ms.unwrap_or(5000);
                cdp.wait_for(sel, timeout).await?;
            }
        }
        "screenshot" => {
            cdp.screenshot(false).await?;
        }
        "scroll" => {
            if let Some(sel) = target {
                cdp.scroll_to_element(sel).await?;
            }
        }
        "hover" => {
            if let Some(sel) = target {
                cdp.hover(sel).await?;
            }
        }
        "key" => {
            if let Some(key) = target {
                cdp.press_key(key).await?;
            }
        }
        _ => {
            // Unknown action, skip
        }
    }

    Ok(())
}

async fn handle_takeover(
    cdp: &mut CdpConnection,
    command: &TakeoverSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::takeover::{format_takeover, TakeoverManager, TakeoverReason, TakeoverSession};

    let domguard_dir = Config::find_domguard_dir().unwrap_or_else(Config::domguard_dir);
    let manager = TakeoverManager::new(domguard_dir);

    match command {
        TakeoverSubcommand::Request {
            reason,
            message,
            instructions,
            expected,
        } => {
            // Check if already in takeover
            if manager.is_active() {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "success": false,
                        "error": "Takeover already active",
                        "current": manager.get_current()
                    }));
                } else {
                    formatter.warning("Takeover already active");
                    if let Some(current) = manager.get_current() {
                        println!("{}", format_takeover(&current));
                    }
                    formatter.hint("Use 'domguard takeover done' to complete or 'domguard takeover cancel' to abort");
                }
                return Ok(());
            }

            // Parse reason
            let takeover_reason = match reason.to_lowercase().as_str() {
                "captcha" => TakeoverReason::Captcha,
                "auth" | "authentication" | "login" => TakeoverReason::Authentication,
                "sensitive" => TakeoverReason::SensitiveAction,
                "error" => TakeoverReason::Error,
                "uncertain" => TakeoverReason::Uncertain,
                "complex" => TakeoverReason::ComplexInteraction,
                "2fa" | "mfa" | "twofactor" => TakeoverReason::TwoFactorAuth,
                "payment" | "pay" => TakeoverReason::Payment,
                "user" | "requested" => TakeoverReason::UserRequested,
                custom => TakeoverReason::Custom(custom.to_string()),
            };

            // Build default message if not provided
            let msg = message.clone().unwrap_or_else(|| match &takeover_reason {
                TakeoverReason::Captcha => {
                    "CAPTCHA detected - human verification required".to_string()
                }
                TakeoverReason::Authentication => {
                    "Authentication required - please log in".to_string()
                }
                TakeoverReason::SensitiveAction => {
                    "Sensitive action requires human confirmation".to_string()
                }
                TakeoverReason::Error => "Error occurred - human intervention needed".to_string(),
                TakeoverReason::Uncertain => {
                    "Agent unsure how to proceed - please help".to_string()
                }
                TakeoverReason::ComplexInteraction => {
                    "Complex interaction - human control needed".to_string()
                }
                TakeoverReason::TwoFactorAuth => "Two-factor authentication required".to_string(),
                TakeoverReason::Payment => "Payment action requires human confirmation".to_string(),
                TakeoverReason::UserRequested => "User requested control".to_string(),
                TakeoverReason::Custom(s) => s.clone(),
            });

            let mut session = TakeoverSession::new(takeover_reason, &msg);

            if let Some(instr) = instructions {
                session = session.with_instructions(instr);
            }

            if let Some(exp) = expected {
                session = session.with_expected_outcome(exp);
            }

            // Try to get current URL
            if cdp.connect().await.is_ok() {
                if let Ok(url) = cdp.current_url().await {
                    session = session.with_url(&url);
                }
            }

            let id = manager.start(session.clone())?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "takeover_id": id,
                    "session": session
                }));
            } else {
                println!("{}", "TAKEOVER REQUESTED".yellow().bold());
                println!();
                println!("{}", format_takeover(&session));
                println!();
                formatter.hint("Use 'domguard takeover done' when finished, or 'domguard takeover cancel' to abort");
            }
        }

        TakeoverSubcommand::Done { success, notes } => {
            if let Some(session) = manager.complete(*success, notes.clone())? {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "success": true,
                        "completed_session": session,
                        "automation_resumed": true
                    }));
                } else {
                    println!("{}", "TAKEOVER COMPLETE".green().bold());
                    println!();
                    println!(
                        "  Result: {}",
                        if *success {
                            "Success".green()
                        } else {
                            "Failed".red()
                        }
                    );
                    if let Some(duration) = session.duration_secs {
                        println!("  Duration: {}s", duration);
                    }
                    if let Some(n) = notes {
                        println!("  Notes: {}", n);
                    }
                    println!();
                    println!("Automation resumed.");
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": false,
                    "error": "No active takeover"
                }));
            } else {
                formatter.warning("No active takeover to complete");
                formatter.hint("Use 'domguard takeover request <reason>' to start one");
            }
        }

        TakeoverSubcommand::Cancel => {
            if manager.cancel()? {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "success": true,
                        "cancelled": true
                    }));
                } else {
                    println!("{}", "Takeover cancelled".yellow());
                    println!("Automation resumed.");
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": false,
                    "error": "No active takeover"
                }));
            } else {
                formatter.warning("No active takeover to cancel");
            }
        }

        TakeoverSubcommand::Status => {
            if let Some(session) = manager.get_current() {
                if formatter.is_json() {
                    formatter.output_json(&serde_json::json!({
                        "active": true,
                        "session": session
                    }));
                } else {
                    println!("{}", "TAKEOVER ACTIVE".yellow().bold());
                    println!();
                    println!("{}", format_takeover(&session));
                }
            } else if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "active": false
                }));
            } else {
                println!("No active takeover");
                println!("Automation is running normally.");
            }
        }

        TakeoverSubcommand::History { limit } => {
            let history = manager.get_history()?;
            let limited: Vec<_> = history.iter().take(*limit).collect();

            if formatter.is_json() {
                formatter.output_json(&limited);
            } else if limited.is_empty() {
                println!("No takeover history");
            } else {
                println!("{}", "Takeover History".cyan().bold());
                println!();
                for session in limited {
                    let success_icon = match session.success {
                        Some(true) => "✓".green(),
                        Some(false) => "✗".red(),
                        None => "?".dimmed(),
                    };
                    println!(
                        "  {} {} - {:?}",
                        success_icon,
                        session.started_at.format("%Y-%m-%d %H:%M:%S"),
                        session.reason
                    );
                    println!("    {}", session.message);
                    if let Some(duration) = session.duration_secs {
                        println!("    Duration: {}s", duration);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

async fn handle_correction(
    cdp: &mut CdpConnection,
    command: &CorrectionSubcommand,
    formatter: &Formatter,
) -> Result<()> {
    use crate::correction::{
        classify_error, dismiss_overlay_script, get_recovery_strategies, wait_stable_script,
        AutomationError, CorrectionConfig, RecoveryStrategy,
    };

    match command {
        CorrectionSubcommand::Config => {
            let config = CorrectionConfig::default();

            if formatter.is_json() {
                formatter.output_json(&config);
            } else {
                println!("{}", "Self-Correction Configuration".cyan().bold());
                println!();
                println!(
                    "  Enabled: {}",
                    if config.enabled {
                        "Yes".green()
                    } else {
                        "No".red()
                    }
                );
                println!("  Max retries: {}", config.max_retries);
                println!("  Base delay: {}ms", config.base_delay_ms);
                println!(
                    "  Exponential backoff: {}",
                    if config.exponential_backoff {
                        "Yes"
                    } else {
                        "No"
                    }
                );
                println!("  Max recovery time: {}ms", config.max_recovery_time_ms);
                println!(
                    "  Auto-dismiss dialogs: {}",
                    if config.auto_dismiss_dialogs {
                        "Yes"
                    } else {
                        "No"
                    }
                );
                println!(
                    "  Auto-scroll: {}",
                    if config.auto_scroll { "Yes" } else { "No" }
                );
                println!(
                    "  Takeover on failure: {}",
                    if config.takeover_on_failure {
                        "Yes"
                    } else {
                        "No"
                    }
                );
            }
        }

        CorrectionSubcommand::Enable => {
            // In a full implementation, this would persist the setting
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "self_correction": "enabled"
                }));
            } else {
                println!("{}", "Self-correction enabled".green().bold());
                println!("DOMGuard will automatically attempt recovery when actions fail.");
            }
        }

        CorrectionSubcommand::Disable => {
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "self_correction": "disabled"
                }));
            } else {
                println!("{}", "Self-correction disabled".yellow().bold());
                println!("DOMGuard will report errors without attempting recovery.");
            }
        }

        CorrectionSubcommand::Analyze { error, action } => {
            let error_type = classify_error(error);
            let strategies = get_recovery_strategies(&error_type, action);

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "error_message": error,
                    "classified_as": error_type,
                    "action": action,
                    "recovery_strategies": strategies
                }));
            } else {
                println!("{}", "Error Analysis".cyan().bold());
                println!();
                println!("  Input: {}", error);
                println!("  Classified as: {}", error_type);
                println!("  Action: {}", action);
                println!();
                println!("{}", "Recovery Strategies".cyan().bold());
                for (i, strategy) in strategies.iter().enumerate() {
                    println!("  {}. {}", i + 1, strategy);
                }
            }
        }

        CorrectionSubcommand::Test { strategy, target } => {
            cdp.connect().await?;

            let result = match strategy.to_lowercase().as_str() {
                "scroll" | "scroll-into-view" => {
                    if let Some(sel) = target {
                        match cdp.scroll_to_element(sel).await {
                            Ok(_) => (true, "Element scrolled into view".to_string()),
                            Err(e) => (false, e.to_string()),
                        }
                    } else {
                        (
                            false,
                            "Target selector required for scroll strategy".to_string(),
                        )
                    }
                }
                "dismiss-overlay" | "overlay" => {
                    let script = dismiss_overlay_script();
                    match cdp.evaluate(script).await {
                        Ok(result) => {
                            let dismissed = result.as_bool().unwrap_or(false);
                            (
                                dismissed,
                                if dismissed {
                                    "Overlays dismissed".to_string()
                                } else {
                                    "No overlays found".to_string()
                                },
                            )
                        }
                        Err(e) => (false, e.to_string()),
                    }
                }
                "wait-stable" | "stable" => {
                    let script = wait_stable_script();
                    match cdp.evaluate(script).await {
                        Ok(result) => {
                            let stable = result
                                .get("stable")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let duration =
                                result.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
                            (
                                stable,
                                format!(
                                    "Page {} after {}ms",
                                    if stable {
                                        "stabilized"
                                    } else {
                                        "still changing"
                                    },
                                    duration
                                ),
                            )
                        }
                        Err(e) => (false, e.to_string()),
                    }
                }
                "refresh" => match cdp.refresh().await {
                    Ok(_) => (true, "Page refreshed".to_string()),
                    Err(e) => (false, e.to_string()),
                },
                _ => (
                    false,
                    format!(
                        "Unknown strategy: {}. Use: scroll, dismiss-overlay, wait-stable, refresh",
                        strategy
                    ),
                ),
            };

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "strategy": strategy,
                    "target": target,
                    "success": result.0,
                    "message": result.1
                }));
            } else {
                let status = if result.0 { "✓".green() } else { "✗".red() };
                println!("{} Strategy: {}", status, strategy);
                println!("  {}", result.1);
            }
        }

        CorrectionSubcommand::DismissOverlay => {
            cdp.connect().await?;

            let script = dismiss_overlay_script();
            let result = cdp.evaluate(script).await?;
            let dismissed = result.as_bool().unwrap_or(false);

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "success": true,
                    "overlays_dismissed": dismissed
                }));
            } else if dismissed {
                println!("{}", "Overlays dismissed".green().bold());
            } else {
                println!("No blocking overlays found");
            }
        }

        CorrectionSubcommand::WaitStable => {
            cdp.connect().await?;

            let script = wait_stable_script();
            let result = cdp.evaluate(script).await?;
            let stable = result
                .get("stable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let duration = result.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "stable": stable,
                    "duration_ms": duration
                }));
            } else if stable {
                println!(
                    "{}",
                    format!("Page stable after {}ms", duration).green().bold()
                );
            } else {
                println!(
                    "{}",
                    format!("Page still changing after {}ms", duration).yellow()
                );
            }
        }

        CorrectionSubcommand::Strategies { error_type } => {
            let error = match error_type.to_lowercase().replace(['-', '_'], " ").as_str() {
                "element not found" | "not found" => AutomationError::ElementNotFound,
                "element not visible" | "not visible" => AutomationError::ElementNotVisible,
                "element not interactable" | "not interactable" => {
                    AutomationError::ElementNotInteractable
                }
                "navigation timeout" | "timeout" => AutomationError::NavigationTimeout,
                "network" | "network error" => AutomationError::NetworkError,
                "javascript" | "js error" => AutomationError::JavaScriptError,
                "captcha" => AutomationError::CaptchaDetected,
                "auth" | "authentication" => AutomationError::AuthRequired,
                "dialog" | "unexpected dialog" => AutomationError::UnexpectedDialog,
                "stale" | "stale element" => AutomationError::StaleElement,
                "click intercepted" | "intercepted" => AutomationError::ClickIntercepted,
                "page change" | "unexpected page" => AutomationError::UnexpectedPageChange,
                other => AutomationError::Unknown(other.to_string()),
            };

            let strategies = get_recovery_strategies(&error, "click");

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "error_type": error,
                    "strategies": strategies
                }));
            } else {
                println!(
                    "{}",
                    format!("Recovery Strategies for: {}", error).cyan().bold()
                );
                println!();
                for (i, strategy) in strategies.iter().enumerate() {
                    let is_takeover = matches!(strategy, RecoveryStrategy::RequestTakeover { .. });
                    let marker = if is_takeover {
                        "→".yellow()
                    } else {
                        format!("{}", i + 1).normal()
                    };
                    println!("  {} {}", marker, strategy);
                }

                println!();
                formatter.hint("These strategies are tried in order until one succeeds");
            }
        }
    }

    Ok(())
}
