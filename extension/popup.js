// DOMGuard Inspector - Popup Script

let isInspecting = false;
let currentTabId = null;

// DOM elements
const inspectBtn = document.getElementById('inspectBtn');
const inspectBtnText = document.getElementById('inspectBtnText');
const statusDot = document.getElementById('statusDot');
const statusText = document.getElementById('statusText');
const elementInfo = document.getElementById('elementInfo');
const emptyState = document.getElementById('emptyState');
const tagName = document.getElementById('tagName');
const elementId = document.getElementById('elementId');
const elementClasses = document.getElementById('elementClasses');
const selectorBox = document.getElementById('selectorBox');
const commandBox = document.getElementById('commandBox');
const commandText = document.getElementById('commandText');
const copyBtn = document.getElementById('copyBtn');
const attrsSection = document.getElementById('attrsSection');
const attrsList = document.getElementById('attrsList');

// Initialize
async function init() {
  // Get current tab
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  currentTabId = tab.id;

  // Check if already inspecting
  try {
    const response = await chrome.tabs.sendMessage(currentTabId, { type: 'IS_INSPECTING' });
    if (response && response.isInspecting) {
      setInspectingState(true);
    }
  } catch (e) {
    // Content script not loaded yet
  }
}

// Toggle inspection mode
async function toggleInspect() {
  isInspecting = !isInspecting;
  setInspectingState(isInspecting);

  try {
    await chrome.tabs.sendMessage(currentTabId, {
      type: isInspecting ? 'START_INSPECTING' : 'STOP_INSPECTING'
    });
  } catch (e) {
    // Inject content script if not present
    await chrome.scripting.executeScript({
      target: { tabId: currentTabId },
      files: ['content.js']
    });
    await chrome.scripting.insertCSS({
      target: { tabId: currentTabId },
      files: ['styles.css']
    });

    // Retry
    await chrome.tabs.sendMessage(currentTabId, {
      type: isInspecting ? 'START_INSPECTING' : 'STOP_INSPECTING'
    });
  }
}

// Update UI for inspecting state
function setInspectingState(inspecting) {
  isInspecting = inspecting;

  if (inspecting) {
    inspectBtn.classList.add('active');
    inspectBtnText.textContent = 'Stop Inspecting';
    statusDot.classList.add('active');
    statusText.textContent = 'Inspecting...';
  } else {
    inspectBtn.classList.remove('active');
    inspectBtnText.textContent = 'Start Inspecting';
    statusDot.classList.remove('active');
    statusText.textContent = 'Ready';
  }
}

// Display element info
function displayElementInfo(data) {
  if (!data) {
    elementInfo.style.display = 'none';
    emptyState.style.display = 'block';
    return;
  }

  elementInfo.style.display = 'block';
  emptyState.style.display = 'none';

  // Tag name
  tagName.textContent = `<${data.tagName}>`;

  // ID
  if (data.id) {
    elementId.textContent = `#${data.id}`;
    elementId.style.display = 'inline';
  } else {
    elementId.style.display = 'none';
  }

  // Classes
  if (data.classes && data.classes.length > 0) {
    elementClasses.textContent = '.' + data.classes.join(' .');
    elementClasses.style.display = 'block';
  } else {
    elementClasses.style.display = 'none';
  }

  // Selector
  selectorBox.textContent = data.selector;

  // Command
  commandText.textContent = data.command;

  // Attributes
  const attrs = Object.entries(data.attributes || {})
    .filter(([_, v]) => v !== null && v !== undefined);

  if (attrs.length > 0) {
    attrsSection.style.display = 'block';
    attrsList.innerHTML = attrs.map(([name, value]) => {
      const displayValue = value.length > 30 ? value.slice(0, 30) + '...' : value;
      return `<span class="attr-tag"><span class="attr-name">${name}</span>=<span class="attr-value">"${displayValue}"</span></span>`;
    }).join('');
  } else {
    attrsSection.style.display = 'none';
  }
}

// Copy to clipboard
async function copyToClipboard(text) {
  try {
    await navigator.clipboard.writeText(text);
    copyBtn.textContent = 'Copied!';
    copyBtn.classList.add('copied');
    setTimeout(() => {
      copyBtn.textContent = 'Copy';
      copyBtn.classList.remove('copied');
    }, 1500);
  } catch (e) {
    console.error('Failed to copy:', e);
  }
}

// Event listeners
inspectBtn.addEventListener('click', toggleInspect);

selectorBox.addEventListener('click', () => {
  copyToClipboard(selectorBox.textContent);
});

copyBtn.addEventListener('click', () => {
  copyToClipboard(commandText.textContent);
});

// Listen for messages from content script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.tab?.id !== currentTabId) return;

  switch (message.type) {
    case 'ELEMENT_HOVERED':
    case 'ELEMENT_SELECTED':
      displayElementInfo(message.data);
      break;

    case 'INSPECTION_STOPPED':
      setInspectingState(false);
      break;
  }
});

// Initialize on load
init();
