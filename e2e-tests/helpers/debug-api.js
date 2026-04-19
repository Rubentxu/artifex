/**
 * E2E Debug API Helpers for WebDriverIO
 * Wraps window.__ARTIFEX_DEBUG__ for use in specs
 */

// Tauri custom protocol base URL
const TAURI_BASE = 'https://tauri.localhost';

/**
 * Wait for the debug API to be available on the window
 * @param {WebDriverIO.Browser} browser
 * @param {number} [timeout=5000]
 */
export async function waitForAppReady(browser, timeout = 15000) {
  await browser.waitUntil(
    () => browser.execute(() => !!window.__ARTIFEX_DEBUG__),
    { timeout, timeoutMsg: 'Debug API not available' }
  );
}

/**
 * Navigate to a path within the Tauri app using client-side routing.
 * This avoids a full page reload which would destroy the debug harness.
 * Falls back to browser.url() if client-side nav fails.
 * @param {WebDriverIO.Browser} browser
 * @param {string} path - The path to navigate to (e.g., '/assets')
 */
export async function navigateTo(browser, path) {
  // Try SvelteKit client-side navigation first (preserves app state)
  const navigated = await browser.execute((targetPath) => {
    // Find sidebar nav link matching the path
    const links = document.querySelectorAll('a[href]');
    for (const link of links) {
      if (link.getAttribute('href') === targetPath || link.getAttribute('href')?.endsWith(targetPath)) {
        link.click();
        return true;
      }
    }
    // Fallback: try pushing to history and dispatching popstate
    if (window.history && window.history.pushState) {
      window.history.pushState({}, '', targetPath);
      window.dispatchEvent(new PopStateEvent('popstate', { state: {} }));
      return true;
    }
    return false;
  }, path);

  if (!navigated) {
    // Last resort: full page navigation (will lose debug harness)
    const url = `${TAURI_BASE}${path}`;
    await browser.url(url);
  }

  // Wait for SvelteKit client-side router to settle
  await browser.pause(500);
}

/**
 * Get a reference to the debug API object
 * @param {WebDriverIO.Browser} browser
 */
export async function getDebugAPI(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__);
}

/**
 * Get all store snapshots
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetStores(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getStores());
}

/**
 * Get a single store value by name
 * @param {WebDriverIO.Browser} browser
 * @param {string} name
 */
export async function debugGetStore(browser, name) {
  return browser.execute(
    (storeName) => window.__ARTIFEX_DEBUG__.getStore(storeName),
    name
  );
}

/**
 * Get current route info
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetRoute(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getRoute());
}

/**
 * Get a single element's debug info
 * @param {WebDriverIO.Browser} browser
 * @param {string} selector CSS selector
 */
export async function debugGetElement(browser, selector) {
  return browser.execute(
    (sel) => window.__ARTIFEX_DEBUG__.getElement(sel),
    selector
  );
}

/**
 * Get multiple elements' debug info
 * @param {WebDriverIO.Browser} browser
 * @param {string} pattern CSS selector pattern
 */
export async function debugGetElements(browser, pattern) {
  return browser.execute(
    (pat) => window.__ARTIFEX_DEBUG__.getElements(pat),
    pattern
  );
}

/**
 * Get navigation links
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetNavigation(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getNavigation());
}

/**
 * Get open dialogs
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetDialogs(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getDialogs());
}

/**
 * Get buttons, optionally filtered by text pattern
 * @param {WebDriverIO.Browser} browser
 * @param {string} [textPattern]
 */
export async function debugGetButtons(browser, textPattern) {
  return browser.execute(
    (pattern) => window.__ARTIFEX_DEBUG__.getButtons(pattern),
    textPattern
  );
}

/**
 * Get text content of an element
 * @param {WebDriverIO.Browser} browser
 * @param {string} selector CSS selector
 */
export async function debugGetTextContent(browser, selector) {
  return browser.execute(
    (sel) => window.__ARTIFEX_DEBUG__.getTextContent(sel),
    selector
  );
}

/**
 * Check if text exists anywhere on the page
 * @param {WebDriverIO.Browser} browser
 * @param {string} text
 */
export async function debugHasText(browser, text) {
  return browser.execute(
    (t) => window.__ARTIFEX_DEBUG__.hasText(t),
    text
  );
}

/**
 * Get current viewport info
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetViewport(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getViewport());
}

/**
 * Get active job status
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetActiveJobs(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getActiveJobs());
}

/**
 * Get current Tauri context
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetTauriContext(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getTauriContext());
}

/**
 * Wait for a condition to be true
 * @param {WebDriverIO.Browser} browser
 * @param {string} predicateCode JavaScript code that returns a boolean
 * @param {number} timeout timeout in ms
 */
export async function debugWaitForCondition(browser, predicateCode, timeout) {
  return browser.execute(
    (code, ms) => window.__ARTIFEX_DEBUG__.waitForCondition(code, ms),
    predicateCode,
    timeout
  );
}

/**
 * Take a full page snapshot
 * @param {WebDriverIO.Browser} browser
 */
export async function debugSnapshot(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.snapshot());
}

// ----------------------------------------------------------------------------
// Mock Layer Helpers
// These wrap window.__ARTIFEX_DEBUG__.mock* methods when available
// ----------------------------------------------------------------------------

/**
 * Enable mock mode (switch from TauriBackend to MockBackend)
 * @param {WebDriverIO.Browser} browser
 */
export async function debugEnableMock(browser) {
  const result = await browser.execute(() => {
    if (!window.__ARTIFEX_DEBUG__.mock) {
      return { success: false, error: 'Mock API not available (DEV only)' };
    }
    window.__ARTIFEX_DEBUG__.mock.enableMock();
    return { success: true };
  });
  if (!result.success) {
    throw new Error(`Failed to enable mock: ${result.error}`);
  }
}

/**
 * Disable mock mode (restore TauriBackend)
 * @param {WebDriverIO.Browser} browser
 */
export async function debugDisableMock(browser) {
  return browser.execute(() => {
    if (window.__ARTIFEX_DEBUG__.mock) {
      window.__ARTIFEX_DEBUG__.mock.disableMock();
    }
  });
}

/**
 * Check if mock mode is enabled
 * @param {WebDriverIO.Browser} browser
 */
export async function debugIsMockMode(browser) {
  return browser.execute(() => {
    return window.__ARTIFEX_DEBUG__.mock?.isMockMode() ?? false;
  });
}

/**
 * Set mock data for a key
 * @param {WebDriverIO.Browser} browser
 * @param {string} key - 'projects', 'assets', 'user', 'providers', 'profiles', 'rules', 'templates'
 * @param {unknown} data
 */
export async function debugSetMockData(browser, key, data) {
  return browser.execute(
    (k, d) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        window.__ARTIFEX_DEBUG__.mock.setMockData(k, d);
      }
    },
    key,
    data
  );
}

/**
 * Set mock tier
 * @param {WebDriverIO.Browser} browser
 * @param {'free' | 'pro'} tier
 */
export async function debugSetMockTier(browser, tier) {
  return browser.execute(
    (t) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        window.__ARTIFEX_DEBUG__.mock.setMockTier(t);
      }
    },
    tier
  );
}

/**
 * Set mock job result
 * @param {WebDriverIO.Browser} browser
 * @param {'success' | 'error'} result
 */
export async function debugSetMockJobResult(browser, result) {
  return browser.execute(
    (r) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        window.__ARTIFEX_DEBUG__.mock.setMockJobResult(r);
      }
    },
    result
  );
}

/**
 * Get all mock calls
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetMockCalls(browser) {
  return browser.execute(() => {
    return window.__ARTIFEX_DEBUG__.mock?.getMockCalls() ?? [];
  });
}

/**
 * Get mock call history for a specific command
 * @param {WebDriverIO.Browser} browser
 * @param {string} command
 */
export async function debugGetMockCallHistory(browser, command) {
  return browser.execute(
    (cmd) => {
      return window.__ARTIFEX_DEBUG__.mock?.getMockCallHistory(cmd) ?? [];
    },
    command
  );
}

/**
 * Reset mock call history
 * @param {WebDriverIO.Browser} browser
 */
export async function debugResetMockCalls(browser) {
  return browser.execute(() => {
    if (window.__ARTIFEX_DEBUG__.mock) {
      window.__ARTIFEX_DEBUG__.mock.resetMockCalls();
    }
  });
}

/**
 * Simulate job progress event manually
 * @param {WebDriverIO.Browser} browser
 * @param {string} jobId
 * @param {number} percent
 * @param {string} message
 */
export async function debugSimulateJobProgress(browser, jobId, percent, message) {
  return browser.execute(
    (id, pct, msg) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        return window.__ARTIFEX_DEBUG__.mock.simulateJobProgress(id, pct, msg);
      }
      return Promise.resolve();
    },
    jobId,
    percent,
    message
  );
}

/**
 * Simulate job completed event manually
 * @param {WebDriverIO.Browser} browser
 * @param {string} jobId
 * @param {string[]} assetIds
 */
export async function debugSimulateJobCompleted(browser, jobId, assetIds) {
  return browser.execute(
    (id, assetIdsArr) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        return window.__ARTIFEX_DEBUG__.mock.simulateJobCompleted(id, assetIdsArr);
      }
      return Promise.resolve();
    },
    jobId,
    assetIds
  );
}

/**
 * Simulate job failed event manually
 * @param {WebDriverIO.Browser} browser
 * @param {string} jobId
 * @param {string} error
 */
export async function debugSimulateJobFailed(browser, jobId, error) {
  return browser.execute(
    (id, errMsg) => {
      if (window.__ARTIFEX_DEBUG__.mock) {
        return window.__ARTIFEX_DEBUG__.mock.simulateJobFailed(id, errMsg);
      }
      return Promise.resolve();
    },
    jobId,
    error
  );
}

/**
 * Wait for route to match expected path
 * @param {WebDriverIO.Browser} browser
 * @param {string} expectedPath
 * @param {number} timeout
 */
export async function debugWaitForRoute(browser, expectedPath, timeout = 5000) {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    const route = await debugGetRoute(browser);
    if (route.path === expectedPath) {
      return true;
    }
    await browser.pause(50);
  }
  throw new Error(`Route did not become ${expectedPath} (was ${(await debugGetRoute(browser)).path})`);
}
