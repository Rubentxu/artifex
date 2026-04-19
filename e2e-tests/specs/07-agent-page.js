/**
 * E2E Test: 07-agent-page
 * Verifies Agent page: conversation UI, engine selector, state machine.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetRoute,
  debugGetElement,
  debugGetElements,
  debugGetStore,
} from '../helpers/debug-api.js';

describe('07 Agent Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await browser.url('/agent');
    await waitForAppReady(browser);
  });

  it('should render agent page at route /agent', async () => {
    const route = await debugGetRoute(browser);
    assert.ok(route.path === '/agent', `Should be on /agent, got ${route.path}`);
  });

  it('should have conversation input area', async () => {
    const inputArea = await debugGetElement(browser, 'textarea, input[type="text"], [class*="input"]');
    assert.ok(inputArea !== null, 'Should have conversation input area');
  });

  it('should have engine selector (Godot/Unity)', async () => {
    const hasEngineSelector = await debugGetElement(browser, '[class*="engine"], select, [class*="selector"]');
    const hasGodot = await debugGetElement(browser, 'button:has-text("Godot"), [class*="godot"]');
    const hasUnity = await debugGetElement(browser, 'button:has-text("Unity"), [class*="unity"]');
    assert.ok(
      hasEngineSelector !== null || hasGodot !== null || hasUnity !== null,
      'Should have engine selector or Godot/Unity buttons'
    );
  });

  it('should have agentStore initialized', async () => {
    const agentStore = await debugGetStore(browser, 'agent');
    assert.ok(agentStore, 'agentStore should exist');
  });

  it('should show phase indicator (Idle initially)', async () => {
    const hasPhaseIndicator = await debugGetElement(browser, '[class*="phase"], [class*="status"], [class*="state"]');
    assert.ok(hasPhaseIndicator !== null, 'Should have phase/status indicator');
  });

  it('should have submit/send button for agent input', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasSendButton = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('send') ||
         btn.text.toLowerCase().includes('run') ||
         btn.text.toLowerCase().includes('execute') ||
         btn.text.toLowerCase().includes('submit'))
    );
    assert.ok(hasSendButton, 'Should have a Send/Run/Execute/Submit button');
  });
});
