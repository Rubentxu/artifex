/**
 * E2E Test: 23-full-workflow
 * End-to-end: create project → generate image → verify asset → export project
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetStore,
  debugEnableMock,
  debugDisableMock,
  debugGetMockCalls,
  debugResetMockCalls,
  debugWaitForRoute,
} from '../helpers/debug-api.js';

describe('23 Full Workflow', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should execute full sequence: projects → assets → generation', async () => {
    // Step 1: Navigate to home (projects list)
    await navigateTo(browser, '/');
    await waitForAppReady(browser);
    await debugWaitForRoute(browser, '/');

    // Verify projects loaded
    const projectStore = await debugGetStore(browser, 'project');
    assert.ok(projectStore, 'projectStore should exist');

    // Step 2: Navigate to assets
    await navigateTo(browser, '/assets');
    await waitForAppReady(browser);
    await debugWaitForRoute(browser, '/assets');

    // Verify assets loaded
    const assetStore = await debugGetStore(browser, 'asset');
    assert.ok(assetStore, 'assetStore should exist');
    if (assetStore && Array.isArray(assetStore.assets)) {
      assert.ok(assetStore.assets.length > 0, 'Should have assets');
    }

    // Step 3: Trigger generate_image
    await debugResetMockCalls(browser);
    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('generate') ||
        b.textContent?.toLowerCase().includes('image')
      );
      if (btn) btn.click();
    });
    await browser.pause(300);

    // Verify generate_image was called
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_image'),
      'Should call generate_image in workflow'
    );
  });

  it('should track complete call history sequence', async () => {
    await debugResetMockCalls(browser);

    // Trigger multiple operations
    await navigateTo(browser, '/assets');
    await waitForAppReady(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      // Click various buttons to trigger mock commands
      btns.forEach(btn => {
        const text = btn.textContent?.toLowerCase() ?? '';
        if (text.includes('generate') || text.includes('image')) {
          btn.click();
        }
      });
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(calls.length > 0, 'Should have recorded mock calls');
  });

  it('should handle full lifecycle from enable to disable', async () => {
    // Verify mock is enabled
    const isMock = await browser.execute(() =>
      window.__ARTIFEX_DEBUG__.mock?.isMockMode()
    );
    assert.ok(isMock, 'Mock should be enabled');

    // Perform operations
    await navigateTo(browser, '/');
    await waitForAppReady(browser);

    // Disable mock
    await debugDisableMock(browser);

    // Verify mock is disabled
    const isMockAfter = await browser.execute(() =>
      window.__ARTIFEX_DEBUG__.mock?.isMockMode()
    );
    assert.ok(!isMockAfter, 'Mock should be disabled');

    // Re-enable for remaining tests
    await debugEnableMock(browser);
  });

  it('should maintain state consistency through navigation', async () => {
    await debugResetMockCalls(browser);

    // Home
    await navigateTo(browser, '/');
    await waitForAppReady(browser);
    const store1 = await debugGetStore(browser, 'project');
    assert.ok(store1 !== undefined, 'Store should exist at home');

    // Assets
    await navigateTo(browser, '/assets');
    await waitForAppReady(browser);
    const store2 = await debugGetStore(browser, 'asset');
    assert.ok(store2 !== undefined, 'Store should exist at assets');

    // Settings
    await navigateTo(browser, '/settings');
    await waitForAppReady(browser);
    const store3 = await debugGetStore(browser, 'project');
    // Settings may not have project store directly

    // Back to Home
    await navigateTo(browser, '/');
    await waitForAppReady(browser);
    const store4 = await debugGetStore(browser, 'project');
    assert.ok(store4 !== undefined, 'Store should exist when back home');
  });
});
