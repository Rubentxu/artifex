/**
 * E2E Test: 19-materials-3d
 * Verifies material generation and 3D render with mock backend
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugEnableMock,
  debugDisableMock,
  debugGetMockCalls,
  debugResetMockCalls,
} from '../helpers/debug-api.js';

describe('19 Materials 3D', () => {
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

  it('should have material generation button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasMaterial = buttons.some(b =>
      b.includes('material') || b.includes('texture') || b.includes('pbr')
    );
    assert.ok(hasMaterial, 'Should have material generation button');
  });

  it('should have 3D render button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const has3D = buttons.some(b =>
      b.includes('render') || b.includes('3d') || b.includes('3d render')
    );
    assert.ok(has3D, 'Should have 3D render button');
  });

  it('should record generate_material in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('material') ||
        b.textContent?.toLowerCase().includes('texture')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_material'),
      'Should call generate_material'
    );
  });

  it('should record render_3d_to_sprites in mock call history', async () => {
    await debugResetMockCalls(browser);

    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('render') ||
        b.textContent?.toLowerCase().includes('3d')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'render_3d_to_sprites'),
      'Should call render_3d_to_sprites'
    );
  });
});
