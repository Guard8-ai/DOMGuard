//! Action explanation mode for DOMGuard
//!
//! Provides human-readable explanations for why each action is being taken.
//! Used for debugging, teaching, and AI agent transparency.

use serde::{Deserialize, Serialize};

/// Context for generating explanations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplanationContext {
    /// Current page URL
    pub current_url: Option<String>,
    /// Current page title
    pub current_title: Option<String>,
    /// Previous action taken
    pub previous_action: Option<String>,
    /// Current task/goal (if known)
    pub current_goal: Option<String>,
}

/// Action explanation with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExplanation {
    /// The action being performed
    pub action: String,
    /// Human-readable explanation of why this action
    pub reason: String,
    /// What we expect to happen
    pub expected_outcome: String,
    /// How this relates to the overall goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_context: Option<String>,
}

/// Generate explanation for an action
pub fn explain_action(action: &str, target: Option<&str>, context: &ExplanationContext) -> ActionExplanation {
    match action {
        // Click actions
        "click" => explain_click(target, context),
        "triple_click" => explain_triple_click(target, context),

        // Type actions
        "type" => explain_type(target, context),
        "key" => explain_key(target, context),
        "hold_key" => explain_hold_key(target, context),

        // Navigation
        "navigate" => explain_navigate(target, context),
        "back" => explain_back(context),
        "refresh" => explain_refresh(context),

        // Mouse movement
        "hover" => explain_hover(target, context),
        "mouse_move" => explain_mouse_move(target, context),
        "mouse_down" => explain_mouse_down(target, context),
        "mouse_up" => explain_mouse_up(target, context),
        "drag" => explain_drag(target, context),

        // Wait actions
        "wait" => explain_wait(target, context),
        "wait_duration" => explain_wait_duration(target, context),

        // Form actions
        "select" => explain_select(target, context),
        "upload" => explain_upload(target, context),
        "dialog" => explain_dialog(target, context),

        // Screenshot/capture
        "screenshot" => explain_screenshot(target, context),
        "screenshot_region" => explain_screenshot_region(target, context),
        "pdf" => explain_pdf(context),

        // Scroll
        "scroll" => explain_scroll(target, context),

        // Viewport
        "resize" => explain_resize(target, context),

        // Cursor
        "cursor_position" => explain_cursor_position(context),

        // Default
        _ => ActionExplanation {
            action: action.to_string(),
            reason: format!("Performing {} action", action),
            expected_outcome: "Action will be executed".to_string(),
            goal_context: context.current_goal.clone(),
        },
    }
}

fn explain_click(target: Option<&str>, context: &ExplanationContext) -> ActionExplanation {
    let target_str = target.unwrap_or("element");

    // Infer purpose from selector
    let (reason, expected) = if let Some(sel) = target {
        let sel_lower = sel.to_lowercase();

        if sel_lower.contains("submit") || sel_lower.contains("login") || sel_lower.contains("sign") {
            ("Submitting the form by clicking the submit button".to_string(),
             "Form will be submitted and page may navigate".to_string())
        } else if sel_lower.contains("button") || sel_lower.contains("btn") {
            ("Clicking a button to trigger an action".to_string(),
             "Button action will be executed".to_string())
        } else if sel_lower.contains("link") || sel.starts_with("a[") || sel.starts_with("a.") {
            ("Clicking a link to navigate to another page".to_string(),
             "Page will navigate to the link destination".to_string())
        } else if sel_lower.contains("menu") || sel_lower.contains("nav") {
            ("Opening or selecting from a menu".to_string(),
             "Menu will open or selection will be made".to_string())
        } else if sel_lower.contains("close") || sel_lower.contains("dismiss") || sel_lower.contains("x") {
            ("Closing a dialog, popup, or notification".to_string(),
             "Element will close or be dismissed".to_string())
        } else if sel_lower.contains("checkbox") || sel.contains("[type=checkbox]") {
            ("Toggling a checkbox option".to_string(),
             "Checkbox state will change".to_string())
        } else if sel_lower.contains("radio") || sel.contains("[type=radio]") {
            ("Selecting a radio button option".to_string(),
             "Radio option will be selected".to_string())
        } else if sel_lower.contains("tab") {
            ("Switching to a different tab or section".to_string(),
             "Tab content will be displayed".to_string())
        } else if sel_lower.contains("expand") || sel_lower.contains("collapse") || sel_lower.contains("accordion") {
            ("Expanding or collapsing a section".to_string(),
             "Section visibility will toggle".to_string())
        } else if sel.starts_with("(") && sel.contains(",") {
            // Coordinates
            ("Clicking at specific screen coordinates".to_string(),
             "Click event will be triggered at location".to_string())
        } else {
            (format!("Clicking on element matching '{}'", sel),
             "Element will receive click event".to_string())
        }
    } else {
        ("Clicking on specified element".to_string(),
         "Click event will be triggered".to_string())
    };

    ActionExplanation {
        action: format!("click {}", target_str),
        reason,
        expected_outcome: expected,
        goal_context: context.current_goal.clone(),
    }
}

fn explain_triple_click(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("triple_click {}", target.unwrap_or("element")),
        reason: "Triple-clicking to select an entire paragraph or text block".to_string(),
        expected_outcome: "Text will be selected for copying or manipulation".to_string(),
        goal_context: None,
    }
}

fn explain_type(target: Option<&str>, context: &ExplanationContext) -> ActionExplanation {
    let target_str = target.unwrap_or("field");

    let (reason, expected) = if let Some(sel) = target {
        let sel_lower = sel.to_lowercase();

        if sel_lower.contains("search") {
            ("Entering search query into search field".to_string(),
             "Search results will be populated".to_string())
        } else if sel_lower.contains("email") {
            ("Entering email address".to_string(),
             "Email field will be filled".to_string())
        } else if sel_lower.contains("password") {
            ("Entering password (credentials masked)".to_string(),
             "Password field will be filled".to_string())
        } else if sel_lower.contains("username") || sel_lower.contains("user") {
            ("Entering username".to_string(),
             "Username field will be filled".to_string())
        } else if sel_lower.contains("comment") || sel_lower.contains("message") || sel_lower.contains("textarea") {
            ("Entering text content".to_string(),
             "Text area will be filled".to_string())
        } else if sel_lower.contains("address") {
            ("Entering address information".to_string(),
             "Address field will be filled".to_string())
        } else if sel == "focused" {
            ("Typing into currently focused element".to_string(),
             "Focused element will receive text".to_string())
        } else {
            (format!("Typing text into '{}'", sel),
             "Text will be entered into the field".to_string())
        }
    } else {
        ("Typing text into field".to_string(),
         "Field will be filled with text".to_string())
    };

    ActionExplanation {
        action: format!("type {}", target_str),
        reason,
        expected_outcome: expected,
        goal_context: context.current_goal.clone(),
    }
}

fn explain_key(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let key = target.unwrap_or("key");
    let key_lower = key.to_lowercase();

    let (reason, expected) = if key_lower == "enter" || key_lower == "return" {
        ("Pressing Enter to submit or confirm".to_string(),
         "Form submission or action confirmation".to_string())
    } else if key_lower == "tab" {
        ("Pressing Tab to move to next field".to_string(),
         "Focus will move to next element".to_string())
    } else if key_lower == "escape" || key_lower == "esc" {
        ("Pressing Escape to cancel or close".to_string(),
         "Dialog or action will be cancelled".to_string())
    } else if key_lower.contains("arrow") || key_lower.contains("up") || key_lower.contains("down") || key_lower.contains("left") || key_lower.contains("right") {
        ("Using arrow keys for navigation".to_string(),
         "Selection or cursor will move".to_string())
    } else if key_lower.contains("cmd+") || key_lower.contains("ctrl+") {
        (format!("Executing keyboard shortcut: {}", key),
         "Shortcut action will be triggered".to_string())
    } else if key_lower == "backspace" || key_lower == "delete" {
        ("Pressing delete key".to_string(),
         "Selected content will be deleted".to_string())
    } else if key_lower == "space" {
        ("Pressing Space".to_string(),
         "Space character or button activation".to_string())
    } else {
        (format!("Pressing key: {}", key),
         "Key event will be dispatched".to_string())
    };

    ActionExplanation {
        action: format!("key {}", key),
        reason,
        expected_outcome: expected,
        goal_context: None,
    }
}

fn explain_hold_key(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let key = target.unwrap_or("key");

    ActionExplanation {
        action: format!("hold_key {}", key),
        reason: format!("Holding down {} key for extended action", key),
        expected_outcome: "Key-hold-dependent action will be triggered".to_string(),
        goal_context: None,
    }
}

fn explain_navigate(target: Option<&str>, context: &ExplanationContext) -> ActionExplanation {
    let url = target.unwrap_or("URL");

    let reason = if let Some(current) = &context.current_url {
        format!("Navigating from '{}' to new URL", current)
    } else {
        "Navigating to URL".to_string()
    };

    ActionExplanation {
        action: format!("navigate {}", url),
        reason,
        expected_outcome: "Page will load new URL".to_string(),
        goal_context: context.current_goal.clone(),
    }
}

fn explain_back(context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: "back".to_string(),
        reason: "Going back to previous page in browser history".to_string(),
        expected_outcome: "Previous page will be loaded".to_string(),
        goal_context: context.current_goal.clone(),
    }
}

fn explain_refresh(context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: "refresh".to_string(),
        reason: "Refreshing the current page to get updated content".to_string(),
        expected_outcome: "Page will reload with fresh data".to_string(),
        goal_context: context.current_goal.clone(),
    }
}

fn explain_hover(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let target_str = target.unwrap_or("element");

    let reason = if let Some(sel) = target {
        let sel_lower = sel.to_lowercase();
        if sel_lower.contains("menu") || sel_lower.contains("dropdown") {
            "Hovering to reveal dropdown menu".to_string()
        } else if sel_lower.contains("tooltip") {
            "Hovering to display tooltip".to_string()
        } else {
            format!("Hovering over '{}' to trigger hover state", sel)
        }
    } else {
        "Hovering over element".to_string()
    };

    ActionExplanation {
        action: format!("hover {}", target_str),
        reason,
        expected_outcome: "Hover effects will be triggered (menus, tooltips, styles)".to_string(),
        goal_context: None,
    }
}

fn explain_mouse_move(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("mouse_move {}", target.unwrap_or("coords")),
        reason: "Moving cursor to specific position".to_string(),
        expected_outcome: "Cursor will be at new position".to_string(),
        goal_context: None,
    }
}

fn explain_mouse_down(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("mouse_down {}", target.unwrap_or("left")),
        reason: "Pressing mouse button down (for drag operations)".to_string(),
        expected_outcome: "Mouse button will be held down".to_string(),
        goal_context: None,
    }
}

fn explain_mouse_up(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("mouse_up {}", target.unwrap_or("left")),
        reason: "Releasing mouse button (completing drag)".to_string(),
        expected_outcome: "Mouse button will be released".to_string(),
        goal_context: None,
    }
}

fn explain_drag(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("drag {}", target.unwrap_or("element")),
        reason: "Dragging element to new position".to_string(),
        expected_outcome: "Element will be moved or dropped".to_string(),
        goal_context: None,
    }
}

fn explain_wait(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let target_str = target.unwrap_or("condition");

    let (reason, expected) = if let Some(sel) = target {
        if sel.contains("gone") {
            ("Waiting for element to disappear (e.g., loading spinner)".to_string(),
             "Element will no longer be in DOM".to_string())
        } else if sel.contains("text") {
            ("Waiting for specific text to appear on page".to_string(),
             "Text will be present in page content".to_string())
        } else {
            (format!("Waiting for element '{}' to appear", sel),
             "Element will be present in DOM".to_string())
        }
    } else {
        ("Waiting for condition to be met".to_string(),
         "Condition will be satisfied".to_string())
    };

    ActionExplanation {
        action: format!("wait {}", target_str),
        reason,
        expected_outcome: expected,
        goal_context: None,
    }
}

fn explain_wait_duration(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let duration = target.unwrap_or("some time");

    ActionExplanation {
        action: format!("wait_duration {}", duration),
        reason: "Pausing execution for specified time".to_string(),
        expected_outcome: "Script will resume after delay".to_string(),
        goal_context: None,
    }
}

fn explain_select(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("select {}", target.unwrap_or("option")),
        reason: "Selecting an option from dropdown menu".to_string(),
        expected_outcome: "Dropdown value will be set".to_string(),
        goal_context: None,
    }
}

fn explain_upload(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("upload {}", target.unwrap_or("file")),
        reason: "Uploading file(s) to file input".to_string(),
        expected_outcome: "Files will be attached for submission".to_string(),
        goal_context: None,
    }
}

fn explain_dialog(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let action_type = target.unwrap_or("accept");

    ActionExplanation {
        action: format!("dialog {}", action_type),
        reason: format!("Responding to browser dialog ({})", action_type),
        expected_outcome: "Dialog will be closed".to_string(),
        goal_context: None,
    }
}

fn explain_screenshot(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let scope = if target.is_some() { "element" } else { "viewport" };

    ActionExplanation {
        action: format!("screenshot {}", target.unwrap_or("")),
        reason: format!("Capturing {} screenshot for verification", scope),
        expected_outcome: "Screenshot image will be saved".to_string(),
        goal_context: None,
    }
}

fn explain_screenshot_region(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("screenshot_region {}", target.unwrap_or("region")),
        reason: "Capturing specific region of the screen".to_string(),
        expected_outcome: "Cropped screenshot will be saved".to_string(),
        goal_context: None,
    }
}

fn explain_pdf(_context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: "pdf".to_string(),
        reason: "Exporting page as PDF document".to_string(),
        expected_outcome: "PDF file will be generated".to_string(),
        goal_context: None,
    }
}

fn explain_scroll(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    let target_str = target.unwrap_or("page");

    let reason = if let Some(sel) = target {
        if sel.contains(",") {
            // Coordinates
            format!("Scrolling by {} pixels", sel)
        } else {
            format!("Scrolling to element '{}'", sel)
        }
    } else {
        "Scrolling page".to_string()
    };

    ActionExplanation {
        action: format!("scroll {}", target_str),
        reason,
        expected_outcome: "Page scroll position will change".to_string(),
        goal_context: None,
    }
}

fn explain_resize(target: Option<&str>, _context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: format!("resize {}", target.unwrap_or("viewport")),
        reason: "Resizing browser viewport (for responsive testing)".to_string(),
        expected_outcome: "Viewport dimensions will change".to_string(),
        goal_context: None,
    }
}

fn explain_cursor_position(_context: &ExplanationContext) -> ActionExplanation {
    ActionExplanation {
        action: "cursor_position".to_string(),
        reason: "Getting current cursor coordinates".to_string(),
        expected_outcome: "Cursor position will be returned".to_string(),
        goal_context: None,
    }
}

/// Format explanation for human-readable output
pub fn format_explanation(explanation: &ActionExplanation) -> String {
    let mut output = String::new();

    output.push_str(&format!("Action: {}\n", explanation.action));
    output.push_str(&format!("Reason: {}\n", explanation.reason));
    output.push_str(&format!("Expected: {}\n", explanation.expected_outcome));

    if let Some(goal) = &explanation.goal_context {
        output.push_str(&format!("Goal: {}\n", goal));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_explanation() {
        let context = ExplanationContext::default();
        let explanation = explain_action("click", Some("button.submit"), &context);
        assert!(explanation.reason.contains("submit"));
    }

    #[test]
    fn test_navigate_explanation() {
        let mut context = ExplanationContext::default();
        context.current_url = Some("https://example.com".to_string());
        let explanation = explain_action("navigate", Some("https://newsite.com"), &context);
        assert!(explanation.reason.contains("example.com"));
    }

    #[test]
    fn test_type_password() {
        let context = ExplanationContext::default();
        let explanation = explain_action("type", Some("#password"), &context);
        assert!(explanation.reason.contains("password"));
    }
}
