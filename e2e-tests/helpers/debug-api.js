/**
 * E2E Debug API Helpers for WebDriverIO
 * Wraps window.__ARTIFEX_DEBUG__ for use in specs
 */

/**
 * Wait for the debug API to be available on the window
 * @param {WebDriverIO.Browser} browser
 * @param {number} [timeout=5000]
 */
export async function waitForAppReady(browser, timeout = 10000) {
  await browser.waitUntil(
    () => browser.execute(() => !!window.__ARTIFEX_DEBUG__),
    { timeout, timeoutMsg: 'Debug API not available' }
  );
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
