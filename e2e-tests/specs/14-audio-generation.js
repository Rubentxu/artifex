/**
 * E2E Test: 14-audio-generation
 * Verifies audio generation job lifecycle with mock backend
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

describe('14 Audio Generation', () => {
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

  it('should have generate audio button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasAudio = buttons.some(b =>
      b.includes('audio') || b.includes('sound') || b.includes('music') || b.includes('generate')
    );
    assert.ok(hasAudio, 'Should have audio generation button');
  });

  it('should record generate_audio in mock call history', async () => {
    await debugResetMockCalls(browser);

    // Try to trigger audio generation
    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('audio') ||
        b.textContent?.toLowerCase().includes('sound') ||
        b.textContent?.toLowerCase().includes('music')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_audio'),
      'Should call generate_audio command'
    );
  });

  it('should have audio asset kind in mock data', async () => {
    const assetStore = await debugGetStore(browser, 'asset');
    if (assetStore && Array.isArray(assetStore.assets)) {
      const hasAudioKind = assetStore.assets.some(a => a.kind === 'Audio');
      assert.ok(hasAudioKind, 'Mock should have Audio kind assets');
    }
  });
});
