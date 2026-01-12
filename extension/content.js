// DOMGuard Inspector - Content Script
// Handles element highlighting and selector generation

(function() {
  'use strict';

  let isInspecting = false;
  let highlightOverlay = null;
  let selectedElement = null;
  let lastHoveredElement = null;

  // Create highlight overlay element
  function createOverlay() {
    if (highlightOverlay) return;

    highlightOverlay = document.createElement('div');
    highlightOverlay.id = 'domguard-highlight-overlay';
    highlightOverlay.style.cssText = `
      position: fixed;
      pointer-events: none;
      z-index: 2147483647;
      border: 2px solid #4F46E5;
      background: rgba(79, 70, 229, 0.1);
      transition: all 0.05s ease-out;
      display: none;
    `;
    document.body.appendChild(highlightOverlay);
  }

  // Generate optimal CSS selector for element
  function generateSelector(element) {
    if (!element || element === document.body || element === document.documentElement) {
      return 'body';
    }

    // Try ID first (most specific)
    if (element.id && /^[a-zA-Z][\w-]*$/.test(element.id)) {
      return `#${element.id}`;
    }

    // Try unique class combination
    if (element.classList.length > 0) {
      const classes = Array.from(element.classList)
        .filter(c => /^[a-zA-Z][\w-]*$/.test(c))
        .slice(0, 3);

      if (classes.length > 0) {
        const selector = element.tagName.toLowerCase() + '.' + classes.join('.');
        const matches = document.querySelectorAll(selector);
        if (matches.length === 1) {
          return selector;
        }
      }
    }

    // Try data attributes
    for (const attr of element.attributes) {
      if (attr.name.startsWith('data-') && attr.value) {
        const selector = `[${attr.name}="${attr.value}"]`;
        const matches = document.querySelectorAll(selector);
        if (matches.length === 1) {
          return selector;
        }
      }
    }

    // Try aria-label
    const ariaLabel = element.getAttribute('aria-label');
    if (ariaLabel) {
      const selector = `[aria-label="${ariaLabel}"]`;
      const matches = document.querySelectorAll(selector);
      if (matches.length === 1) {
        return selector;
      }
    }

    // Try text content for buttons/links
    if (['BUTTON', 'A'].includes(element.tagName)) {
      const text = element.textContent?.trim().slice(0, 30);
      if (text && !text.includes('\n')) {
        // This will need --text option in DOMGuard
        return `--text "${text}"`;
      }
    }

    // Fall back to nth-child path
    const path = [];
    let current = element;

    while (current && current !== document.body) {
      let selector = current.tagName.toLowerCase();

      if (current.id && /^[a-zA-Z][\w-]*$/.test(current.id)) {
        path.unshift(`#${current.id}`);
        break;
      }

      const parent = current.parentElement;
      if (parent) {
        const siblings = Array.from(parent.children).filter(
          c => c.tagName === current.tagName
        );
        if (siblings.length > 1) {
          const index = siblings.indexOf(current) + 1;
          selector += `:nth-of-type(${index})`;
        }
      }

      path.unshift(selector);
      current = parent;
    }

    return path.join(' > ');
  }

  // Generate DOMGuard CLI command
  function generateCommand(element, action = 'click') {
    const selector = generateSelector(element);

    if (selector.startsWith('--text')) {
      return `domguard interact ${action} ${selector}`;
    }

    return `domguard interact ${action} "${selector}"`;
  }

  // Get element info for popup
  function getElementInfo(element) {
    if (!element) return null;

    const rect = element.getBoundingClientRect();
    const styles = window.getComputedStyle(element);

    return {
      tagName: element.tagName.toLowerCase(),
      id: element.id || null,
      classes: Array.from(element.classList),
      selector: generateSelector(element),
      command: generateCommand(element),
      text: element.textContent?.trim().slice(0, 100) || null,
      rect: {
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height)
      },
      attributes: {
        href: element.getAttribute('href'),
        src: element.getAttribute('src'),
        type: element.getAttribute('type'),
        name: element.getAttribute('name'),
        placeholder: element.getAttribute('placeholder'),
        ariaLabel: element.getAttribute('aria-label')
      },
      isInteractive: isInteractiveElement(element)
    };
  }

  // Check if element is interactive
  function isInteractiveElement(element) {
    const interactiveTags = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'LABEL'];
    const hasClickHandler = element.onclick !== null;
    const hasRole = ['button', 'link', 'checkbox', 'radio', 'tab'].includes(
      element.getAttribute('role')
    );
    const isContentEditable = element.isContentEditable;

    return interactiveTags.includes(element.tagName) ||
           hasClickHandler ||
           hasRole ||
           isContentEditable;
  }

  // Update highlight position
  function updateHighlight(element) {
    if (!highlightOverlay || !element) {
      if (highlightOverlay) highlightOverlay.style.display = 'none';
      return;
    }

    const rect = element.getBoundingClientRect();
    highlightOverlay.style.display = 'block';
    highlightOverlay.style.left = rect.left + 'px';
    highlightOverlay.style.top = rect.top + 'px';
    highlightOverlay.style.width = rect.width + 'px';
    highlightOverlay.style.height = rect.height + 'px';
  }

  // Mouse move handler
  function handleMouseMove(e) {
    if (!isInspecting) return;

    const element = document.elementFromPoint(e.clientX, e.clientY);
    if (element && element !== highlightOverlay && element !== lastHoveredElement) {
      lastHoveredElement = element;
      updateHighlight(element);

      // Send element info to popup
      chrome.runtime.sendMessage({
        type: 'ELEMENT_HOVERED',
        data: getElementInfo(element)
      });
    }
  }

  // Click handler for selecting element
  function handleClick(e) {
    if (!isInspecting) return;

    e.preventDefault();
    e.stopPropagation();

    const element = document.elementFromPoint(e.clientX, e.clientY);
    if (element && element !== highlightOverlay) {
      selectedElement = element;
      const info = getElementInfo(element);

      // Copy selector to clipboard
      navigator.clipboard.writeText(info.command).then(() => {
        showCopiedNotification(element);
      });

      // Send to popup
      chrome.runtime.sendMessage({
        type: 'ELEMENT_SELECTED',
        data: info
      });
    }
  }

  // Show "Copied!" notification
  function showCopiedNotification(element) {
    const notification = document.createElement('div');
    notification.id = 'domguard-copied-notification';
    notification.textContent = 'Copied!';
    notification.style.cssText = `
      position: fixed;
      z-index: 2147483647;
      background: #10B981;
      color: white;
      padding: 8px 16px;
      border-radius: 6px;
      font-family: system-ui, sans-serif;
      font-size: 14px;
      font-weight: 500;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      pointer-events: none;
      animation: domguard-fadeout 1s ease-out forwards;
    `;

    const rect = element.getBoundingClientRect();
    notification.style.left = rect.left + rect.width / 2 + 'px';
    notification.style.top = rect.top - 40 + 'px';
    notification.style.transform = 'translateX(-50%)';

    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 1000);
  }

  // Start inspection mode
  function startInspecting() {
    if (isInspecting) return;

    isInspecting = true;
    createOverlay();
    document.addEventListener('mousemove', handleMouseMove, true);
    document.addEventListener('click', handleClick, true);
    document.body.style.cursor = 'crosshair';
  }

  // Stop inspection mode
  function stopInspecting() {
    if (!isInspecting) return;

    isInspecting = false;
    document.removeEventListener('mousemove', handleMouseMove, true);
    document.removeEventListener('click', handleClick, true);
    document.body.style.cursor = '';

    if (highlightOverlay) {
      highlightOverlay.style.display = 'none';
    }
    lastHoveredElement = null;
  }

  // Listen for messages from popup/background
  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.type) {
      case 'START_INSPECTING':
        startInspecting();
        sendResponse({ success: true });
        break;

      case 'STOP_INSPECTING':
        stopInspecting();
        sendResponse({ success: true });
        break;

      case 'GET_ELEMENT_AT_POINT':
        const element = document.elementFromPoint(message.x, message.y);
        sendResponse({ data: getElementInfo(element) });
        break;

      case 'IS_INSPECTING':
        sendResponse({ isInspecting });
        break;
    }
    return true;
  });

  // Cleanup on page unload
  window.addEventListener('beforeunload', () => {
    stopInspecting();
    if (highlightOverlay) {
      highlightOverlay.remove();
    }
  });

  // ESC key to stop inspecting
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && isInspecting) {
      stopInspecting();
      chrome.runtime.sendMessage({ type: 'INSPECTION_STOPPED' });
    }
  });

})();
