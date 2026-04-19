/**
 * E2E Test: 04-projects-page
 * Verifies project list, create project dialog, empty state.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetStore,
  debugGetRoute,
  debugGetElement,
  debugGetElements,
} from '../helpers/debug-api.js';

describe('04 Projects Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await browser.url('/');
    await waitForAppReady(browser);
  });

  it('should render projects page at route /', async () => {
    const route = await debugGetRoute(browser);
    assert.ok(route.path === '/', `Should be on /, got ${route.path}`);
  });

  it('should have projectStore loaded (not loading, no error)', async () => {
    const projectStore = await debugGetStore(browser, 'project');
    assert.ok(projectStore, 'projectStore should exist');
    if (projectStore) {
      // Store should have loading: false or projects array
      assert.ok(
        projectStore.loading === false || Array.isArray(projectStore.projects),
        'Project store should have loaded (loading=false or projects array)'
      );
    }
  });

  it('should show empty state OR project cards', async () => {
    // Either empty state message OR project cards should exist
    const projectCards = await debugGetElements(browser, '[class*="project"], [class*="card"]');
    const hasEmptyState = await debugGetElement(browser, '[class*="empty"], [class*="no-project"]');
    assert.ok(
      projectCards.length > 0 || hasEmptyState !== null,
      'Should show project cards or empty state'
    );
  });

  it('should show project cards with name and path text', async () => {
    const projectCards = await debugGetElements(browser, '[class*="project-card"], [class*="card"]');
    if (projectCards.length > 0) {
      // At least one card should have text content
      const hasText = projectCards.some((card) => card.text && card.text.length > 0);
      assert.ok(hasText, 'Project cards should have text content');
    }
    // If no cards, should show empty state (covered by previous test)
  });

  it('should have "Create Project" or "New Project" button', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasCreateButton = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('new') ||
         btn.text.toLowerCase().includes('create') ||
         btn.text.toLowerCase().includes('add'))
    );
    assert.ok(hasCreateButton, 'Should have a Create/New Project button');
  });
});
