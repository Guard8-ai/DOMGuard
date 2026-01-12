// DOMGuard Inspector - Background Service Worker

// Handle extension icon click (alternative to popup)
chrome.action.onClicked.addListener((tab) => {
  // Open popup by default
});

// Handle messages between content scripts and popup
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  // Forward messages to popup if needed
  if (message.type === 'ELEMENT_HOVERED' || message.type === 'ELEMENT_SELECTED') {
    // Broadcast to all extension pages
    chrome.runtime.sendMessage(message).catch(() => {
      // Popup might be closed
    });
  }
  return true;
});

// Handle keyboard shortcuts
chrome.commands?.onCommand?.addListener((command) => {
  if (command === 'toggle-inspect') {
    chrome.tabs.query({ active: true, currentWindow: true }, ([tab]) => {
      if (tab) {
        chrome.tabs.sendMessage(tab.id, { type: 'TOGGLE_INSPECTING' });
      }
    });
  }
});

// Context menu for quick actions
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: 'domguard-inspect',
    title: 'Inspect with DOMGuard',
    contexts: ['all']
  });

  chrome.contextMenus.create({
    id: 'domguard-copy-selector',
    title: 'Copy CSS Selector',
    contexts: ['all']
  });

  chrome.contextMenus.create({
    id: 'domguard-copy-command',
    title: 'Copy DOMGuard Command',
    contexts: ['all']
  });
});

// Handle context menu clicks
chrome.contextMenus.onClicked.addListener((info, tab) => {
  switch (info.menuItemId) {
    case 'domguard-inspect':
      chrome.tabs.sendMessage(tab.id, { type: 'START_INSPECTING' });
      break;

    case 'domguard-copy-selector':
    case 'domguard-copy-command':
      // Get element at click position
      chrome.tabs.sendMessage(tab.id, {
        type: 'GET_ELEMENT_AT_POINT',
        x: info.pageX || 0,
        y: info.pageY || 0,
        copyType: info.menuItemId === 'domguard-copy-selector' ? 'selector' : 'command'
      });
      break;
  }
});
