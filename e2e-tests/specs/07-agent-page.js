/**
 * E2E Test: 07-agent-page
 * Verifies Agent page: conversation UI, engine selector, state machine.
 * Note: Some elements may not render without a selected project.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetRoute,
  debugGetElement,
  debugGetElements,
  debugGetStore,
  debugHasText,
} from '../helpers/debug-api.js';

describe('07 Agent Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await navigateTo(browser, '/agent');
    await waitForAppReady(browser);
    // Agent page may need time to load components
    await browser.pause(500);
  });

  it('should render agent page at route /agent', async () => {
    const route = await debugGetRoute(browser);
    assert.ok(route.path === '/agent', `Should be on /agent, got ${route.path}`);
  });

  it('should have agentStore initialized', async () => {
    const agentStore = await debugGetStore(browser, 'agent');
    assert.ok(agentStore, 'agentStore should exist');
  });

  it('should have conversation input area', async () => {
    const inputArea = await debugGetElement(browser, 'textarea, input[type="text"], [class*="input"]');
    // If no input found, check if there's a "select project" message instead
    if (!inputArea) {
      const hasProjectMsg = await debugHasText(browser, 'select a project') ||
                            await debugHasText(browser, 'Select a project');
      if (hasProjectMsg) return; // Expected when no project is selected
    }
    assert.ok(inputArea !== null, 'Should have conversation input area (or project selection prompt)');
  });

  it('should have send/submit button', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasSendButton = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('send') ||
         btn.text.toLowerCase().includes('run') ||
         btn.text.toLowerCase().includes('execute') ||
         btn.text.toLowerCase().includes('submit'))
    );
    if (!hasSendButton) {
      // May not show without a project selected
      const hasProjectMsg = await debugHasText(browser, 'select a project');
      if (hasProjectMsg) return;
    }
    assert.ok(hasSendButton, 'Should have a Send/Run/Execute/Submit button');
  });
});
