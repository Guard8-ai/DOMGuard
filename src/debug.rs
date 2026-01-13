//! Debug mode - Inspect page state
//!
//! DOM tree, console messages, network requests, storage, cookies

use anyhow::Result;
use serde::Serialize;
use std::fmt::Write as _;

use crate::cdp::CdpConnection;
use crate::output::{mask_sensitive, AriaNode, ConsoleMessage, DomNode, Formatter, NetworkRequest};

/// Get current timestamp in seconds, with fallback to 0 if system clock is before UNIX epoch
fn safe_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Tab action types
#[derive(Debug, Clone)]
pub enum TabCommand {
    List,
    New { url: Option<String> },
    Switch { id: String },
    Close { id: String },
}

/// Throttle mode for CPU/network emulation
#[derive(Debug, Clone)]
pub enum ThrottleMode {
    /// Disable all throttling
    Off,
    /// CPU throttling (rate: 1 = no throttle, 4 = 4x slowdown)
    Cpu {
        rate: f64,
    },
    /// Network throttling presets
    Network3g,
    NetworkSlow3g,
    NetworkOffline,
    /// Custom network throttling
    NetworkCustom {
        download_kbps: f64,
        upload_kbps: f64,
        latency_ms: f64,
    },
}

/// Debug subcommand types
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Dom {
        selector: Option<String>,
    },
    Styles {
        selector: String,
    },
    Console {
        follow: bool,
        filter: Option<String>,
    },
    Network {
        filter: Option<String>,
    },
    Eval {
        expression: String,
    },
    Storage,
    Cookies,
    Aria {
        selector: Option<String>,
    },
    Tabs {
        action: TabCommand,
    },
    // Chrome DevTools MCP features
    Performance,
    Snapshot {
        output: Option<std::path::PathBuf>,
    },
    Throttle {
        mode: ThrottleMode,
    },
    NetworkDetails {
        filter: Option<String>,
    },
    // Element highlighting
    Highlight {
        selector: String,
        color: String,
        duration: u64,
        all: bool,
    },
    ClearHighlights,
    // CAPTCHA detection
    Captcha,
}

/// Run debug command
pub async fn run_debug(
    cdp: &CdpConnection,
    command: DebugCommand,
    formatter: &Formatter,
) -> Result<()> {
    match command {
        DebugCommand::Dom { selector } => debug_dom(cdp, selector.as_deref(), formatter).await,
        DebugCommand::Styles { selector } => debug_styles(cdp, &selector, formatter).await,
        DebugCommand::Console { follow, filter } => {
            debug_console(cdp, follow, filter.as_deref(), formatter).await
        }
        DebugCommand::Network { filter } => debug_network(cdp, filter.as_deref(), formatter).await,
        DebugCommand::Eval { expression } => debug_eval(cdp, &expression, formatter).await,
        DebugCommand::Storage => debug_storage(cdp, formatter).await,
        DebugCommand::Cookies => debug_cookies(cdp, formatter).await,
        DebugCommand::Aria { selector } => debug_aria(cdp, selector.as_deref(), formatter).await,
        DebugCommand::Tabs { action } => debug_tabs(cdp, action, formatter).await,
        // Chrome DevTools MCP features
        DebugCommand::Performance => debug_performance(cdp, formatter).await,
        DebugCommand::Snapshot { output } => debug_snapshot(cdp, output, formatter).await,
        DebugCommand::Throttle { mode } => debug_throttle(cdp, mode, formatter).await,
        DebugCommand::NetworkDetails { filter } => {
            debug_network_details(cdp, filter.as_deref(), formatter).await
        }
        // Element highlighting
        DebugCommand::Highlight {
            selector,
            color,
            duration,
            all,
        } => debug_highlight(cdp, &selector, &color, duration, all, formatter).await,
        DebugCommand::ClearHighlights => debug_clear_highlights(cdp, formatter).await,
        // CAPTCHA detection
        DebugCommand::Captcha => debug_captcha(cdp, formatter).await,
    }
}

/// Debug DOM tree
async fn debug_dom(
    cdp: &CdpConnection,
    selector: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    let dom_tree = if let Some(sel) = selector {
        // Get specific element(s)
        let js = format!(
            r#"
            (function() {{
                const elements = document.querySelectorAll('{}');
                if (elements.length === 0) return null;

                function nodeToJson(el, depth) {{
                    if (depth > 5) return null;
                    const result = {{
                        tag: el.tagName.toLowerCase(),
                        id: el.id || null,
                        classes: el.className ? el.className.split(' ').filter(c => c) : null,
                        text: el.childNodes.length === 1 && el.childNodes[0].nodeType === 3
                            ? el.textContent.trim() : null,
                        children: []
                    }};
                    for (const child of el.children) {{
                        const childJson = nodeToJson(child, depth + 1);
                        if (childJson) result.children.push(childJson);
                    }}
                    return result;
                }}

                return Array.from(elements).map(el => nodeToJson(el, 0));
            }})()
        "#,
            sel
        );

        let result = cdp.evaluate(&js).await?;
        if result.is_null() {
            return Err(anyhow::anyhow!("No element matches selector \"{}\"", sel));
        }
        result
    } else {
        // Get full DOM tree (limited depth)
        let js = r#"
            (function() {
                function nodeToJson(el, depth) {
                    if (depth > 4) return null;
                    if (!el.tagName) return null;

                    const result = {
                        tag: el.tagName.toLowerCase(),
                        id: el.id || null,
                        classes: el.className && typeof el.className === 'string'
                            ? el.className.split(' ').filter(c => c) : null,
                        text: el.childNodes.length === 1 && el.childNodes[0].nodeType === 3
                            ? el.textContent.trim().substring(0, 100) : null,
                        children: []
                    };

                    for (const child of el.children) {
                        if (result.children.length >= 20) break;
                        const childJson = nodeToJson(child, depth + 1);
                        if (childJson) result.children.push(childJson);
                    }
                    return result;
                }

                return nodeToJson(document.documentElement, 0);
            })()
        "#;

        cdp.evaluate(js).await?
    };

    if formatter.is_json() {
        formatter.output_json(&dom_tree);
    } else {
        // Convert JSON to DomNode for human-readable display
        fn json_to_dom_node(value: &serde_json::Value) -> Option<DomNode> {
            let obj = value.as_object()?;
            Some(DomNode {
                tag: obj.get("tag")?.as_str()?.to_string(),
                id: obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                classes: obj.get("classes").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                        .collect()
                }),
                attributes: None,
                text: obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                children: obj
                    .get("children")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(json_to_dom_node).collect())
                    .unwrap_or_default(),
            })
        }

        if let Some(nodes) = dom_tree.as_array() {
            // Multiple elements from selector
            for node in nodes {
                if let Some(dom) = json_to_dom_node(node) {
                    println!("{}", dom);
                }
            }
        } else if let Some(dom) = json_to_dom_node(&dom_tree) {
            // Single element (full tree)
            println!("{}", dom);
        }
    }

    Ok(())
}

/// Debug computed styles
async fn debug_styles(cdp: &CdpConnection, selector: &str, formatter: &Formatter) -> Result<()> {
    let js = format!(
        r#"
        (function() {{
            const el = document.querySelector('{}');
            if (!el) return null;

            const style = getComputedStyle(el);
            const important = [
                'display', 'position', 'width', 'height',
                'padding', 'margin', 'border',
                'color', 'backgroundColor', 'fontSize', 'fontFamily', 'fontWeight',
                'flexDirection', 'justifyContent', 'alignItems', 'gap',
                'gridTemplateColumns', 'gridTemplateRows',
                'overflow', 'opacity', 'zIndex', 'transform'
            ];

            const result = {{}};
            for (const prop of important) {{
                const val = style[prop];
                if (val && val !== 'none' && val !== 'normal' && val !== 'auto' && val !== '0px') {{
                    result[prop] = val;
                }}
            }}
            return result;
        }})()
    "#,
        selector
    );

    let result = cdp.evaluate(&js).await?;

    if result.is_null() {
        return Err(anyhow::anyhow!(
            "No element matches selector \"{}\"",
            selector
        ));
    }

    if formatter.is_json() {
        formatter.output_json(&result);
    } else {
        formatter.header(&format!("Styles for \"{}\"", selector));
        if let Some(obj) = result.as_object() {
            for (key, value) in obj {
                formatter.kv(key, value.as_str().unwrap_or(""));
            }
        }
    }

    Ok(())
}

/// Debug console messages using CDP event listeners
async fn debug_console(
    cdp: &CdpConnection,
    follow: bool,
    filter: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    // Use CDP to capture console messages, exceptions, and log entries
    // Timeout determines how long to listen for events
    let timeout_ms = if follow { 5000 } else { 500 };

    let entries = cdp.capture_console_messages(timeout_ms).await?;

    // Apply filter if specified
    let filtered: Vec<_> = if let Some(f) = filter {
        entries
            .iter()
            .filter(|e| e.text.contains(f) || e.source.contains(f))
            .collect()
    } else {
        entries.iter().collect()
    };

    // Convert ConsoleEntry to ConsoleMessage for output
    let messages: Vec<ConsoleMessage> = filtered
        .iter()
        .map(|e| ConsoleMessage {
            level: e.level.clone(),
            text: e.text.clone(),
            url: e.url.clone(),
            line: e.line,
        })
        .collect();

    if formatter.is_json() {
        formatter.output_json(&messages);
    } else {
        formatter.header("Console Messages (CDP)");

        if messages.is_empty() {
            if follow {
                println!("  No new messages captured in {}ms", timeout_ms);
            } else {
                println!("  No messages. Use --follow to wait for new events.");
            }
            formatter.hint("Refresh page with 'domguard interact refresh' then run console again.");
        } else {
            for msg in &messages {
                // Use ConsoleMessage's Display implementation for colored output
                println!("  {}", msg);
            }

            println!("\n  Total: {} message(s)", messages.len());
        }

        if let Some(f) = filter {
            println!("  Filter: {}", f);
        }
    }

    Ok(())
}

/// Debug network requests
async fn debug_network(
    cdp: &CdpConnection,
    filter: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    // Get performance entries for network info
    let js = r#"
        (function() {
            const entries = performance.getEntriesByType('resource');
            return entries.map(e => ({
                method: 'GET',
                url: e.name,
                type: e.initiatorType,
                duration_ms: Math.round(e.duration),
                size_bytes: e.transferSize || null
            }));
        })()
    "#;

    let result = cdp.evaluate(js).await?;

    // Parse into NetworkRequest structs
    let requests: Vec<NetworkRequest> = if let Some(arr) = result.as_array() {
        arr.iter()
            .filter_map(|r| {
                Some(NetworkRequest {
                    method: r
                        .get("method")
                        .and_then(|m| m.as_str())
                        .unwrap_or("GET")
                        .to_string(),
                    url: r.get("url").and_then(|u| u.as_str())?.to_string(),
                    status: None, // Performance API doesn't provide status
                    mime_type: r
                        .get("type")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                    size_bytes: r.get("size_bytes").and_then(|s| s.as_u64()),
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    // Apply filter
    let filtered: Vec<_> = if let Some(f) = filter {
        requests.into_iter().filter(|r| r.url.contains(f)).collect()
    } else {
        requests
    };

    if formatter.is_json() {
        formatter.output_json(&filtered);
    } else {
        formatter.header("Network Requests");
        for req in &filtered {
            println!("  {}", req);
        }
        println!("\n  Total: {} requests", filtered.len());
    }

    Ok(())
}

/// Debug eval - execute JavaScript
async fn debug_eval(cdp: &CdpConnection, expression: &str, formatter: &Formatter) -> Result<()> {
    let result = cdp.evaluate(expression).await?;

    if formatter.is_json() {
        formatter.output_json(&result);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Debug storage (localStorage and sessionStorage)
async fn debug_storage(cdp: &CdpConnection, formatter: &Formatter) -> Result<()> {
    let local_storage = cdp.get_local_storage().await?;
    let session_storage = cdp.get_session_storage().await?;

    #[derive(Serialize)]
    struct StorageData {
        local_storage: serde_json::Value,
        session_storage: serde_json::Value,
    }

    let data = StorageData {
        local_storage: serde_json::from_str(local_storage.as_str().unwrap_or("[]"))
            .unwrap_or(serde_json::json!([])),
        session_storage: serde_json::from_str(session_storage.as_str().unwrap_or("[]"))
            .unwrap_or(serde_json::json!([])),
    };

    if formatter.is_json() {
        formatter.output_json(&data);
    } else {
        formatter.header("Local Storage");
        if let Some(arr) = data.local_storage.as_array() {
            for item in arr {
                if let Some(pair) = item.as_array() {
                    let key = pair.first().and_then(|k| k.as_str()).unwrap_or("");
                    let val = pair.get(1).and_then(|v| v.as_str()).unwrap_or("");
                    // Mask sensitive values (tokens, passwords, secrets, etc.)
                    let masked_val = mask_sensitive(&val[..val.len().min(80)]);
                    formatter.kv(key, &masked_val);
                }
            }
            if arr.is_empty() {
                println!("  (empty)");
            }
        }

        formatter.header("Session Storage");
        if let Some(arr) = data.session_storage.as_array() {
            for item in arr {
                if let Some(pair) = item.as_array() {
                    let key = pair.first().and_then(|k| k.as_str()).unwrap_or("");
                    let val = pair.get(1).and_then(|v| v.as_str()).unwrap_or("");
                    // Mask sensitive values (tokens, passwords, secrets, etc.)
                    let masked_val = mask_sensitive(&val[..val.len().min(80)]);
                    formatter.kv(key, &masked_val);
                }
            }
            if arr.is_empty() {
                println!("  (empty)");
            }
        }
    }

    Ok(())
}

/// Debug cookies
async fn debug_cookies(cdp: &CdpConnection, formatter: &Formatter) -> Result<()> {
    let result = cdp.get_cookies().await?;

    if formatter.is_json() {
        formatter.output_json(&result);
    } else {
        formatter.header("Cookies");
        if let Some(arr) = result.as_array() {
            for cookie in arr {
                let name = cookie.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let value = cookie.get("value").and_then(|v| v.as_str()).unwrap_or("");
                // Mask potentially sensitive cookies
                let display_value = if name.to_lowercase().contains("token")
                    || name.to_lowercase().contains("session")
                    || name.to_lowercase().contains("auth")
                {
                    "****"
                } else {
                    &value[..value.len().min(30)]
                };
                formatter.kv(name, display_value);
            }
            if arr.is_empty() {
                println!("  (no cookies)");
            }
        }
    }

    Ok(())
}

/// Debug accessibility tree (ARIA snapshot)
async fn debug_aria(
    cdp: &CdpConnection,
    selector: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    let aria_tree = cdp.get_accessibility_tree(selector).await?;

    if aria_tree.is_null() {
        if let Some(sel) = selector {
            return Err(anyhow::anyhow!("No element matches selector \"{}\"", sel));
        }
        return Err(anyhow::anyhow!("Could not get accessibility tree"));
    }

    if formatter.is_json() {
        formatter.output_json(&aria_tree);
    } else {
        // Convert JSON to AriaNode for human-readable display
        fn json_to_aria_node(value: &serde_json::Value) -> Option<AriaNode> {
            let obj = value.as_object()?;
            Some(AriaNode {
                role: obj.get("role")?.as_str()?.to_string(),
                name: obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                value: obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                description: obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                states: obj
                    .get("states")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                children: obj
                    .get("children")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(json_to_aria_node).collect())
                    .unwrap_or_default(),
            })
        }

        formatter.header("Accessibility Tree");
        if let Some(aria) = json_to_aria_node(&aria_tree) {
            println!("{}", aria);
        }
    }

    Ok(())
}

/// Debug tabs - list, create, switch, close browser tabs
async fn debug_tabs(cdp: &CdpConnection, action: TabCommand, formatter: &Formatter) -> Result<()> {
    match action {
        TabCommand::List => {
            let tabs = cdp.list_tabs().await?;

            if formatter.is_json() {
                formatter.output_json(&tabs);
            } else {
                formatter.header("Browser Tabs");
                if tabs.is_empty() {
                    println!("  (no tabs found)");
                } else {
                    for tab in &tabs {
                        println!("  {} - {}", tab.id, tab.title);
                        println!("    {}", tab.url);
                    }
                    println!("\n  Total: {} tab(s)", tabs.len());
                }
            }
        }
        TabCommand::New { url } => {
            let tab_id = cdp.new_tab(url.as_deref()).await?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "new",
                    "tab_id": tab_id,
                    "url": url.as_deref().unwrap_or("about:blank")
                }));
            } else {
                formatter.success(&format!("Created new tab: {}", tab_id));
                if let Some(u) = url {
                    println!("  URL: {}", u);
                }
            }
        }
        TabCommand::Switch { id } => {
            cdp.switch_tab(&id).await?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "switch",
                    "tab_id": id
                }));
            } else {
                formatter.success(&format!("Switched to tab: {}", id));
            }
        }
        TabCommand::Close { id } => {
            cdp.close_tab(&id).await?;

            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "close",
                    "tab_id": id
                }));
            } else {
                formatter.success(&format!("Closed tab: {}", id));
            }
        }
    }

    Ok(())
}

// ============================================================================
// Chrome DevTools MCP features
// ============================================================================

/// Debug performance metrics (Core Web Vitals and runtime metrics)
async fn debug_performance(cdp: &CdpConnection, formatter: &Formatter) -> Result<()> {
    let metrics = cdp.get_performance_metrics().await?;

    if formatter.is_json() {
        formatter.output_json(&metrics);
    } else {
        formatter.header("Performance Metrics");

        // Core Web Vitals (from Navigation Timing API)
        if let Some(vitals) = metrics.get("web_vitals").and_then(|v| v.as_object()) {
            formatter.header("Core Web Vitals");
            for (key, value) in vitals {
                let formatted_key = match key.as_str() {
                    "lcp" => "Largest Contentful Paint (LCP)",
                    "fid" => "First Input Delay (FID)",
                    "cls" => "Cumulative Layout Shift (CLS)",
                    "fcp" => "First Contentful Paint (FCP)",
                    "ttfb" => "Time to First Byte (TTFB)",
                    _ => key,
                };
                if let Some(num) = value.as_f64() {
                    if key == "cls" {
                        formatter.kv(formatted_key, &format!("{:.3}", num));
                    } else {
                        formatter.kv(formatted_key, &format!("{:.0}ms", num));
                    }
                }
            }
        }

        // CDP Performance metrics
        if let Some(cdp_metrics) = metrics.get("cdp_metrics").and_then(|v| v.as_array()) {
            formatter.header("Runtime Metrics (CDP)");
            for metric in cdp_metrics {
                if let (Some(name), Some(value)) = (
                    metric.get("name").and_then(|n| n.as_str()),
                    metric.get("value").and_then(|v| v.as_f64()),
                ) {
                    // Format important metrics nicely
                    let formatted = match name {
                        "JSHeapUsedSize" | "JSHeapTotalSize" => {
                            format!("{:.2} MB", value / 1_048_576.0)
                        }
                        "LayoutDuration" | "ScriptDuration" | "TaskDuration" => {
                            format!("{:.2}ms", value * 1000.0)
                        }
                        _ => format!("{:.2}", value),
                    };
                    formatter.kv(name, &formatted);
                }
            }
        }
    }

    Ok(())
}

/// Debug snapshot - export full DOM as HTML
async fn debug_snapshot(
    cdp: &CdpConnection,
    output: Option<std::path::PathBuf>,
    formatter: &Formatter,
) -> Result<()> {
    let html = cdp.get_full_dom_snapshot().await?;

    let output_path = output
        .unwrap_or_else(|| std::path::PathBuf::from(format!("snapshot_{}.html", safe_timestamp())));

    std::fs::write(&output_path, &html)?;

    if formatter.is_json() {
        formatter.output_json(&serde_json::json!({
            "action": "snapshot",
            "output": output_path.display().to_string(),
            "size_bytes": html.len()
        }));
    } else {
        formatter.success(&format!("DOM snapshot saved: {}", output_path.display()));
        formatter.kv("Size", &format!("{} bytes", html.len()));
    }

    Ok(())
}

/// Debug throttle - enable CPU or network throttling
async fn debug_throttle(
    cdp: &CdpConnection,
    mode: ThrottleMode,
    formatter: &Formatter,
) -> Result<()> {
    match &mode {
        ThrottleMode::Off => {
            cdp.disable_cpu_throttling().await?;
            cdp.disable_network_throttling().await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "off"
                }));
            } else {
                formatter.success("Throttling disabled");
            }
        }
        ThrottleMode::Cpu { rate } => {
            cdp.set_cpu_throttling(*rate).await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "cpu",
                    "rate": rate
                }));
            } else {
                formatter.success(&format!("CPU throttling enabled: {}x slowdown", rate));
            }
        }
        ThrottleMode::Network3g => {
            // 3G: ~1.6 Mbps down, 750 Kbps up, 300ms latency
            cdp.set_network_throttling(1600.0, 750.0, 300.0, false)
                .await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "network",
                    "preset": "3g"
                }));
            } else {
                formatter.success("Network throttling enabled: 3G (1.6 Mbps down, 300ms latency)");
            }
        }
        ThrottleMode::NetworkSlow3g => {
            // Slow 3G: ~400 Kbps down, 400 Kbps up, 2000ms latency
            cdp.set_network_throttling(400.0, 400.0, 2000.0, false)
                .await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "network",
                    "preset": "slow3g"
                }));
            } else {
                formatter
                    .success("Network throttling enabled: Slow 3G (400 Kbps down, 2s latency)");
            }
        }
        ThrottleMode::NetworkOffline => {
            cdp.set_network_throttling(0.0, 0.0, 0.0, true).await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "network",
                    "preset": "offline"
                }));
            } else {
                formatter.success("Network throttling enabled: Offline mode");
            }
        }
        ThrottleMode::NetworkCustom {
            download_kbps,
            upload_kbps,
            latency_ms,
        } => {
            cdp.set_network_throttling(*download_kbps, *upload_kbps, *latency_ms, false)
                .await?;
            if formatter.is_json() {
                formatter.output_json(&serde_json::json!({
                    "action": "throttle",
                    "mode": "network",
                    "download_kbps": download_kbps,
                    "upload_kbps": upload_kbps,
                    "latency_ms": latency_ms
                }));
            } else {
                formatter.success(&format!(
                    "Network throttling enabled: {} Kbps down, {} Kbps up, {}ms latency",
                    download_kbps, upload_kbps, latency_ms
                ));
            }
        }
    }

    Ok(())
}

/// Debug network details - get detailed info including headers and timing
async fn debug_network_details(
    cdp: &CdpConnection,
    filter: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    let details = cdp.get_network_details(filter).await?;

    if formatter.is_json() {
        formatter.output_json(&details);
    } else {
        formatter.header("Network Request Details");

        if let Some(arr) = details.as_array() {
            if arr.is_empty() {
                println!("  No requests captured. Try refreshing the page first.");
                formatter.hint("Run 'domguard interact refresh' then try again.");
            } else {
                for (i, req) in arr.iter().enumerate() {
                    println!("\n  --- Request {} ---", i + 1);

                    // Basic info
                    if let Some(url) = req.get("url").and_then(|v| v.as_str()) {
                        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                        println!("  {} {}", method, url);
                    }

                    // Status
                    if let Some(status) = req.get("status").and_then(|v| v.as_i64()) {
                        let status_text = req
                            .get("status_text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        println!("  Status: {} {}", status, status_text);
                    }

                    // Type
                    if let Some(res_type) = req.get("type").and_then(|v| v.as_str()) {
                        println!("  Type: {}", res_type);
                    }

                    // Timing
                    if let Some(timing) = req.get("timing").and_then(|v| v.as_object()) {
                        println!("  Timing:");
                        if let Some(dns) = timing.get("dns").and_then(|v| v.as_f64()) {
                            println!("    DNS: {:.1}ms", dns);
                        }
                        if let Some(connect) = timing.get("connect").and_then(|v| v.as_f64()) {
                            println!("    Connect: {:.1}ms", connect);
                        }
                        if let Some(ttfb) = timing.get("ttfb").and_then(|v| v.as_f64()) {
                            println!("    TTFB: {:.1}ms", ttfb);
                        }
                        if let Some(download) = timing.get("download").and_then(|v| v.as_f64()) {
                            println!("    Download: {:.1}ms", download);
                        }
                        if let Some(total) = timing.get("total").and_then(|v| v.as_f64()) {
                            println!("    Total: {:.1}ms", total);
                        }
                    }

                    // Size
                    if let Some(size) = req.get("size_bytes").and_then(|v| v.as_u64()) {
                        if size > 1024 * 1024 {
                            println!("  Size: {:.2} MB", size as f64 / 1_048_576.0);
                        } else if size > 1024 {
                            println!("  Size: {:.2} KB", size as f64 / 1024.0);
                        } else {
                            println!("  Size: {} bytes", size);
                        }
                    }

                    // Headers (abbreviated)
                    if let Some(headers) = req.get("response_headers").and_then(|v| v.as_object()) {
                        println!("  Response Headers:");
                        let important_headers = [
                            "content-type",
                            "cache-control",
                            "content-encoding",
                            "server",
                        ];
                        for header in important_headers {
                            if let Some(val) = headers.get(header).and_then(|v| v.as_str()) {
                                println!("    {}: {}", header, val);
                            }
                        }
                    }
                }
                println!("\n  Total: {} request(s)", arr.len());
            }
        }

        if let Some(f) = filter {
            println!("  Filter: {}", f);
        }
    }

    Ok(())
}

/// Highlight element(s) on the page
async fn debug_highlight(
    cdp: &CdpConnection,
    selector: &str,
    color: &str,
    duration: u64,
    all: bool,
    formatter: &Formatter,
) -> Result<()> {
    if all {
        // Highlight all matching elements with numbered labels
        let result = cdp.highlight_elements(selector, color).await?;

        if formatter.is_json() {
            formatter.output_json(&result);
        } else {
            formatter.header("Highlighted Elements");
            if let Some(count) = result.get("count").and_then(|c| c.as_u64()) {
                println!("  {} element(s) highlighted", count);
            }
            if let Some(elements) = result.get("elements").and_then(|e| e.as_array()) {
                for el in elements {
                    let idx = el.get("index").and_then(|i| i.as_u64()).unwrap_or(0);
                    let tag = el.get("tagName").and_then(|t| t.as_str()).unwrap_or("?");
                    let id = el.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    let classes = el.get("classes").and_then(|c| c.as_str()).unwrap_or("");

                    let mut desc = format!("  [{}] <{}", idx, tag);
                    if !id.is_empty() {
                        let _ = write!(desc, " id=\"{}\"", id);
                    }
                    if !classes.is_empty() {
                        let _ = write!(desc, " class=\"{}\"", classes);
                    }
                    desc.push('>');
                    println!("{}", desc);
                }
            }
            formatter.hint("Use 'domguard debug clear-highlights' to remove");
        }
    } else {
        // Highlight single element
        cdp.highlight_element(selector, color, duration).await?;

        if formatter.is_json() {
            formatter.output_json(&serde_json::json!({
                "success": true,
                "selector": selector,
                "color": color,
                "duration_ms": duration
            }));
        } else {
            println!("Highlighted element: {}", selector);
            if duration > 0 {
                println!("  Auto-cleared after {}ms", duration);
            } else {
                formatter.hint("Use 'domguard debug clear-highlights' to remove");
            }
        }
    }

    Ok(())
}

/// Clear all highlights from the page
async fn debug_clear_highlights(cdp: &CdpConnection, formatter: &Formatter) -> Result<()> {
    cdp.clear_all_highlights().await?;

    if formatter.is_json() {
        formatter.output_json(&serde_json::json!({
            "success": true,
            "cleared": true
        }));
    } else {
        println!("Cleared all highlights");
    }

    Ok(())
}

/// Detect CAPTCHAs on the current page
async fn debug_captcha(cdp: &CdpConnection, formatter: &Formatter) -> Result<()> {
    use crate::captcha::{
        captcha_detection_script, format_captcha_detection, parse_captcha_detection,
    };

    let script = captcha_detection_script();
    let result = cdp.evaluate(script).await?;
    let detection = parse_captcha_detection(&result);

    if formatter.is_json() {
        formatter.output_json(&detection);
    } else {
        println!("{}", format_captcha_detection(&detection));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_timestamp() {
        let ts = safe_timestamp();
        // Should be a reasonable Unix timestamp (after 2020)
        assert!(ts > 1577836800, "timestamp should be after 2020");
    }

    #[test]
    fn test_tab_command_variants() {
        let list = TabCommand::List;
        let new = TabCommand::New {
            url: Some("https://example.com".to_string()),
        };
        let switch = TabCommand::Switch {
            id: "abc123".to_string(),
        };
        let close = TabCommand::Close {
            id: "def456".to_string(),
        };

        // Test debug formatting
        assert!(format!("{:?}", list).contains("List"));
        assert!(format!("{:?}", new).contains("example.com"));
        assert!(format!("{:?}", switch).contains("abc123"));
        assert!(format!("{:?}", close).contains("def456"));
    }

    #[test]
    fn test_throttle_mode_variants() {
        let slow_3g = ThrottleMode::NetworkSlow3g;
        let net_3g = ThrottleMode::Network3g;
        let offline = ThrottleMode::NetworkOffline;
        let off = ThrottleMode::Off;

        // Verify all variants exist
        assert!(format!("{:?}", slow_3g).contains("NetworkSlow3g"));
        assert!(format!("{:?}", net_3g).contains("Network3g"));
        assert!(format!("{:?}", offline).contains("NetworkOffline"));
        assert!(format!("{:?}", off).contains("Off"));
    }

    #[test]
    fn test_debug_command_variants() {
        let dom = DebugCommand::Dom {
            selector: Some("body".to_string()),
        };
        let console = DebugCommand::Console {
            follow: false,
            filter: None,
        };
        let network = DebugCommand::Network { filter: None };
        let storage = DebugCommand::Storage;

        assert!(format!("{:?}", dom).contains("Dom"));
        assert!(format!("{:?}", console).contains("Console"));
        assert!(format!("{:?}", network).contains("Network"));
        assert!(format!("{:?}", storage).contains("Storage"));
    }
}
