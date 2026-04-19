/**
 * E2E Test: 01-app-launch
 * Verifies the app starts, window exists, and debug harness loads correctly.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  getDebugAPI,
  debugGetViewport,
} from '../helpers/debug-api.js';

describe('01 App Launch', () => {
  it('should create a window handle', async () => {
    const handle = await browser.getWindowHandle();
    assert.ok(handle, 'Should have a valid window handle');
  });

  it('should render the body element as visible', async () => {
    const body = await browser.$('body');
    assert.ok(await body.isDisplayed(), 'Body should be visible');
  });

  it('should initialize the debug harness via waitForAppReady', async () => {
    await waitForAppReady(browser);
    const api = await getDebugAPI(browser);
    assert.ok(api, 'Debug API should be available');
    assert.ok(api.getStores, 'getStores should be a function');
  });

  it('should expose correct debug API version', async () => {
    await waitForAppReady(browser);
    const api = await getDebugAPI(browser);
    assert.ok(api.version, 'Version should be defined');
    assert.ok(typeof api.version === 'string', 'Version should be a string');
    assert.ok(api.version.length > 0, 'Version should not be empty');
  });
});
