//! Interact mode - Control browser like a human
//!
//! Mouse events, keyboard input, navigation, screenshots, wait conditions

use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

use crate::cdp::CdpConnection;
use crate::config::Config;
use crate::output::{CommandResult, Formatter};

/// Get current timestamp in seconds, with fallback to 0 if system clock is before UNIX epoch
fn safe_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Result data for interact commands
#[derive(Debug, Serialize)]
pub struct InteractResult {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl std::fmt::Display for InteractResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.action)?;
        if let Some(target) = &self.target {
            write!(f, " \"{}\"", target)?;
        }
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

/// Interact subcommand types
#[derive(Debug, Clone)]
pub enum InteractCommand {
    Click {
        selector: Option<String>,
        coords: Option<(f64, f64)>,
        nth: i32,
        text: Option<String>,
    },
    Type {
        selector: Option<String>,
        text: Option<String>,
        focused: bool,
    },
    Key {
        keys: String,
    },
    Hover {
        selector: String,
    },
    Scroll {
        down: Option<i64>,
        up: Option<i64>,
        left: Option<i64>,
        right: Option<i64>,
        to: Option<String>,
    },
    Screenshot {
        full: bool,
        element: Option<String>,
        output: Option<PathBuf>,
    },
    Navigate {
        url: String,
    },
    Back,
    Refresh,
    Wait {
        selector: String,
        visible: bool,
        gone: bool,
        timeout_ms: u64,
        text: Option<String>,
        text_gone: Option<String>,
    },
    Drag {
        from_selector: Option<String>,
        to_selector: Option<String>,
        from_coords: Option<(f64, f64)>,
        to_coords: Option<(f64, f64)>,
    },
    Select {
        selector: String,
        value: String,
        by_label: bool,
        by_index: bool,
    },
    Upload {
        selector: String,
        files: Vec<PathBuf>,
    },
    Dialog {
        accept: bool,
        text: Option<String>,
    },
    Resize {
        width: u32,
        height: u32,
    },
    Pdf {
        output: Option<PathBuf>,
        landscape: bool,
    },
    // Anthropic Computer Use features
    MouseMove {
        coords: (f64, f64),
    },
    CursorPosition,
    HoldKey {
        key: String,
        duration_ms: u64,
    },
    TripleClick {
        selector: Option<String>,
        coords: Option<(f64, f64)>,
    },
    MouseDown {
        button: String,
    },
    MouseUp {
        button: String,
    },
    ScreenshotRegion {
        region: (i32, i32, i32, i32), // x, y, width, height
        output: Option<PathBuf>,
    },
    WaitDuration {
        duration_ms: u64,
    },
}

/// Run interact command
pub async fn run_interact(
    cdp: &CdpConnection,
    config: &Config,
    command: InteractCommand,
    formatter: &Formatter,
) -> Result<()> {
    let start = Instant::now();

    let result = match command {
        InteractCommand::Click {
            selector,
            coords,
            nth,
            text,
        } => {
            interact_click(
                cdp,
                selector.as_deref(),
                coords,
                nth,
                text.as_deref(),
                formatter,
            )
            .await
        }
        InteractCommand::Type {
            selector,
            text,
            focused,
        } => {
            // When --focused is used, the first positional arg (selector) is actually the text
            let (actual_selector, actual_text) = if focused {
                (None, selector.as_ref().or(text.as_ref()))
            } else {
                (selector.as_ref(), text.as_ref())
            };
            match actual_text {
                Some(t) => {
                    interact_type(
                        cdp,
                        actual_selector.map(|s| s.as_str()),
                        t,
                        focused,
                        formatter,
                    )
                    .await
                }
                None => Err(anyhow::anyhow!("Text to type is required")),
            }
        }
        InteractCommand::Key { keys } => interact_key(cdp, &keys, formatter).await,
        InteractCommand::Hover { selector } => interact_hover(cdp, &selector, formatter).await,
        InteractCommand::Scroll {
            down,
            up,
            left,
            right,
            to,
        } => interact_scroll(cdp, down, up, left, right, to.as_deref(), formatter).await,
        InteractCommand::Screenshot {
            full,
            element,
            output,
        } => interact_screenshot(cdp, config, full, element.as_deref(), output, formatter).await,
        InteractCommand::Navigate { url } => interact_navigate(cdp, &url, formatter).await,
        InteractCommand::Back => interact_back(cdp, formatter).await,
        InteractCommand::Refresh => interact_refresh(cdp, formatter).await,
        InteractCommand::Wait {
            selector,
            visible,
            gone,
            timeout_ms,
            text,
            text_gone,
        } => {
            interact_wait(
                cdp,
                &selector,
                visible,
                gone,
                timeout_ms,
                text.as_deref(),
                text_gone.as_deref(),
                formatter,
            )
            .await
        }
        InteractCommand::Drag {
            from_selector,
            to_selector,
            from_coords,
            to_coords,
        } => {
            interact_drag(
                cdp,
                from_selector.as_deref(),
                to_selector.as_deref(),
                from_coords,
                to_coords,
                formatter,
            )
            .await
        }
        InteractCommand::Select {
            selector,
            value,
            by_label,
            by_index,
        } => interact_select(cdp, &selector, &value, by_label, by_index, formatter).await,
        InteractCommand::Upload { selector, files } => {
            interact_upload(cdp, &selector, &files, formatter).await
        }
        InteractCommand::Dialog { accept, text } => {
            interact_dialog(cdp, accept, text.as_deref(), formatter).await
        }
        InteractCommand::Resize { width, height } => {
            interact_resize(cdp, width, height, formatter).await
        }
        InteractCommand::Pdf { output, landscape } => {
            interact_pdf(cdp, config, output, landscape, formatter).await
        }
        // Anthropic Computer Use features
        InteractCommand::MouseMove { coords } => interact_mouse_move(cdp, coords, formatter).await,
        InteractCommand::CursorPosition => interact_cursor_position(cdp, formatter).await,
        InteractCommand::HoldKey { key, duration_ms } => {
            interact_hold_key(cdp, &key, duration_ms, formatter).await
        }
        InteractCommand::TripleClick { selector, coords } => {
            interact_triple_click(cdp, selector.as_deref(), coords, formatter).await
        }
        InteractCommand::MouseDown { button } => interact_mouse_down(cdp, &button, formatter).await,
        InteractCommand::MouseUp { button } => interact_mouse_up(cdp, &button, formatter).await,
        InteractCommand::ScreenshotRegion { region, output } => {
            interact_screenshot_region(cdp, config, region, output, formatter).await
        }
        InteractCommand::WaitDuration { duration_ms } => {
            interact_wait_duration(duration_ms, formatter).await
        }
    }?;

    let elapsed = start.elapsed().as_millis() as u64;

    // Use formatter.output for consistent output formatting
    let cmd_result = CommandResult::success(result).with_timing(elapsed);
    formatter.output(&cmd_result);

    Ok(())
}

/// Click element or coordinates
async fn interact_click(
    cdp: &CdpConnection,
    selector: Option<&str>,
    coords: Option<(f64, f64)>,
    nth: i32,
    text: Option<&str>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    if let Some((x, y)) = coords {
        cdp.click_at(x, y).await?;
        formatter.success(&format!("Clicked at ({}, {})", x, y));
        Ok(InteractResult {
            action: "click".to_string(),
            target: Some(format!("({}, {})", x, y)),
            details: None,
        })
    } else if let Some(txt) = text {
        cdp.click_by_text(txt, nth).await?;
        let nth_info = if nth != 0 {
            format!(" (nth: {})", nth)
        } else {
            String::new()
        };
        formatter.success(&format!("Clicked text \"{}\"{}", txt, nth_info));
        Ok(InteractResult {
            action: "click".to_string(),
            target: Some(format!("text:{}", txt)),
            details: if nth != 0 {
                Some(format!("nth: {}", nth))
            } else {
                None
            },
        })
    } else if let Some(sel) = selector {
        cdp.click(sel, nth).await?;
        let nth_info = if nth != 0 {
            format!(" (nth: {})", nth)
        } else {
            String::new()
        };
        formatter.success(&format!("Clicked \"{}\"{}", sel, nth_info));
        Ok(InteractResult {
            action: "click".to_string(),
            target: Some(sel.to_string()),
            details: if nth != 0 {
                Some(format!("nth: {}", nth))
            } else {
                None
            },
        })
    } else {
        Err(anyhow::anyhow!(
            "Either selector, --text, or --coords required"
        ))
    }
}

/// Type text into element or focused element
async fn interact_type(
    cdp: &CdpConnection,
    selector: Option<&str>,
    text: &str,
    focused: bool,
    formatter: &Formatter,
) -> Result<InteractResult> {
    if focused {
        cdp.type_focused(text).await?;
        formatter.success("Typed into focused element");
        Ok(InteractResult {
            action: "type".to_string(),
            target: Some("focused".to_string()),
            details: None, // Don't log text for security
        })
    } else if let Some(sel) = selector {
        cdp.type_into(sel, text).await?;
        formatter.success(&format!("Typed into \"{}\"", sel));
        Ok(InteractResult {
            action: "type".to_string(),
            target: Some(sel.to_string()),
            details: None, // Don't log text for security
        })
    } else {
        Err(anyhow::anyhow!("Either selector or --focused required"))
    }
}

/// Press key or key sequence
async fn interact_key(
    cdp: &CdpConnection,
    keys: &str,
    formatter: &Formatter,
) -> Result<InteractResult> {
    // Parse key sequence (space-separated)
    let key_list: Vec<&str> = keys.split_whitespace().collect();

    for key in &key_list {
        // Handle modifier combinations like "cmd+k"
        if key.contains('+') {
            let parts: Vec<&str> = key.split('+').collect();
            // For now, just press the final key (full modifier support needs more work)
            if let Some(final_key) = parts.last() {
                cdp.press_key(final_key).await?;
            }
        } else {
            cdp.press_key(key).await?;
        }

        // Small delay between keys
        if key_list.len() > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    formatter.success(&format!("Pressed: {}", keys));
    Ok(InteractResult {
        action: "key".to_string(),
        target: Some(keys.to_string()),
        details: None,
    })
}

/// Hover over element
async fn interact_hover(
    cdp: &CdpConnection,
    selector: &str,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.hover(selector).await?;
    formatter.success(&format!("Hovering over \"{}\"", selector));
    Ok(InteractResult {
        action: "hover".to_string(),
        target: Some(selector.to_string()),
        details: None,
    })
}

/// Scroll page
async fn interact_scroll(
    cdp: &CdpConnection,
    down: Option<i64>,
    up: Option<i64>,
    left: Option<i64>,
    right: Option<i64>,
    to: Option<&str>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    if let Some(sel) = to {
        cdp.scroll_to_element(sel).await?;
        formatter.success(&format!("Scrolled to \"{}\"", sel));
        Ok(InteractResult {
            action: "scroll".to_string(),
            target: Some(sel.to_string()),
            details: None,
        })
    } else {
        let x = right.unwrap_or(0) - left.unwrap_or(0);
        let y = down.unwrap_or(0) - up.unwrap_or(0);
        cdp.scroll_by(x, y).await?;
        formatter.success(&format!("Scrolled by ({}, {})", x, y));
        Ok(InteractResult {
            action: "scroll".to_string(),
            target: Some(format!("({}, {})", x, y)),
            details: None,
        })
    }
}

/// Capture screenshot
async fn interact_screenshot(
    cdp: &CdpConnection,
    _config: &Config,
    full: bool,
    element: Option<&str>,
    output: Option<PathBuf>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    let data = if let Some(sel) = element {
        // Element screenshot via JS
        let js = format!(
            r#"
            (async function() {{
                const el = document.querySelector('{}');
                if (!el) return null;
                el.scrollIntoView({{ block: 'center' }});
                await new Promise(r => setTimeout(r, 100));
                const rect = el.getBoundingClientRect();
                return {{ x: rect.x, y: rect.y, width: rect.width, height: rect.height }};
            }})()
        "#,
            sel
        );

        let rect = cdp.evaluate(&js).await?;
        if rect.is_null() {
            return Err(anyhow::anyhow!("No element matches selector \"{}\"", sel));
        }

        // For now, take full screenshot (element clipping needs CDP clip parameter)
        cdp.screenshot(false).await?
    } else {
        cdp.screenshot(full).await?
    };

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        Config::find_domguard_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join("screenshots")
            .join(format!("screenshot_{}.png", safe_timestamp()))
    });

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, &data)?;
    formatter.success(&format!("Screenshot saved: {}", output_path.display()));

    Ok(InteractResult {
        action: "screenshot".to_string(),
        target: None,
        details: Some(output_path.display().to_string()),
    })
}

/// Navigate to URL
async fn interact_navigate(
    cdp: &CdpConnection,
    url: &str,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.navigate(url).await?;

    // Wait a bit for navigation
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let title = cdp.get_title().await.unwrap_or_default();
    formatter.success(&format!("Navigated to: {} - {}", url, title));

    Ok(InteractResult {
        action: "navigate".to_string(),
        target: Some(url.to_string()),
        details: Some(title),
    })
}

/// Go back in history
async fn interact_back(cdp: &CdpConnection, formatter: &Formatter) -> Result<InteractResult> {
    cdp.go_back().await?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    formatter.success("Navigated back");
    Ok(InteractResult {
        action: "back".to_string(),
        target: None,
        details: None,
    })
}

/// Refresh page
async fn interact_refresh(cdp: &CdpConnection, formatter: &Formatter) -> Result<InteractResult> {
    cdp.refresh().await?;
    formatter.success("Page refreshed");
    Ok(InteractResult {
        action: "refresh".to_string(),
        target: None,
        details: None,
    })
}

/// Wait for element or text
#[allow(clippy::too_many_arguments)]
async fn interact_wait(
    cdp: &CdpConnection,
    selector: &str,
    _visible: bool,
    gone: bool,
    timeout_ms: u64,
    text: Option<&str>,
    text_gone: Option<&str>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    // Text-based wait
    if let Some(wait_text) = text {
        cdp.wait_for_text(wait_text, timeout_ms).await?;
        formatter.success(&format!("Text \"{}\" found", wait_text));
        return Ok(InteractResult {
            action: "wait".to_string(),
            target: Some(wait_text.to_string()),
            details: Some("text_found".to_string()),
        });
    }

    if let Some(wait_text) = text_gone {
        cdp.wait_for_text_gone(wait_text, timeout_ms).await?;
        formatter.success(&format!("Text \"{}\" is gone", wait_text));
        return Ok(InteractResult {
            action: "wait".to_string(),
            target: Some(wait_text.to_string()),
            details: Some("text_gone".to_string()),
        });
    }

    // Element-based wait
    if gone {
        cdp.wait_for_gone(selector, timeout_ms).await?;
        formatter.success(&format!("Element \"{}\" is gone", selector));
        Ok(InteractResult {
            action: "wait".to_string(),
            target: Some(selector.to_string()),
            details: Some("gone".to_string()),
        })
    } else {
        cdp.wait_for(selector, timeout_ms).await?;
        formatter.success(&format!("Element \"{}\" found", selector));
        Ok(InteractResult {
            action: "wait".to_string(),
            target: Some(selector.to_string()),
            details: Some("found".to_string()),
        })
    }
}

/// Drag and drop
async fn interact_drag(
    cdp: &CdpConnection,
    from_selector: Option<&str>,
    to_selector: Option<&str>,
    from_coords: Option<(f64, f64)>,
    to_coords: Option<(f64, f64)>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    let (from_x, from_y) = if let Some((x, y)) = from_coords {
        (x, y)
    } else if let Some(sel) = from_selector {
        cdp.get_element_center(sel).await?
    } else {
        return Err(anyhow::anyhow!(
            "Either from_selector or from_coords required"
        ));
    };

    let (to_x, to_y) = if let Some((x, y)) = to_coords {
        (x, y)
    } else if let Some(sel) = to_selector {
        cdp.get_element_center(sel).await?
    } else {
        return Err(anyhow::anyhow!("Either to_selector or to_coords required"));
    };

    cdp.drag(from_x, from_y, to_x, to_y).await?;

    let from_str = from_selector
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("({}, {})", from_x, from_y));
    let to_str = to_selector
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("({}, {})", to_x, to_y));

    formatter.success(&format!("Dragged from {} to {}", from_str, to_str));
    Ok(InteractResult {
        action: "drag".to_string(),
        target: Some(from_str),
        details: Some(to_str),
    })
}

/// Select dropdown option
async fn interact_select(
    cdp: &CdpConnection,
    selector: &str,
    value: &str,
    by_label: bool,
    by_index: bool,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.select_option(selector, value, by_label, by_index)
        .await?;

    let method = if by_label {
        "label"
    } else if by_index {
        "index"
    } else {
        "value"
    };
    formatter.success(&format!(
        "Selected {} \"{}\" in \"{}\"",
        method, value, selector
    ));
    Ok(InteractResult {
        action: "select".to_string(),
        target: Some(selector.to_string()),
        details: Some(value.to_string()),
    })
}

/// Upload files
async fn interact_upload(
    cdp: &CdpConnection,
    selector: &str,
    files: &[PathBuf],
    formatter: &Formatter,
) -> Result<InteractResult> {
    // Validate files exist
    for file in files {
        if !file.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file.display()));
        }
    }

    cdp.upload_files(selector, files).await?;

    let file_names: Vec<_> = files
        .iter()
        .filter_map(|f| f.file_name())
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    formatter.success(&format!(
        "Uploaded {} file(s) to \"{}\"",
        files.len(),
        selector
    ));
    Ok(InteractResult {
        action: "upload".to_string(),
        target: Some(selector.to_string()),
        details: Some(file_names.join(", ")),
    })
}

/// Handle JavaScript dialog
async fn interact_dialog(
    cdp: &CdpConnection,
    accept: bool,
    text: Option<&str>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.handle_dialog(accept, text).await?;

    let action_str = if accept { "accepted" } else { "dismissed" };
    formatter.success(&format!("Dialog {}", action_str));
    Ok(InteractResult {
        action: "dialog".to_string(),
        target: None,
        details: Some(action_str.to_string()),
    })
}

/// Resize viewport
async fn interact_resize(
    cdp: &CdpConnection,
    width: u32,
    height: u32,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.resize_viewport(width, height).await?;

    formatter.success(&format!("Resized viewport to {}x{}", width, height));
    Ok(InteractResult {
        action: "resize".to_string(),
        target: Some(format!("{}x{}", width, height)),
        details: None,
    })
}

/// Export page as PDF
async fn interact_pdf(
    cdp: &CdpConnection,
    _config: &Config,
    output: Option<PathBuf>,
    landscape: bool,
    formatter: &Formatter,
) -> Result<InteractResult> {
    let data = cdp.print_to_pdf(landscape).await?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        Config::find_domguard_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join("screenshots")
            .join(format!("page_{}.pdf", safe_timestamp()))
    });

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, &data)?;
    formatter.success(&format!("PDF saved: {}", output_path.display()));

    Ok(InteractResult {
        action: "pdf".to_string(),
        target: None,
        details: Some(output_path.display().to_string()),
    })
}

// ============================================================================
// Anthropic Computer Use feature implementations
// ============================================================================

/// Move mouse cursor to coordinates without clicking
async fn interact_mouse_move(
    cdp: &CdpConnection,
    coords: (f64, f64),
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.mouse_move(coords.0, coords.1).await?;
    formatter.success(&format!("Moved cursor to ({}, {})", coords.0, coords.1));
    Ok(InteractResult {
        action: "mouse_move".to_string(),
        target: Some(format!("({}, {})", coords.0, coords.1)),
        details: None,
    })
}

/// Get current cursor position
async fn interact_cursor_position(
    cdp: &CdpConnection,
    formatter: &Formatter,
) -> Result<InteractResult> {
    let (x, y) = cdp.cursor_position().await?;
    formatter.success(&format!("Cursor at ({}, {})", x, y));
    Ok(InteractResult {
        action: "cursor_position".to_string(),
        target: Some(format!("({}, {})", x, y)),
        details: None,
    })
}

/// Hold a key for specified duration
async fn interact_hold_key(
    cdp: &CdpConnection,
    key: &str,
    duration_ms: u64,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.hold_key(key, duration_ms).await?;
    formatter.success(&format!("Held key \"{}\" for {}ms", key, duration_ms));
    Ok(InteractResult {
        action: "hold_key".to_string(),
        target: Some(key.to_string()),
        details: Some(format!("{}ms", duration_ms)),
    })
}

/// Triple-click to select paragraph
async fn interact_triple_click(
    cdp: &CdpConnection,
    selector: Option<&str>,
    coords: Option<(f64, f64)>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    if let Some((x, y)) = coords {
        cdp.triple_click_at(x, y).await?;
        formatter.success(&format!("Triple-clicked at ({}, {})", x, y));
        Ok(InteractResult {
            action: "triple_click".to_string(),
            target: Some(format!("({}, {})", x, y)),
            details: None,
        })
    } else if let Some(sel) = selector {
        cdp.triple_click(sel).await?;
        formatter.success(&format!("Triple-clicked \"{}\"", sel));
        Ok(InteractResult {
            action: "triple_click".to_string(),
            target: Some(sel.to_string()),
            details: None,
        })
    } else {
        Err(anyhow::anyhow!("Either selector or --coords required"))
    }
}

/// Press mouse button down (without releasing)
async fn interact_mouse_down(
    cdp: &CdpConnection,
    button: &str,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.mouse_down(button).await?;
    formatter.success(&format!("Mouse {} button down", button));
    Ok(InteractResult {
        action: "mouse_down".to_string(),
        target: Some(button.to_string()),
        details: None,
    })
}

/// Release mouse button
async fn interact_mouse_up(
    cdp: &CdpConnection,
    button: &str,
    formatter: &Formatter,
) -> Result<InteractResult> {
    cdp.mouse_up(button).await?;
    formatter.success(&format!("Mouse {} button up", button));
    Ok(InteractResult {
        action: "mouse_up".to_string(),
        target: Some(button.to_string()),
        details: None,
    })
}

/// Capture screenshot of specific region (zoom/crop)
async fn interact_screenshot_region(
    cdp: &CdpConnection,
    _config: &Config,
    region: (i32, i32, i32, i32),
    output: Option<PathBuf>,
    formatter: &Formatter,
) -> Result<InteractResult> {
    let (x, y, width, height) = region;
    let data = cdp.screenshot_region(x, y, width, height).await?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        Config::find_domguard_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join("screenshots")
            .join(format!("region_{}.png", safe_timestamp()))
    });

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&output_path, &data)?;
    formatter.success(&format!(
        "Screenshot region saved: {}",
        output_path.display()
    ));

    Ok(InteractResult {
        action: "screenshot_region".to_string(),
        target: Some(format!("({}, {}, {}x{})", x, y, width, height)),
        details: Some(output_path.display().to_string()),
    })
}

/// Wait for specified duration
async fn interact_wait_duration(duration_ms: u64, formatter: &Formatter) -> Result<InteractResult> {
    tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
    formatter.success(&format!("Waited {}ms", duration_ms));
    Ok(InteractResult {
        action: "wait_duration".to_string(),
        target: Some(format!("{}ms", duration_ms)),
        details: None,
    })
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
    fn test_interact_result_display() {
        let result = InteractResult {
            action: "click".to_string(),
            target: Some("#button".to_string()),
            details: Some("clicked".to_string()),
        };
        let display = format!("{}", result);
        assert!(display.contains("click"));
        assert!(display.contains("#button"));
        assert!(display.contains("clicked"));
    }

    #[test]
    fn test_interact_result_display_minimal() {
        let result = InteractResult {
            action: "navigate".to_string(),
            target: None,
            details: None,
        };
        let display = format!("{}", result);
        assert_eq!(display, "navigate");
    }

    #[test]
    fn test_interact_result_serialize() {
        let result = InteractResult {
            action: "type".to_string(),
            target: Some("input".to_string()),
            details: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"action\":\"type\""));
        assert!(json.contains("\"target\":\"input\""));
        // details should be skipped when None
        assert!(!json.contains("details"));
    }

    #[test]
    fn test_interact_command_variants() {
        // Test that all command variants can be created
        let _click = InteractCommand::Click {
            selector: Some("#btn".to_string()),
            coords: None,
            nth: 0,
            text: None,
        };
        let _type = InteractCommand::Type {
            selector: Some("input".to_string()),
            text: Some("hello".to_string()),
            focused: false,
        };
        let _nav = InteractCommand::Navigate {
            url: "https://example.com".to_string(),
        };
    }
}
