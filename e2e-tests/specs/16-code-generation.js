/**
 * E2E Test: 16-code-generation
 * Verifies code generation (single-shot) with mock backend
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
} from '../helpers/debug-api.js';

describe('16 Code Generation', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await navigateTo(browser, '/assets');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should have code generation button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasCode = buttons.some(b =>
      b.includes('code') || b.includes('script') || b.includes('godot') || b.includes('unity')
    );
    assert.ok(hasCode, 'Should have code generation button');
  });

  it('should record generate_code in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('code') ||
        b.textContent?.toLowerCase().includes('script')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_code'),
      'Should call generate_code'
    );
  });

  it('should have code kind assets in mock', async () => {
    const assetStore = await debugGetStore(browser, 'asset');
    if (assetStore && Array.isArray(assetStore.assets)) {
      const hasCode = assetStore.assets.some(a => a.kind === 'Code');
      assert.ok(hasCode, 'Mock should have Code kind assets');
    }
  });

  it('should have list_code_templates command', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'list_code_templates'),
      'Should call list_code_templates'
    );
  });
});
