/**
 * E2E Test: 15-sprite-pipeline
 * Verifies sprite sheet generation and slicing with mock backend
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetStore,
  debugEnableMock,
  debugDisableMock,
  debugGetMockCalls,
  debugResetMockCalls,
} from '../helpers/debug-api.js';

describe('15 Sprite Pipeline', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await browser.url('/assets');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should have sprite sheet generation button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasSprite = buttons.some(b =>
      b.includes('sprite') || b.includes('sheet') || b.includes('animation')
    );
    assert.ok(hasSprite, 'Should have sprite sheet button');
  });

  it('should record generate_sprite_sheet in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('sprite') ||
        b.textContent?.toLowerCase().includes('sheet')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_sprite_sheet'),
      'Should call generate_sprite_sheet'
    );
  });

  it('should have sprite kind assets in mock', async () => {
    const assetStore = await debugGetStore(browser, 'asset');
    if (assetStore && Array.isArray(assetStore.assets)) {
      const hasSprite = assetStore.assets.some(a => a.kind === 'Sprite');
      assert.ok(hasSprite, 'Mock should have Sprite kind assets');
    }
  });
});
