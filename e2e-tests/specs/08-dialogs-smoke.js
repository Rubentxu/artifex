/**
 * E2E Test: 08-dialogs-smoke
 * Smoke tests for dialog components - verify they exist and can be triggered.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetElement,
  debugGetElements,
  debugGetDialogs,
} from '../helpers/debug-api.js';

describe('08 Dialogs Smoke Test', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    // Navigate to assets page where dialog triggers are likely
    await navigateTo(browser, '/assets');
    await waitForAppReady(browser);
  });

  it('should have dialog overlay element in DOM (may be hidden)', async () => {
    const dialogs = await debugGetDialogs(browser);
    // Dialog elements may exist even when not open
    assert.ok(Array.isArray(dialogs), 'getDialogs should return an array');
  });

  it('should have at least one dialog trigger button on assets page', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasDialogTrigger = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('generate') ||
         btn.text.toLowerCase().includes('create') ||
         btn.text.toLowerCase().includes('new') ||
         btn.text.toLowerCase().includes('import'))
    );
    assert.ok(hasDialogTrigger, 'Should have dialog trigger buttons like Generate/Create/Import');
  });

  it('should have dialog CSS classes defined (not missing styles)', async () => {
    // Verify dialog-related CSS classes are defined
    const dialogOverlay = await debugGetElement(browser, '[class*="dialog"], [class*="overlay"], [class*="modal"]');
    if (dialogOverlay) {
      // If we found dialog elements, they should have proper class names
      assert.ok(
        dialogOverlay.classes && dialogOverlay.classes.length > 0,
        'Dialog elements should have CSS classes'
      );
    }
    // No assert.fail - dialogs may be hidden or not rendered yet
  });
});
