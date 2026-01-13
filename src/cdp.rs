//! Chrome DevTools Protocol connection handling
//!
//! Manages WebSocket connections to Chrome

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use chromiumoxide::browser::Browser;
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::cdp::browser_protocol::log::{self, EventEntryAdded};
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotParams, PrintToPdfParams};
use chromiumoxide::cdp::js_protocol::runtime::{self, EventConsoleApiCalled, EventExceptionThrown};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::Config;

/// Tab information for listing browser tabs
#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub id: String,
    pub url: String,
    pub title: String,
}

/// CDP connection manager
pub struct CdpConnection {
    config: Config,
    browser: Option<Arc<Mutex<Browser>>>,
}

impl CdpConnection {
    /// Create a new CDP connection manager
    pub fn new(config: Config) -> Self {
        Self {
            config,
            browser: None,
        }
    }

    /// Check if connection is to localhost (security check)
    pub fn validate_security(&self) -> Result<()> {
        if !self.config.is_localhost() {
            return Err(anyhow!(
                "Security: Non-localhost connections are blocked by default.\n\
                 Remote Chrome connections have security risks (no encryption, no auth).\n\
                 See SECURITY.md for safe remote connection setup.\n\
                 Use --allow-remote flag to proceed anyway."
            ));
        }
        Ok(())
    }

    /// Connect to Chrome DevTools, launching Chrome if needed
    pub async fn connect(&mut self) -> Result<()> {
        self.validate_security()?;

        let ws_url = self.config.ws_url();

        // Try to connect to existing Chrome first
        match Browser::connect(&ws_url).await {
            Ok((browser, mut handler)) => {
                // Spawn handler to process CDP events
                tokio::spawn(async move {
                    while let Some(_event) = handler.next().await {
                        // Event handling happens automatically
                    }
                });
                self.browser = Some(Arc::new(Mutex::new(browser)));
                return Ok(());
            }
            Err(_) => {
                // Chrome not running, launch it
                self.launch_chrome().await?;
            }
        }

        Ok(())
    }

    /// Launch Chrome with remote debugging enabled
    async fn launch_chrome(&mut self) -> Result<()> {
        let chrome_path = Self::find_chrome()?;
        let port = self.config.chrome.port;

        // Create user data dir in temp to avoid profile conflicts
        let user_data_dir = std::env::temp_dir().join("domguard-chrome-profile");
        std::fs::create_dir_all(&user_data_dir)?;

        // Launch Chrome as a detached process
        let mut command = std::process::Command::new(&chrome_path);
        command
            .arg(format!("--remote-debugging-port={}", port))
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        command.spawn().context("Failed to launch Chrome")?;

        // Wait for Chrome to start and become available
        let ws_url = self.config.ws_url();
        for i in 0..30 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if let Ok((browser, mut handler)) = Browser::connect(&ws_url).await {
                tokio::spawn(async move { while let Some(_event) = handler.next().await {} });
                self.browser = Some(Arc::new(Mutex::new(browser)));
                return Ok(());
            }
            if i == 29 {
                return Err(anyhow!(
                    "Chrome started but failed to connect via WebSocket"
                ));
            }
        }

        Ok(())
    }

    /// Find Chrome executable on the system
    fn find_chrome() -> Result<std::path::PathBuf> {
        let candidates = [
            "google-chrome-stable",
            "google-chrome",
            "chromium-browser",
            "chromium",
            "chrome",
            // macOS
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            // Windows
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];

        for candidate in candidates {
            if let Ok(path) = which::which(candidate) {
                return Ok(path);
            }
            let path = std::path::PathBuf::from(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow!(
            "Chrome not found. Please install Chrome or Chromium.\n\
             Searched: google-chrome-stable, google-chrome, chromium-browser, chromium"
        ))
    }

    /// Get the current page or first available page
    pub async fn get_page(&self) -> Result<Page> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Chrome"))?;

        let mut browser = browser.lock().await;

        // Fetch existing targets from the browser - this registers them internally as pages
        // By default, only targets launched after connection are tracked, so we need this
        // See: https://docs.rs/chromiumoxide/latest/chromiumoxide/browser/struct.Browser.html#method.fetch_targets
        let targets = browser.fetch_targets().await?;

        // Small delay as docs recommend - pages may not be immediately ready
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Find the best target: prefer real URLs over about:blank, skip chrome:// pages
        let mut best_target = None;
        let mut fallback_target = None;

        for target in &targets {
            if target.r#type != "page" {
                continue;
            }
            // Skip internal Chrome pages and extensions
            if target.url.starts_with("chrome://")
                || target.url.starts_with("chrome-extension://")
                || target.url.starts_with("devtools://")
            {
                continue;
            }
            // Prefer pages with real URLs over about:blank
            if target.url != "about:blank" && !target.url.is_empty() {
                best_target = Some(target.target_id.clone());
                break;
            }
            // Keep about:blank as fallback
            if fallback_target.is_none() {
                fallback_target = Some(target.target_id.clone());
            }
        }

        // Use best target or fallback
        if let Some(target_id) = best_target.or(fallback_target) {
            return browser
                .get_page(target_id)
                .await
                .context("Failed to attach to page");
        }

        // No existing pages - create one
        browser
            .new_page("about:blank")
            .await
            .context("Failed to create page")
    }

    /// Get or create a page
    pub async fn get_or_create_page(&self) -> Result<Page> {
        self.get_page().await
    }

    /// List all open tabs (targets)
    pub async fn list_tabs(&self) -> Result<Vec<TabInfo>> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Chrome"))?;
        let mut browser = browser.lock().await;

        let targets = browser.fetch_targets().await?;

        let mut tabs = Vec::new();
        for target in &targets {
            if target.r#type != "page" {
                continue;
            }
            // Skip internal Chrome pages
            if target.url.starts_with("chrome://")
                || target.url.starts_with("chrome-extension://")
                || target.url.starts_with("devtools://")
            {
                continue;
            }
            tabs.push(TabInfo {
                id: target.target_id.as_ref().to_string(),
                url: target.url.clone(),
                title: target.title.clone(),
            });
        }
        Ok(tabs)
    }

    /// Create a new tab
    pub async fn new_tab(&self, url: Option<&str>) -> Result<String> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Chrome"))?;
        let browser = browser.lock().await;

        let page = browser
            .new_page(url.unwrap_or("about:blank"))
            .await
            .context("Failed to create new tab")?;

        let target_id = page.target_id().as_ref().to_string();
        Ok(target_id)
    }

    /// Switch to a specific tab by ID
    pub async fn switch_tab(&self, target_id: &str) -> Result<()> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Chrome"))?;
        let browser = browser.lock().await;

        use chromiumoxide::cdp::browser_protocol::target::{ActivateTargetParams, TargetId};
        let tid = TargetId::from(target_id.to_string());

        // Use CDP command directly to activate the target
        browser
            .execute(ActivateTargetParams::new(tid))
            .await
            .context("Failed to switch to tab")?;

        Ok(())
    }

    /// Close a tab by ID
    pub async fn close_tab(&self, target_id: &str) -> Result<()> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Chrome"))?;
        let browser = browser.lock().await;

        use chromiumoxide::cdp::browser_protocol::target::{CloseTargetParams, TargetId};
        let tid = TargetId::from(target_id.to_string());

        // Use CDP command directly to close the target
        browser
            .execute(CloseTargetParams::new(tid))
            .await
            .context("Failed to close tab")?;

        Ok(())
    }

    /// Query selector all via JavaScript
    pub async fn query_selector_all(&self, selector: &str) -> Result<Vec<i64>> {
        let result = self
            .evaluate(&format!(
                "Array.from(document.querySelectorAll('{}')).length",
                selector.replace('\'', "\\'")
            ))
            .await?;

        let count = result.as_i64().unwrap_or(0);
        Ok((0..count).collect())
    }

    /// Capture screenshot
    pub async fn screenshot(&self, full_page: bool) -> Result<Vec<u8>> {
        let page = self.get_page().await?;

        let params = if full_page {
            CaptureScreenshotParams::builder()
                .capture_beyond_viewport(true)
                .build()
        } else {
            CaptureScreenshotParams::default()
        };

        let data = page
            .execute(params)
            .await
            .context("Failed to capture screenshot")?;

        base64::engine::general_purpose::STANDARD
            .decode(&data.data)
            .context("Failed to decode screenshot data")
    }

    /// Execute JavaScript and return result
    pub async fn evaluate(&self, expression: &str) -> Result<serde_json::Value> {
        let page = self.get_page().await?;

        let result = page
            .evaluate(expression)
            .await
            .context("Failed to evaluate JavaScript")?;

        Ok(result.value().cloned().unwrap_or(serde_json::Value::Null))
    }

    /// Navigate to URL with extended timeout
    pub async fn navigate(&self, url: &str) -> Result<()> {
        // Ensure we have a page to work with
        let _ = self.get_or_create_page().await?;

        // Use JavaScript navigation with our own timeout handling
        // This avoids chromiumoxide's default timeout which can be too short
        let escaped_url = url.replace('\\', "\\\\").replace('\'', "\\'");

        // Start navigation
        self.evaluate(&format!("window.location.href = '{}'", escaped_url))
            .await?;

        // Wait for navigation to complete (check document.readyState)
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Navigation timeout after 30s"));
            }

            // Check if we've navigated and page is ready
            let ready_state = self.evaluate("document.readyState").await?;
            let current_url = self.evaluate("window.location.href").await?;

            if let (Some(state), Some(cur)) = (ready_state.as_str(), current_url.as_str()) {
                // Check if we're on the target URL (or redirected) and page is loaded
                if (cur.starts_with(url) || cur != "about:blank")
                    && (state == "complete" || state == "interactive")
                {
                    return Ok(());
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get current URL
    pub async fn current_url(&self) -> Result<String> {
        let result = self.evaluate("window.location.href").await?;
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Failed to get current URL"))
    }

    /// Click at coordinates using JavaScript
    pub async fn click_at(&self, x: f64, y: f64) -> Result<()> {
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.elementFromPoint({}, {});
                if (el) el.click();
            }})()
            "#,
            x, y
        ))
        .await?;
        Ok(())
    }

    /// Click element by selector using JavaScript (more reliable than CDP)
    /// Supports nth parameter to select nth matching element (0-indexed, -1 for last)
    pub async fn click(&self, selector: &str, nth: i32) -> Result<()> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const els = document.querySelectorAll('{}');
                if (els.length === 0) return {{ found: false, count: 0 }};
                const idx = {} < 0 ? els.length + {} : {};
                if (idx < 0 || idx >= els.length) return {{ found: false, count: els.length, index: idx }};
                const el = els[idx];
                el.scrollIntoView({{ block: 'center' }});
                el.click();
                return {{ found: true }};
            }})()
            "#,
                escaped, nth, nth, nth
            ))
            .await?;

        if let Some(obj) = result.as_object() {
            if obj.get("found").and_then(|v| v.as_bool()) == Some(true) {
                return Ok(());
            }
            let count = obj.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
            if count == 0 {
                return Err(anyhow!("No element matches selector \"{}\"", selector));
            }
            let index = obj
                .get("index")
                .and_then(|v| v.as_i64())
                .unwrap_or(i64::from(nth));
            return Err(anyhow!(
                "Index {} out of bounds, found {} element(s) matching \"{}\"",
                index,
                count,
                selector
            ));
        }
        Err(anyhow!("No element matches selector \"{}\"", selector))
    }

    /// Click element by visible text content
    /// Useful for dynamic dropdowns and elements without stable CSS selectors
    pub async fn click_by_text(&self, text: &str, nth: i32) -> Result<()> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const searchText = '{}';
                const nth = {};

                // Find all elements containing the text
                const matches = [];
                const walker = document.createTreeWalker(
                    document.body,
                    NodeFilter.SHOW_ELEMENT,
                    null
                );

                while (walker.nextNode()) {{
                    const node = walker.currentNode;
                    // Skip invisible elements
                    const style = window.getComputedStyle(node);
                    if (style.display === 'none' || style.visibility === 'hidden') continue;

                    // Check direct text content (text nodes that are direct children)
                    const directText = Array.from(node.childNodes)
                        .filter(n => n.nodeType === Node.TEXT_NODE)
                        .map(n => n.textContent.trim())
                        .join(' ');

                    if (directText.includes(searchText)) {{
                        matches.push(node);
                        continue;
                    }}

                    // For leaf-ish elements (few children), check full textContent
                    if (node.children.length <= 2 && node.textContent.includes(searchText)) {{
                        // Make sure it's not a container with many text nodes
                        const text = node.textContent.trim();
                        if (text.length < 200) {{ // Reasonable limit for clickable elements
                            matches.push(node);
                        }}
                    }}
                }}

                // Remove duplicates (parent/child pairs) - keep the most specific (deepest) element
                const filtered = matches.filter((el, i) => {{
                    return !matches.some((other, j) => i !== j && el.contains(other) && el !== other);
                }});

                if (filtered.length === 0) return {{ found: false, count: 0 }};

                const idx = nth < 0 ? filtered.length + nth : nth;
                if (idx < 0 || idx >= filtered.length) {{
                    return {{ found: false, count: filtered.length, index: idx }};
                }}

                const el = filtered[idx];
                el.scrollIntoView({{ block: 'center' }});
                el.click();
                return {{ found: true, count: filtered.length, tag: el.tagName }};
            }})()
            "#,
                escaped, nth
            ))
            .await?;

        if let Some(obj) = result.as_object() {
            if obj.get("found").and_then(|v| v.as_bool()) == Some(true) {
                return Ok(());
            }
            let count = obj.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
            if count == 0 {
                return Err(anyhow!("No element found containing text \"{}\"", text));
            }
            let index = obj
                .get("index")
                .and_then(|v| v.as_i64())
                .unwrap_or(i64::from(nth));
            return Err(anyhow!(
                "Index {} out of bounds, found {} element(s) containing \"{}\"",
                index,
                count,
                text
            ));
        }
        Err(anyhow!("No element found containing text \"{}\"", text))
    }

    /// Type text into element using JavaScript
    /// Uses native value setter to work with React controlled inputs
    pub async fn type_into(&self, selector: &str, text: &str) -> Result<()> {
        let escaped_sel = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_text = text
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                el.focus();
                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                    // Use native setter to bypass React's value interception
                    const proto = el.tagName === 'INPUT'
                        ? HTMLInputElement.prototype
                        : HTMLTextAreaElement.prototype;
                    const nativeSetter = Object.getOwnPropertyDescriptor(proto, 'value').set;
                    nativeSetter.call(el, '{}');
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }} else if (el.contentEditable === 'true') {{
                    el.textContent = '{}';
                }}
                return true;
            }})()
            "#,
                escaped_sel, escaped_text, escaped_text
            ))
            .await?;

        if result.as_bool() != Some(true) {
            return Err(anyhow!("No element matches selector \"{}\"", selector));
        }
        Ok(())
    }

    /// Type text into currently focused element using JavaScript
    /// Uses native value setter to work with React controlled inputs
    pub async fn type_focused(&self, text: &str) -> Result<()> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.activeElement;
                if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.contentEditable === 'true')) {{
                    if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                        // Use native setter to bypass React's value interception
                        const proto = el.tagName === 'INPUT'
                            ? HTMLInputElement.prototype
                            : HTMLTextAreaElement.prototype;
                        const nativeSetter = Object.getOwnPropertyDescriptor(proto, 'value').set;
                        const currentValue = el.value || '';
                        nativeSetter.call(el, currentValue + '{}');
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    }} else {{
                        document.execCommand('insertText', false, '{}');
                    }}
                }}
            }})()
            "#, escaped, escaped
        )).await?;
        Ok(())
    }

    /// Press a key using JavaScript
    pub async fn press_key(&self, key: &str) -> Result<()> {
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.activeElement || document.body;
                const event = new KeyboardEvent('keydown', {{
                    key: '{}',
                    bubbles: true,
                    cancelable: true
                }});
                el.dispatchEvent(event);
            }})()
            "#,
            key
        ))
        .await?;
        Ok(())
    }

    /// Hover over element using JavaScript
    pub async fn hover(&self, selector: &str) -> Result<()> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                el.scrollIntoView({{ block: 'center' }});
                el.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true }}));
                el.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
                return true;
            }})()
            "#,
                escaped
            ))
            .await?;

        if result.as_bool() != Some(true) {
            return Err(anyhow!("No element matches selector \"{}\"", selector));
        }
        Ok(())
    }

    /// Scroll by pixels
    pub async fn scroll_by(&self, x: i64, y: i64) -> Result<()> {
        self.evaluate(&format!("window.scrollBy({}, {})", x, y))
            .await?;
        Ok(())
    }

    /// Scroll to element using JavaScript
    pub async fn scroll_to_element(&self, selector: &str) -> Result<()> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                return true;
            }})()
            "#,
                escaped
            ))
            .await?;

        if result.as_bool() != Some(true) {
            return Err(anyhow!("No element matches selector \"{}\"", selector));
        }
        Ok(())
    }

    /// Wait for element to appear using JavaScript polling
    pub async fn wait_for(&self, selector: &str, timeout_ms: u64) -> Result<()> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "Timeout waiting for \"{}\" ({}ms)",
                    selector,
                    timeout_ms
                ));
            }

            let result = self
                .evaluate(&format!("document.querySelector('{}') !== null", escaped))
                .await?;

            if result.as_bool() == Some(true) {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Wait for element to be gone
    pub async fn wait_for_gone(&self, selector: &str, timeout_ms: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "Timeout waiting for \"{}\" to disappear ({}ms)",
                    selector,
                    timeout_ms
                ));
            }

            let result = self.query_selector_all(selector).await;
            match result {
                Ok(nodes) if nodes.is_empty() => return Ok(()),
                Err(_) => return Ok(()),
                _ => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
    }

    /// Go back in history
    pub async fn go_back(&self) -> Result<()> {
        self.evaluate("window.history.back()").await?;
        Ok(())
    }

    /// Refresh page
    pub async fn refresh(&self) -> Result<()> {
        let page = self.get_page().await?;
        page.reload().await.context("Failed to refresh")?;
        Ok(())
    }

    /// Get page title
    pub async fn get_title(&self) -> Result<String> {
        let result = self.evaluate("document.title").await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    /// Get cookies via JavaScript
    pub async fn get_cookies(&self) -> Result<serde_json::Value> {
        self.evaluate(
            r#"
            document.cookie.split('; ').filter(c => c).map(c => {
                const [name, ...rest] = c.split('=');
                return { name, value: rest.join('=') };
            })
        "#,
        )
        .await
    }

    /// Get localStorage
    pub async fn get_local_storage(&self) -> Result<serde_json::Value> {
        self.evaluate("JSON.stringify(Object.entries(localStorage))")
            .await
    }

    /// Get sessionStorage
    pub async fn get_session_storage(&self) -> Result<serde_json::Value> {
        self.evaluate("JSON.stringify(Object.entries(sessionStorage))")
            .await
    }

    /// Capture console messages via CDP event listeners
    /// This captures Log entries, Console API calls, and Runtime exceptions
    pub async fn capture_console_messages(&self, timeout_ms: u64) -> Result<Vec<ConsoleEntry>> {
        let page = self.get_page().await?;
        let mut entries = Vec::new();

        // IMPORTANT: Set up event listeners BEFORE enabling domains
        // When domains are enabled, they send previously collected entries
        let mut log_events = page.event_listener::<EventEntryAdded>().await?;
        let mut console_events = page.event_listener::<EventConsoleApiCalled>().await?;
        let mut exception_events = page.event_listener::<EventExceptionThrown>().await?;

        // Enable log domain - sends collected entries via entryAdded
        page.execute(log::EnableParams::default())
            .await
            .context("Failed to enable log domain")?;

        // Enable runtime for console API and exceptions
        page.execute(runtime::EnableParams::default())
            .await
            .context("Failed to enable runtime domain")?;

        // Collect events for the specified timeout
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            tokio::select! {
                _ = tokio::time::sleep(remaining) => break,

                Some(event) = log_events.next() => {
                    entries.push(ConsoleEntry {
                        level: format!("{:?}", event.entry.level).to_lowercase(),
                        source: format!("{:?}", event.entry.source).to_lowercase(),
                        text: event.entry.text.clone(),
                        url: event.entry.url.clone(),
                        line: event.entry.line_number.map(|n| n as u32),
                        stack_trace: event.entry.stack_trace.as_ref().map(|st| {
                            st.call_frames.iter()
                                .take(5)
                                .map(|f| format!("    at {} ({}:{}:{})",
                                    f.function_name.as_str(),
                                    f.url, f.line_number, f.column_number))
                                .collect::<Vec<_>>()
                                .join("\n")
                        }),
                        timestamp: *event.entry.timestamp.inner(),
                    });
                }

                Some(event) = console_events.next() => {
                    let text = event.args.iter()
                        .filter_map(|arg| arg.value.as_ref().map(|v| v.to_string()))
                        .collect::<Vec<_>>()
                        .join(" ");

                    entries.push(ConsoleEntry {
                        level: format!("{:?}", event.r#type).to_lowercase(),
                        source: "console".to_string(),
                        text,
                        url: None,
                        line: None,
                        stack_trace: event.stack_trace.as_ref().map(|st| {
                            st.call_frames.iter()
                                .take(5)
                                .map(|f| format!("    at {} ({}:{}:{})",
                                    f.function_name.as_str(),
                                    f.url, f.line_number, f.column_number))
                                .collect::<Vec<_>>()
                                .join("\n")
                        }),
                        timestamp: *event.timestamp.inner(),
                    });
                }

                Some(event) = exception_events.next() => {
                    let details = &event.exception_details;
                    entries.push(ConsoleEntry {
                        level: "error".to_string(),
                        source: "exception".to_string(),
                        text: details.exception.as_ref()
                            .and_then(|e| e.description.clone())
                            .unwrap_or_else(|| details.text.clone()),
                        url: details.url.clone(),
                        line: Some(details.line_number as u32),
                        stack_trace: details.stack_trace.as_ref().map(|st| {
                            st.call_frames.iter()
                                .take(5)
                                .map(|f| format!("    at {} ({}:{}:{})",
                                    f.function_name.as_str(),
                                    f.url, f.line_number, f.column_number))
                                .collect::<Vec<_>>()
                                .join("\n")
                        }),
                        timestamp: *event.timestamp.inner(),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Get accessibility tree via JavaScript
    /// Uses JavaScript to traverse the accessibility tree since chromiumoxide
    /// doesn't expose the Accessibility domain directly
    pub async fn get_accessibility_tree(
        &self,
        selector: Option<&str>,
    ) -> Result<serde_json::Value> {
        let js = if let Some(sel) = selector {
            format!(
                r#"
                (function() {{
                    const root = document.querySelector('{}');
                    if (!root) return null;

                    function getAriaNode(el, depth) {{
                        if (depth > 10) return null;
                        if (!el || el.nodeType !== 1) return null;

                        // Get computed accessibility properties
                        const role = el.getAttribute('role') ||
                            el.tagName.toLowerCase() === 'button' ? 'button' :
                            el.tagName.toLowerCase() === 'a' ? 'link' :
                            el.tagName.toLowerCase() === 'input' ? (el.type === 'checkbox' ? 'checkbox' : el.type === 'radio' ? 'radio' : 'textbox') :
                            el.tagName.toLowerCase() === 'select' ? 'combobox' :
                            el.tagName.toLowerCase() === 'textarea' ? 'textbox' :
                            el.tagName.toLowerCase() === 'img' ? 'img' :
                            el.tagName.toLowerCase() === 'nav' ? 'navigation' :
                            el.tagName.toLowerCase() === 'main' ? 'main' :
                            el.tagName.toLowerCase() === 'header' ? 'banner' :
                            el.tagName.toLowerCase() === 'footer' ? 'contentinfo' :
                            el.tagName.toLowerCase() === 'aside' ? 'complementary' :
                            el.tagName.toLowerCase() === 'article' ? 'article' :
                            el.tagName.toLowerCase() === 'section' ? 'region' :
                            el.tagName.toLowerCase() === 'form' ? 'form' :
                            el.tagName.toLowerCase() === 'ul' || el.tagName.toLowerCase() === 'ol' ? 'list' :
                            el.tagName.toLowerCase() === 'li' ? 'listitem' :
                            el.tagName.toLowerCase() === 'table' ? 'table' :
                            el.tagName.toLowerCase() === 'tr' ? 'row' :
                            el.tagName.toLowerCase() === 'th' ? 'columnheader' :
                            el.tagName.toLowerCase() === 'td' ? 'cell' :
                            el.tagName.toLowerCase().match(/^h[1-6]$/) ? 'heading' :
                            'generic';

                        // Get accessible name
                        const name = el.getAttribute('aria-label') ||
                            el.getAttribute('aria-labelledby') && document.getElementById(el.getAttribute('aria-labelledby'))?.textContent ||
                            el.getAttribute('alt') ||
                            el.getAttribute('title') ||
                            (el.tagName.toLowerCase() === 'input' && el.labels?.[0]?.textContent) ||
                            (el.tagName.toLowerCase().match(/^h[1-6]$/) && el.textContent?.trim()) ||
                            (role === 'button' || role === 'link' ? el.textContent?.trim().substring(0, 100) : null);

                        // Get value
                        const value = el.value !== undefined && el.value !== '' ? el.value :
                            el.getAttribute('aria-valuenow') ||
                            null;

                        // Get states
                        const states = [];
                        if (el.disabled) states.push('disabled');
                        if (el.getAttribute('aria-disabled') === 'true') states.push('disabled');
                        if (el.getAttribute('aria-expanded') === 'true') states.push('expanded');
                        if (el.getAttribute('aria-expanded') === 'false') states.push('collapsed');
                        if (el.getAttribute('aria-selected') === 'true') states.push('selected');
                        if (el.getAttribute('aria-checked') === 'true') states.push('checked');
                        if (el.getAttribute('aria-checked') === 'false') states.push('unchecked');
                        if (el.checked) states.push('checked');
                        if (el.getAttribute('aria-hidden') === 'true') states.push('hidden');
                        if (el.getAttribute('aria-required') === 'true') states.push('required');
                        if (el.required) states.push('required');
                        if (el.getAttribute('aria-invalid') === 'true') states.push('invalid');
                        if (document.activeElement === el) states.push('focused');

                        // Get description
                        const description = el.getAttribute('aria-describedby') &&
                            document.getElementById(el.getAttribute('aria-describedby'))?.textContent || null;

                        // Skip non-semantic elements without meaningful content
                        const isSemanticRole = role !== 'generic' || name || states.length > 0;

                        // Get children
                        const children = [];
                        for (const child of el.children) {{
                            if (child.getAttribute('aria-hidden') === 'true') continue;
                            const childNode = getAriaNode(child, depth + 1);
                            if (childNode) {{
                                if (childNode.role === 'generic' && !childNode.name && childNode.states.length === 0) {{
                                    // Flatten generic nodes without meaning
                                    children.push(...(childNode.children || []));
                                }} else {{
                                    children.push(childNode);
                                }}
                            }}
                        }}

                        // Return null for empty generic nodes
                        if (role === 'generic' && !name && states.length === 0 && children.length === 0) {{
                            return null;
                        }}

                        return {{
                            role,
                            name: name || null,
                            value: value || null,
                            description: description || null,
                            states,
                            children
                        }};
                    }}

                    return getAriaNode(root, 0);
                }})()
            "#,
                sel
            )
        } else {
            r#"
                (function() {
                    function getAriaNode(el, depth) {
                        if (depth > 10) return null;
                        if (!el || el.nodeType !== 1) return null;

                        // Get computed accessibility properties
                        const role = el.getAttribute('role') ||
                            el.tagName.toLowerCase() === 'button' ? 'button' :
                            el.tagName.toLowerCase() === 'a' ? 'link' :
                            el.tagName.toLowerCase() === 'input' ? (el.type === 'checkbox' ? 'checkbox' : el.type === 'radio' ? 'radio' : 'textbox') :
                            el.tagName.toLowerCase() === 'select' ? 'combobox' :
                            el.tagName.toLowerCase() === 'textarea' ? 'textbox' :
                            el.tagName.toLowerCase() === 'img' ? 'img' :
                            el.tagName.toLowerCase() === 'nav' ? 'navigation' :
                            el.tagName.toLowerCase() === 'main' ? 'main' :
                            el.tagName.toLowerCase() === 'header' ? 'banner' :
                            el.tagName.toLowerCase() === 'footer' ? 'contentinfo' :
                            el.tagName.toLowerCase() === 'aside' ? 'complementary' :
                            el.tagName.toLowerCase() === 'article' ? 'article' :
                            el.tagName.toLowerCase() === 'section' ? 'region' :
                            el.tagName.toLowerCase() === 'form' ? 'form' :
                            el.tagName.toLowerCase() === 'ul' || el.tagName.toLowerCase() === 'ol' ? 'list' :
                            el.tagName.toLowerCase() === 'li' ? 'listitem' :
                            el.tagName.toLowerCase() === 'table' ? 'table' :
                            el.tagName.toLowerCase() === 'tr' ? 'row' :
                            el.tagName.toLowerCase() === 'th' ? 'columnheader' :
                            el.tagName.toLowerCase() === 'td' ? 'cell' :
                            el.tagName.toLowerCase().match(/^h[1-6]$/) ? 'heading' :
                            'generic';

                        // Get accessible name
                        const name = el.getAttribute('aria-label') ||
                            el.getAttribute('aria-labelledby') && document.getElementById(el.getAttribute('aria-labelledby'))?.textContent ||
                            el.getAttribute('alt') ||
                            el.getAttribute('title') ||
                            (el.tagName.toLowerCase() === 'input' && el.labels?.[0]?.textContent) ||
                            (el.tagName.toLowerCase().match(/^h[1-6]$/) && el.textContent?.trim()) ||
                            (role === 'button' || role === 'link' ? el.textContent?.trim().substring(0, 100) : null);

                        // Get value
                        const value = el.value !== undefined && el.value !== '' ? el.value :
                            el.getAttribute('aria-valuenow') ||
                            null;

                        // Get states
                        const states = [];
                        if (el.disabled) states.push('disabled');
                        if (el.getAttribute('aria-disabled') === 'true') states.push('disabled');
                        if (el.getAttribute('aria-expanded') === 'true') states.push('expanded');
                        if (el.getAttribute('aria-expanded') === 'false') states.push('collapsed');
                        if (el.getAttribute('aria-selected') === 'true') states.push('selected');
                        if (el.getAttribute('aria-checked') === 'true') states.push('checked');
                        if (el.getAttribute('aria-checked') === 'false') states.push('unchecked');
                        if (el.checked) states.push('checked');
                        if (el.getAttribute('aria-hidden') === 'true') states.push('hidden');
                        if (el.getAttribute('aria-required') === 'true') states.push('required');
                        if (el.required) states.push('required');
                        if (el.getAttribute('aria-invalid') === 'true') states.push('invalid');
                        if (document.activeElement === el) states.push('focused');

                        // Get description
                        const description = el.getAttribute('aria-describedby') &&
                            document.getElementById(el.getAttribute('aria-describedby'))?.textContent || null;

                        // Get children
                        const children = [];
                        for (const child of el.children) {
                            if (child.getAttribute('aria-hidden') === 'true') continue;
                            const childNode = getAriaNode(child, depth + 1);
                            if (childNode) {
                                if (childNode.role === 'generic' && !childNode.name && childNode.states.length === 0) {
                                    // Flatten generic nodes without meaning
                                    children.push(...(childNode.children || []));
                                } else {
                                    children.push(childNode);
                                }
                            }
                        }

                        // Return null for empty generic nodes
                        if (role === 'generic' && !name && states.length === 0 && children.length === 0) {
                            return null;
                        }

                        return {
                            role,
                            name: name || null,
                            value: value || null,
                            description: description || null,
                            states,
                            children
                        };
                    }

                    return getAriaNode(document.documentElement, 0);
                })()
            "#.to_string()
        };

        self.evaluate(&js).await
    }

    /// Wait for text to appear on page
    pub async fn wait_for_text(&self, text: &str, timeout_ms: u64) -> Result<()> {
        let escaped_text = text.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            r#"
            (function() {{
                return document.body.innerText.includes('{}');
            }})()
        "#,
            escaped_text
        );

        let timeout = Duration::from_millis(timeout_ms);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "Timeout waiting for text \"{}\" ({}ms)",
                    text,
                    timeout_ms
                ));
            }

            let result = self.evaluate(&js).await?;
            if result.as_bool() == Some(true) {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Wait for text to disappear from page
    pub async fn wait_for_text_gone(&self, text: &str, timeout_ms: u64) -> Result<()> {
        let escaped_text = text.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            r#"
            (function() {{
                return !document.body.innerText.includes('{}');
            }})()
        "#,
            escaped_text
        );

        let timeout = Duration::from_millis(timeout_ms);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "Timeout waiting for text \"{}\" to disappear ({}ms)",
                    text,
                    timeout_ms
                ));
            }

            let result = self.evaluate(&js).await?;
            if result.as_bool() == Some(true) {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get element center coordinates
    pub async fn get_element_center(&self, selector: &str) -> Result<(f64, f64)> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return null;
                const rect = el.getBoundingClientRect();
                return {{
                    x: rect.left + rect.width / 2,
                    y: rect.top + rect.height / 2
                }};
            }})()
        "#,
            escaped
        );

        let result = self.evaluate(&js).await?;
        if result.is_null() {
            return Err(anyhow!("Element not found: {}", selector));
        }

        let x = result
            .get("x")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Invalid coordinates"))?;
        let y = result
            .get("y")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Invalid coordinates"))?;
        Ok((x, y))
    }

    /// Drag from one point to another
    pub async fn drag(&self, from_x: f64, from_y: f64, to_x: f64, to_y: f64) -> Result<()> {
        let js = format!(
            r#"
            (async function() {{
                // Create and dispatch mouse events for drag operation
                const fromEl = document.elementFromPoint({}, {});
                const toEl = document.elementFromPoint({}, {});

                if (!fromEl) return false;

                // Mouse down at start
                fromEl.dispatchEvent(new MouseEvent('mousedown', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    button: 0
                }}));

                // Mouse move to end
                const moveEvent = new MouseEvent('mousemove', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    button: 0
                }});
                document.dispatchEvent(moveEvent);

                // Mouse up at end
                const upEl = toEl || document.elementFromPoint({}, {});
                if (upEl) {{
                    upEl.dispatchEvent(new MouseEvent('mouseup', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        button: 0
                    }}));
                }}

                // Also dispatch dragstart/dragend for native drag-drop
                fromEl.dispatchEvent(new DragEvent('dragstart', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {}
                }}));

                if (upEl) {{
                    upEl.dispatchEvent(new DragEvent('drop', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {}
                    }}));
                    upEl.dispatchEvent(new DragEvent('dragend', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {}
                    }}));
                }}

                return true;
            }})()
        "#,
            from_x,
            from_y,
            to_x,
            to_y,
            from_x,
            from_y,
            to_x,
            to_y,
            to_x,
            to_y,
            to_x,
            to_y,
            from_x,
            from_y,
            to_x,
            to_y,
            to_x,
            to_y
        );

        self.evaluate(&js).await?;
        Ok(())
    }

    /// Select option in dropdown
    pub async fn select_option(
        &self,
        selector: &str,
        value: &str,
        by_label: bool,
        by_index: bool,
    ) -> Result<()> {
        let escaped_sel = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_val = value.replace('\\', "\\\\").replace('\'', "\\'");

        let js = if by_index {
            format!(
                r#"
                (function() {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    const idx = parseInt('{}');
                    if (idx >= 0 && idx < select.options.length) {{
                        select.selectedIndex = idx;
                        select.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                    return false;
                }})()
            "#,
                escaped_sel, escaped_val
            )
        } else if by_label {
            format!(
                r#"
                (function() {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    for (let i = 0; i < select.options.length; i++) {{
                        if (select.options[i].text === '{}') {{
                            select.selectedIndex = i;
                            select.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            return true;
                        }}
                    }}
                    return false;
                }})()
            "#,
                escaped_sel, escaped_val
            )
        } else {
            format!(
                r#"
                (function() {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    select.value = '{}';
                    select.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }})()
            "#,
                escaped_sel, escaped_val
            )
        };

        let result = self.evaluate(&js).await?;
        if result.as_bool() != Some(true) {
            return Err(anyhow!("Failed to select option in \"{}\"", selector));
        }
        Ok(())
    }

    /// Upload files to file input using CDP DOM.setFileInputFiles
    pub async fn upload_files(&self, selector: &str, files: &[std::path::PathBuf]) -> Result<()> {
        let page = self.get_page().await?;
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");

        // Get the file input element and verify it's a file input, then get its object ID
        let js = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el || el.tagName !== 'INPUT' || el.type !== 'file') return null;
                return el;
            }})()
        "#,
            escaped
        );

        // Use evaluate to get the RemoteObjectId
        let result = page
            .evaluate(js)
            .await
            .context("Failed to find file input element")?;

        let object_id =
            result.object().object_id.clone().ok_or_else(|| {
                anyhow!("Element \"{}\" is not a file input or not found", selector)
            })?;

        // Convert file paths to strings
        let file_paths: Vec<String> = files
            .iter()
            .map(|f| f.to_string_lossy().to_string())
            .collect();

        // Use CDP DOM.setFileInputFiles with the object ID
        let params = SetFileInputFilesParams::builder()
            .files(file_paths)
            .object_id(object_id.clone())
            .build()
            .map_err(|e| anyhow!("Failed to build SetFileInputFiles params: {}", e))?;

        page.execute(params)
            .await
            .context("Failed to set file input files")?;

        // Dispatch change event to notify any listeners
        let trigger_js = format!(
            r#"
            (function() {{
                const input = document.querySelector('{}');
                if (input) {{
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                return true;
            }})()
        "#,
            escaped
        );

        self.evaluate(&trigger_js).await?;
        Ok(())
    }

    /// Handle JavaScript dialog (alert, confirm, prompt)
    pub async fn handle_dialog(&self, accept: bool, text: Option<&str>) -> Result<()> {
        // Override window methods to auto-handle dialogs
        let text_value = text.map_or_else(|| "''".to_string(), |t| format!("'{}'", t.replace('\'', "\\'")));
        let js = format!(
            r#"
            (function() {{
                // Store original methods
                window.__origAlert = window.__origAlert || window.alert;
                window.__origConfirm = window.__origConfirm || window.confirm;
                window.__origPrompt = window.__origPrompt || window.prompt;

                // Override with auto-response
                window.alert = function(msg) {{ window.__lastDialog = {{ type: 'alert', message: msg }}; }};
                window.confirm = function(msg) {{ window.__lastDialog = {{ type: 'confirm', message: msg }}; return {}; }};
                window.prompt = function(msg, def) {{ window.__lastDialog = {{ type: 'prompt', message: msg }}; return {} ? {} : null; }};

                return true;
            }})()
        "#,
            accept, accept, text_value
        );

        self.evaluate(&js).await?;
        Ok(())
    }

    /// Resize viewport
    pub async fn resize_viewport(&self, width: u32, height: u32) -> Result<()> {
        let js = format!(
            r#"
            (function() {{
                // This only works for the visual viewport within the page
                // Actual viewport resize requires CDP Emulation.setDeviceMetricsOverride
                window.resizeTo({}, {});
                return true;
            }})()
        "#,
            width, height
        );

        self.evaluate(&js).await?;
        Ok(())
    }

    /// Print page to PDF using CDP Page.printToPDF
    /// Note: PDF generation only works in Chrome headless mode
    pub async fn print_to_pdf(&self, landscape: bool) -> Result<Vec<u8>> {
        let page = self.get_page().await?;

        // Build PDF params with landscape option
        let params = PrintToPdfParams::builder()
            .landscape(landscape)
            .print_background(true)
            .build();

        // Use page.pdf() which handles the CDP command and base64 decoding
        let pdf_data = page.pdf(params).await.context(
            "Failed to generate PDF. Note: PDF export only works in Chrome headless mode",
        )?;

        Ok(pdf_data)
    }

    // =========================================================================
    // Anthropic Computer Use CDP methods
    // =========================================================================

    /// Move mouse cursor to coordinates without clicking
    pub async fn mouse_move(&self, x: f64, y: f64) -> Result<()> {
        let js = format!(
            r#"
            (function() {{
                // Store cursor position for later retrieval
                window.__domguardCursorX = {};
                window.__domguardCursorY = {};

                // Dispatch mousemove event to element under cursor
                const el = document.elementFromPoint({}, {});
                if (el) {{
                    el.dispatchEvent(new MouseEvent('mousemove', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        view: window
                    }}));
                }}
                return true;
            }})()
        "#,
            x, y, x, y, x, y
        );

        self.evaluate(&js).await?;
        Ok(())
    }

    /// Get current cursor position (tracked via mouse_move)
    pub async fn cursor_position(&self) -> Result<(f64, f64)> {
        let result = self
            .evaluate(
                r#"
            (function() {
                return {
                    x: window.__domguardCursorX || 0,
                    y: window.__domguardCursorY || 0
                };
            })()
        "#,
            )
            .await?;

        let x = result.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = result.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok((x, y))
    }

    /// Hold a key for specified duration
    pub async fn hold_key(&self, key: &str, duration_ms: u64) -> Result<()> {
        let escaped_key = key.replace('\\', "\\\\").replace('\'', "\\'");

        // Key down
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.activeElement || document.body;
                el.dispatchEvent(new KeyboardEvent('keydown', {{
                    key: '{}',
                    bubbles: true,
                    cancelable: true,
                    repeat: false
                }}));
            }})()
        "#,
            escaped_key
        ))
        .await?;

        // Hold for duration
        tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;

        // Key up
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.activeElement || document.body;
                el.dispatchEvent(new KeyboardEvent('keyup', {{
                    key: '{}',
                    bubbles: true,
                    cancelable: true
                }}));
            }})()
        "#,
            escaped_key
        ))
        .await?;

        Ok(())
    }

    /// Triple-click element by selector (select paragraph/block)
    pub async fn triple_click(&self, selector: &str) -> Result<()> {
        let escaped = selector.replace('\\', "\\\\").replace('\'', "\\'");
        let result = self
            .evaluate(&format!(
                r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                el.scrollIntoView({{ block: 'center' }});

                const rect = el.getBoundingClientRect();
                const x = rect.left + rect.width / 2;
                const y = rect.top + rect.height / 2;

                // Dispatch 3 click events with increasing detail
                for (let i = 1; i <= 3; i++) {{
                    el.dispatchEvent(new MouseEvent('click', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: x,
                        clientY: y,
                        detail: i,
                        view: window
                    }}));
                }}

                // Also select text if it's a text container
                const selection = window.getSelection();
                const range = document.createRange();
                range.selectNodeContents(el);
                selection.removeAllRanges();
                selection.addRange(range);

                return true;
            }})()
        "#,
                escaped
            ))
            .await?;

        if result.as_bool() != Some(true) {
            return Err(anyhow!("No element matches selector \"{}\"", selector));
        }
        Ok(())
    }

    /// Triple-click at coordinates
    pub async fn triple_click_at(&self, x: f64, y: f64) -> Result<()> {
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.elementFromPoint({}, {});
                if (!el) return false;

                // Dispatch 3 click events with increasing detail
                for (let i = 1; i <= 3; i++) {{
                    el.dispatchEvent(new MouseEvent('click', {{
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        detail: i,
                        view: window
                    }}));
                }}

                // Try to select the text at that point
                const selection = window.getSelection();
                if (selection.rangeCount > 0) {{
                    selection.modify('extend', 'backward', 'paragraphboundary');
                    selection.modify('extend', 'forward', 'paragraphboundary');
                }}

                return true;
            }})()
        "#,
            x, y, x, y
        ))
        .await?;

        Ok(())
    }

    /// Press mouse button down (without releasing)
    pub async fn mouse_down(&self, button: &str) -> Result<()> {
        let button_num = match button.to_lowercase().as_str() {
            "left" | "main" => 0,
            "middle" | "auxiliary" => 1,
            "right" | "secondary" => 2,
            _ => 0,
        };

        let (x, y) = self.cursor_position().await?;
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.elementFromPoint({}, {}) || document.body;
                el.dispatchEvent(new MouseEvent('mousedown', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    button: {},
                    buttons: {},
                    view: window
                }}));
                return true;
            }})()
        "#,
            x,
            y,
            x,
            y,
            button_num,
            1 << button_num
        ))
        .await?;

        Ok(())
    }

    /// Release mouse button
    pub async fn mouse_up(&self, button: &str) -> Result<()> {
        let button_num = match button.to_lowercase().as_str() {
            "left" | "main" => 0,
            "middle" | "auxiliary" => 1,
            "right" | "secondary" => 2,
            _ => 0,
        };

        let (x, y) = self.cursor_position().await?;
        self.evaluate(&format!(
            r#"
            (function() {{
                const el = document.elementFromPoint({}, {}) || document.body;
                el.dispatchEvent(new MouseEvent('mouseup', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    button: {},
                    view: window
                }}));
                return true;
            }})()
        "#,
            x, y, x, y, button_num
        ))
        .await?;

        Ok(())
    }

    /// Capture screenshot of specific region using CDP clip parameter
    pub async fn screenshot_region(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        let page = self.get_page().await?;

        // Use CDP with clip viewport
        use chromiumoxide::cdp::browser_protocol::page::Viewport;

        let clip = Viewport {
            x: f64::from(x),
            y: f64::from(y),
            width: f64::from(width),
            height: f64::from(height),
            scale: 1.0,
        };

        let params = CaptureScreenshotParams::builder().clip(clip).build();

        let data = page
            .execute(params)
            .await
            .context("Failed to capture screenshot region")?;

        base64::engine::general_purpose::STANDARD
            .decode(&data.data)
            .context("Failed to decode screenshot data")
    }

    // =========================================================================
    // Chrome DevTools MCP CDP methods
    // =========================================================================

    /// Get performance metrics including Core Web Vitals
    pub async fn get_performance_metrics(&self) -> Result<serde_json::Value> {
        use chromiumoxide::cdp::browser_protocol::performance::{EnableParams, GetMetricsParams};

        let page = self.get_page().await?;

        // Enable performance domain
        page.execute(EnableParams::default())
            .await
            .context("Failed to enable Performance domain")?;

        // Get CDP metrics
        let cdp_result = page
            .execute(GetMetricsParams {})
            .await
            .context("Failed to get performance metrics")?;

        let cdp_metrics: Vec<serde_json::Value> = cdp_result
            .metrics
            .iter()
            .map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "value": m.value
                })
            })
            .collect();

        // Get Web Vitals from JavaScript
        let web_vitals = self
            .evaluate(
                r#"
            (function() {
                const nav = performance.getEntriesByType('navigation')[0] || {};
                const paint = performance.getEntriesByType('paint');
                const fcp = paint.find(e => e.name === 'first-contentful-paint');
                const lcp = performance.getEntriesByType('largest-contentful-paint').pop();

                return {
                    ttfb: nav.responseStart || null,
                    fcp: fcp ? fcp.startTime : null,
                    lcp: lcp ? lcp.startTime : null,
                    domContentLoaded: nav.domContentLoadedEventEnd || null,
                    load: nav.loadEventEnd || null,
                    cls: window.__domguardCLS || null
                };
            })()
        "#,
            )
            .await
            .unwrap_or(serde_json::json!({}));

        Ok(serde_json::json!({
            "web_vitals": web_vitals,
            "cdp_metrics": cdp_metrics
        }))
    }

    /// Get full DOM snapshot as HTML string
    pub async fn get_full_dom_snapshot(&self) -> Result<String> {
        let result = self
            .evaluate(
                r#"
            (function() {
                return document.documentElement.outerHTML;
            })()
        "#,
            )
            .await?;

        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Failed to get DOM snapshot"))
    }

    /// Set CPU throttling rate (1 = no throttle, 4 = 4x slowdown, etc.)
    pub async fn set_cpu_throttling(&self, rate: f64) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::emulation::SetCpuThrottlingRateParams;

        let page = self.get_page().await?;
        let params = SetCpuThrottlingRateParams::builder()
            .rate(rate)
            .build()
            .map_err(|e| anyhow!("Failed to build CPU throttling params: {}", e))?;

        page.execute(params)
            .await
            .context("Failed to set CPU throttling")?;

        Ok(())
    }

    /// Disable CPU throttling
    pub async fn disable_cpu_throttling(&self) -> Result<()> {
        self.set_cpu_throttling(1.0).await
    }

    /// Set network throttling conditions
    pub async fn set_network_throttling(
        &self,
        download_throughput_kbps: f64,
        upload_throughput_kbps: f64,
        latency_ms: f64,
        offline: bool,
    ) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams;

        let page = self.get_page().await?;

        // Convert Kbps to bytes per second (CDP expects bytes/s)
        let download_bps = (download_throughput_kbps * 1024.0 / 8.0) as i64;
        let upload_bps = (upload_throughput_kbps * 1024.0 / 8.0) as i64;

        let params = EmulateNetworkConditionsParams::builder()
            .offline(offline)
            .latency(latency_ms)
            .download_throughput(download_bps as f64)
            .upload_throughput(upload_bps as f64)
            .build()
            .map_err(|e| anyhow!("Failed to build network throttling params: {}", e))?;

        page.execute(params)
            .await
            .context("Failed to set network throttling")?;

        Ok(())
    }

    /// Disable network throttling
    pub async fn disable_network_throttling(&self) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams;

        let page = self.get_page().await?;

        let params = EmulateNetworkConditionsParams::builder()
            .offline(false)
            .latency(0.0)
            .download_throughput(-1.0) // -1 disables throttling
            .upload_throughput(-1.0)
            .build()
            .map_err(|e| anyhow!("Failed to build network params: {}", e))?;

        page.execute(params)
            .await
            .context("Failed to disable network throttling")?;

        Ok(())
    }

    /// Get detailed network request info using Resource Timing API
    pub async fn get_network_details(&self, filter: Option<&str>) -> Result<serde_json::Value> {
        let filter_str = filter.map(|f| f.replace('\'', "\\'")).unwrap_or_default();

        let js = format!(
            r#"
            (function() {{
                const entries = performance.getEntriesByType('resource');
                const filter = '{}';

                return entries
                    .filter(e => !filter || e.name.includes(filter))
                    .map(e => {{
                        // Calculate timing breakdown
                        const timing = {{
                            dns: e.domainLookupEnd - e.domainLookupStart,
                            connect: e.connectEnd - e.connectStart,
                            ssl: e.secureConnectionStart > 0 ? e.connectEnd - e.secureConnectionStart : 0,
                            ttfb: e.responseStart - e.requestStart,
                            download: e.responseEnd - e.responseStart,
                            total: e.duration
                        }};

                        return {{
                            url: e.name,
                            method: 'GET', // Resource Timing doesn't expose method
                            type: e.initiatorType,
                            status: null, // Not available in Resource Timing
                            size_bytes: e.transferSize || null,
                            encoded_size: e.encodedBodySize || null,
                            decoded_size: e.decodedBodySize || null,
                            timing: timing,
                            cache_hit: e.transferSize === 0 && e.decodedBodySize > 0,
                            next_hop_protocol: e.nextHopProtocol || null
                        }};
                    }});
            }})()
        "#,
            filter_str
        );

        self.evaluate(&js).await
    }
}

impl CdpConnection {
    /// Highlight an element on the page with a colored overlay
    pub async fn highlight_element(
        &self,
        selector: &str,
        color: &str,
        duration_ms: u64,
    ) -> Result<()> {
        // Parse color or use default
        let (r, g, b, a) = parse_color(color).unwrap_or((255, 0, 0, 128));

        // First, get the element info to highlight
        let highlight_js = format!(
            r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return {{ error: 'Element not found' }};

                const rect = el.getBoundingClientRect();

                // Create highlight overlay
                const overlay = document.createElement('div');
                overlay.id = '__domguard_highlight__';
                overlay.style.cssText = `
                    position: fixed;
                    top: ${{rect.top}}px;
                    left: ${{rect.left}}px;
                    width: ${{rect.width}}px;
                    height: ${{rect.height}}px;
                    background-color: rgba({}, {}, {}, {});
                    border: 2px solid rgb({}, {}, {});
                    pointer-events: none;
                    z-index: 2147483647;
                    transition: opacity 0.2s;
                    box-shadow: 0 0 10px rgba({}, {}, {}, 0.5);
                `;

                // Remove any existing highlight
                const existing = document.getElementById('__domguard_highlight__');
                if (existing) existing.remove();

                document.body.appendChild(overlay);

                return {{
                    success: true,
                    selector: '{}',
                    rect: {{ top: rect.top, left: rect.left, width: rect.width, height: rect.height }}
                }};
            }})()
            "#,
            selector.replace('\'', "\\'"),
            r,
            g,
            b,
            f64::from(a) / 255.0,
            r,
            g,
            b,
            r,
            g,
            b,
            selector.replace('\'', "\\'")
        );

        let result = self.evaluate(&highlight_js).await?;

        if let Some(error) = result.get("error") {
            anyhow::bail!("{}", error.as_str().unwrap_or("Unknown error"));
        }

        // If duration is specified, remove after delay
        if duration_ms > 0 {
            tokio::time::sleep(Duration::from_millis(duration_ms)).await;
            self.clear_highlight().await?;
        }

        Ok(())
    }

    /// Clear any active highlight
    pub async fn clear_highlight(&self) -> Result<()> {
        let clear_js = r#"
            (() => {
                const overlay = document.getElementById('__domguard_highlight__');
                if (overlay) {
                    overlay.style.opacity = '0';
                    setTimeout(() => overlay.remove(), 200);
                    return { removed: true };
                }
                return { removed: false };
            })()
        "#;

        let _ = self.evaluate(clear_js).await?;
        Ok(())
    }

    /// Highlight multiple elements with labels
    pub async fn highlight_elements(
        &self,
        selector: &str,
        color: &str,
    ) -> Result<serde_json::Value> {
        let (r, g, b, a) = parse_color(color).unwrap_or((255, 0, 0, 128));

        let highlight_js = format!(
            r#"
            (() => {{
                const els = document.querySelectorAll('{}');
                if (els.length === 0) return {{ error: 'No elements found' }};

                // Remove any existing highlights
                document.querySelectorAll('.__domguard_highlight__').forEach(el => el.remove());

                const highlighted = [];
                els.forEach((el, index) => {{
                    const rect = el.getBoundingClientRect();
                    if (rect.width === 0 || rect.height === 0) return;

                    // Create highlight overlay
                    const overlay = document.createElement('div');
                    overlay.className = '__domguard_highlight__';
                    overlay.style.cssText = `
                        position: fixed;
                        top: ${{rect.top}}px;
                        left: ${{rect.left}}px;
                        width: ${{rect.width}}px;
                        height: ${{rect.height}}px;
                        background-color: rgba({}, {}, {}, {});
                        border: 2px solid rgb({}, {}, {});
                        pointer-events: none;
                        z-index: 2147483647;
                    `;

                    // Add label
                    const label = document.createElement('span');
                    label.textContent = index + 1;
                    label.style.cssText = `
                        position: absolute;
                        top: -20px;
                        left: 0;
                        background: rgb({}, {}, {});
                        color: white;
                        font-size: 12px;
                        padding: 2px 6px;
                        border-radius: 3px;
                        font-family: sans-serif;
                    `;
                    overlay.appendChild(label);

                    document.body.appendChild(overlay);
                    highlighted.push({{
                        index: index + 1,
                        rect: {{ top: rect.top, left: rect.left, width: rect.width, height: rect.height }},
                        tagName: el.tagName.toLowerCase(),
                        id: el.id || null,
                        classes: Array.from(el.classList).join(' ') || null
                    }});
                }});

                return {{
                    success: true,
                    count: highlighted.length,
                    elements: highlighted
                }};
            }})()
            "#,
            selector.replace('\'', "\\'"),
            r,
            g,
            b,
            f64::from(a) / 255.0,
            r,
            g,
            b,
            r,
            g,
            b
        );

        let result = self.evaluate(&highlight_js).await?;

        if let Some(error) = result.get("error") {
            anyhow::bail!("{}", error.as_str().unwrap_or("Unknown error"));
        }

        Ok(result)
    }

    /// Clear all highlights
    pub async fn clear_all_highlights(&self) -> Result<()> {
        let clear_js = r#"
            (() => {
                const overlays = document.querySelectorAll('.__domguard_highlight__, #__domguard_highlight__');
                overlays.forEach(el => el.remove());
                return { removed: overlays.length };
            })()
        "#;

        let _ = self.evaluate(clear_js).await?;
        Ok(())
    }
}

/// Parse color string into RGBA values
fn parse_color(color: &str) -> Option<(u8, u8, u8, u8)> {
    let color = color.trim().to_lowercase();

    // Named colors
    match color.as_str() {
        "red" => return Some((255, 0, 0, 128)),
        "green" => return Some((0, 255, 0, 128)),
        "blue" => return Some((0, 0, 255, 128)),
        "yellow" => return Some((255, 255, 0, 128)),
        "orange" => return Some((255, 165, 0, 128)),
        "purple" => return Some((128, 0, 128, 128)),
        "pink" => return Some((255, 192, 203, 128)),
        "cyan" => return Some((0, 255, 255, 128)),
        _ => {}
    }

    // Hex color (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
    if let Some(hex) = color.strip_prefix('#') {
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                return Some((r, g, b, 128));
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                return Some((r, g, b, a));
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some((r, g, b, 128));
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some((r, g, b, a));
            }
            _ => {}
        }
    }

    None
}

/// Console entry captured via CDP
#[derive(Debug, Clone, Serialize)]
pub struct ConsoleEntry {
    pub level: String,
    pub source: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    pub timestamp: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_validation_localhost() {
        let config = Config::default();
        let conn = CdpConnection::new(config);
        assert!(conn.validate_security().is_ok());
    }

    #[test]
    fn test_security_validation_remote() {
        let mut config = Config::default();
        config.chrome.host = "192.168.1.100".to_string();
        let conn = CdpConnection::new(config);
        assert!(conn.validate_security().is_err());
    }
}
