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
 * Get current viewport info
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetViewport(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getViewport());
}

/**
 * Get current Tauri context
 * @param {WebDriverIO.Browser} browser
 */
export async function debugGetTauriContext(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.getTauriContext());
}

/**
 * Take a full page snapshot
 * @param {WebDriverIO.Browser} browser
 */
export async function debugSnapshot(browser) {
  return browser.execute(() => window.__ARTIFEX_DEBUG__.snapshot());
}
