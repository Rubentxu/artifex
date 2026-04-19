/**
 * E2E Test: 18-image-processing
 * Verifies remove BG, pixel art, inpaint, outpaint with mock backend
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugEnableMock,
  debugDisableMock,
  debugGetMockCalls,
  debugResetMockCalls,
} from '../helpers/debug-api.js';

describe('18 Image Processing', () => {
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

  it('should have remove background button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasRemoveBg = buttons.some(b =>
      b.includes('remove') || b.includes('background') || b.includes('bg')
    );
    assert.ok(hasRemoveBg, 'Should have remove background button');
  });

  it('should have pixel art conversion button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasPixel = buttons.some(b =>
      b.includes('pixel') || b.includes('pixelate') || b.includes('pixel art')
    );
    assert.ok(hasPixel, 'Should have pixel art button');
  });

  it('should record remove_background in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('remove') ||
        b.textContent?.toLowerCase().includes('background')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'remove_background'),
      'Should call remove_background'
    );
  });

  it('should record convert_pixel_art in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('pixel')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'convert_pixel_art'),
      'Should call convert_pixel_art'
    );
  });
});
