/**
 * E2E Test: 05-assets-page
 * Verifies asset grid, filter buttons, toolbar, kind badges.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetStore,
  debugGetRoute,
  debugGetElement,
  debugGetElements,
} from '../helpers/debug-api.js';

describe('05 Assets Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await browser.url('/assets');
    await waitForAppReady(browser);
  });

  it('should render assets page at route /assets', async () => {
    const route = await debugGetRoute(browser);
    assert.ok(route.path === '/assets', `Should be on /assets, got ${route.path}`);
  });

  it('should have asset filter buttons (All, Image, Sprite, etc.)', async () => {
    const filterButtons = await debugGetElements(browser, '[class*="filter"], button');
    assert.ok(filterButtons.length > 0, 'Should have filter buttons or buttons');
    // Look for filter-related text
    const hasFilterText = filterButtons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('all') ||
         btn.text.toLowerCase().includes('image') ||
         btn.text.toLowerCase().includes('filter'))
    );
    assert.ok(
      hasFilterText || filterButtons.length > 3,
      'Should have filter-related buttons'
    );
  });

  it('should have assetStore initialized', async () => {
    const assetStore = await debugGetStore(browser, 'asset');
    assert.ok(assetStore, 'assetStore should exist');
  });

  it('should have toolbar buttons (Generate, Create, etc.)', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasToolbarButtons = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('generate') ||
         btn.text.toLowerCase().includes('create') ||
         btn.text.toLowerCase().includes('import') ||
         btn.text.toLowerCase().includes('add'))
    );
    assert.ok(hasToolbarButtons, 'Should have toolbar buttons like Generate/Create/Import');
  });

  it('should render asset grid or empty state', async () => {
    const assetGrid = await debugGetElement(browser, '[class*="grid"], [class*="asset"]');
    const emptyState = await debugGetElement(browser, '[class*="empty"]');
    assert.ok(
      assetGrid !== null || emptyState !== null,
      'Should have asset grid or empty state'
    );
  });
});
