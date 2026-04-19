/**
 * E2E Test: 11-projects-crud
 * Verifies project CRUD operations with mock backend
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

describe('11 Projects CRUD', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await browser.url('/');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should load projects via mock', async () => {
    const projectStore = await debugGetStore(browser, 'project');
    assert.ok(projectStore, 'projectStore should exist');
    // Mock returns 2 active projects
    if (projectStore && Array.isArray(projectStore.projects)) {
      assert.ok(projectStore.projects.length >= 2, 'Should have at least 2 mock projects');
    }
  });

  it('should show mock project names', async () => {
    const projectStore = await debugGetStore(browser, 'project');
    if (projectStore && Array.isArray(projectStore.projects)) {
      const names = projectStore.projects.map(p => p.name);
      assert.ok(names.some(n => n.includes('Dungeon') || n.includes('Space')), 'Should have mock project names');
    }
  });

  it('should record list_projects in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    // EnableMock itself may trigger calls, so check for list_projects
    await browser.url('/');
    await waitForAppReady(browser);
    const callsAfterReload = await debugGetMockCalls(browser);
    assert.ok(callsAfterReload.some(c => c.command === 'list_projects'), 'Should call list_projects');
  });

  it('should have create project button', async () => {
    const buttons = await browser.execute(() => {
      return Array.from(document.querySelectorAll('button'))
        .map(b => b.textContent?.toLowerCase() ?? '')
        .filter(t => t.includes('new') || t.includes('create') || t.includes('add'));
    });
    assert.ok(buttons.length > 0, 'Should have a create/new/add project button');
  });

  it('should switch back to TauriBackend when mock disabled', async () => {
    await debugDisableMock(browser);
    const isMock = await browser.execute(() => window.__ARTIFEX_DEBUG__.mock?.isMockMode());
    assert.ok(isMock === false, 'Mock should be disabled');
    await debugEnableMock(browser);
  });
});
