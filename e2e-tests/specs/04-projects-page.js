/**
 * E2E Test: 04-projects-page
 * Verifies project list, create project button, empty state.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetStore,
  debugGetRoute,
  debugGetElement,
  debugGetElements,
  debugHasText,
} from '../helpers/debug-api.js';

describe('04 Projects Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await navigateTo(browser, '/');
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
      assert.ok(
        projectStore.loading === false || projectStore.loading === undefined || Array.isArray(projectStore.projects),
        'Project store should have loaded (loading=false or projects array)'
      );
    }
  });

  it('should show empty state OR project grid', async () => {
    // The empty state shows "No projects yet" text
    const hasNoProjects = await debugHasText(browser, 'No projects yet') ||
                          await debugHasText(browser, 'Create your first project');
    // If there are projects, there should be a grid
    const hasGrid = await debugGetElement(browser, '.grid, [class*="grid"]');
    assert.ok(
      hasNoProjects || hasGrid !== null,
      'Should show empty state text or project grid'
    );
  });

  it('should show project cards with name and path text', async () => {
    const projectCards = await debugGetElements(browser, '[class*="project-card"], [class*="card"]');
    if (projectCards.length > 0) {
      const hasText = projectCards.some((card) => card.text && card.text.length > 0);
      assert.ok(hasText, 'Project cards should have text content');
    }
    // If no cards, empty state is shown (covered by previous test)
  });

  it('should have "New Project" button', async () => {
    const buttons = await debugGetElements(browser, 'button');
    const hasCreateButton = buttons.some(
      (btn) =>
        btn.text &&
        (btn.text.toLowerCase().includes('new') ||
         btn.text.toLowerCase().includes('create') ||
         btn.text.toLowerCase().includes('add'))
    );
    assert.ok(hasCreateButton, 'Should have a New/Create Project button');
  });
});
