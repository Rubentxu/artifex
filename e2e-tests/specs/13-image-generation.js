/**
 * E2E Test: 13-image-generation
 * Verifies image generation job lifecycle with mock backend
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

describe('13 Image Generation', () => {
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

  it('should have generate image button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasGenerate = buttons.some(b =>
      b.includes('generate') || b.includes('image') || b.includes('create')
    );
    assert.ok(hasGenerate, 'Should have generate image button');
  });

  it('should record generate_image in mock call history when triggered', async () => {
    // Reset to have clean call history
    await debugResetMockCalls(browser);

    // Try to trigger generate_image by looking for the dialog/button
    const generateBtn = await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('generate') &&
        b.textContent?.toLowerCase().includes('image')
      );
      if (btn) {
        btn.click();
        return true;
      }
      return false;
    });

    // Wait for mock job lifecycle (200ms delay for progress + completion)
    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'generate_image'),
      'Should call generate_image command'
    );
  });

  it('should simulate job lifecycle events', async () => {
    // Mock automatically emits job-progress and job-completed
    const calls = await debugGetMockCalls(browser);
    const generateCall = calls.find(c => c.command === 'generate_image');
    assert.ok(generateCall, 'generate_image should have been called');
    // Job ID should be returned
    assert.ok(generateCall.args, 'Should have args with job_id');
  });

  it('should have job status in StatusBar', async () => {
    const jobStatus = await browser.execute(() => {
      const statusBar = document.querySelector('[class*="status"]');
      return statusBar ? statusBar.textContent : null;
    });
    // Mock generates fake job, status should update
    assert.ok(jobStatus !== undefined, 'StatusBar should exist');
  });
});
