/**
 * E2E Test: 17-agent-workflow
 * Verifies agent multi-step workflow with mock backend
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

describe('17 Agent Workflow', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await navigateTo(browser, '/agent');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should load agent page', async () => {
    const route = await browser.execute(() => window.location.pathname);
    assert.ok(route === '/agent', `Should be on /agent, got ${route}`);
  });

  it('should have agent store', async () => {
    const agentStore = await debugGetStore(browser, 'agent');
    assert.ok(agentStore !== undefined, 'agentStore should exist');
  });

  it('should have prompt input', async () => {
    const hasInput = await browser.execute(() => {
      const inputs = document.querySelectorAll('input, textarea');
      return Array.from(inputs).some(el =>
        el.placeholder?.toLowerCase().includes('prompt') ||
        el.placeholder?.toLowerCase().includes('task') ||
        el.placeholder?.toLowerCase().includes('instruction')
      );
    });
    assert.ok(hasInput, 'Should have prompt input');
  });

  it('should have start/run button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '');
    });
    const hasStart = buttons.some(b =>
      b.includes('start') || b.includes('run') || b.includes('execute') || b.includes('agent')
    );
    assert.ok(hasStart, 'Should have start/run button');
  });

  it('should record start_code_agent in mock call history', async () => {
    await debugResetMockCalls(browser);

    // Find and click start button
    await browser.execute(() => {
      const btns = Array.from(document.querySelectorAll('button'));
      const btn = btns.find(b =>
        b.textContent?.toLowerCase().includes('start') ||
        b.textContent?.toLowerCase().includes('run')
      );
      if (btn) btn.click();
    });

    await browser.pause(300);

    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'start_code_agent'),
      'Should call start_code_agent'
    );
  });
});
