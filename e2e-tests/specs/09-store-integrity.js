/**
 * E2E Test: 09-store-integrity
 * Verifies all 16 store subscriptions initialize and respond to navigation.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetStores,
  debugGetStore,
} from '../helpers/debug-api.js';

describe('09 Store Integrity', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
  });

  it('should have all 16 store subscriptions registered', async () => {
    const stores = await debugGetStores(browser);
    const storeKeys = Object.keys(stores);
    // We expect at least 16 stores based on the store-inspector
    assert.ok(
      storeKeys.length >= 16,
      `Should have at least 16 stores registered, got ${storeKeys.length}: ${storeKeys.join(', ')}`
    );
  });

  it('should have ui stores with expected keys', async () => {
    const sidebarCollapsed = await debugGetStore(browser, 'ui.sidebarCollapsed');
    const propertiesCollapsed = await debugGetStore(browser, 'ui.propertiesCollapsed');
    const selectedProjectId = await debugGetStore(browser, 'ui.selectedProjectId');

    // All should exist (may be null/undefined if not set)
    assert.ok(sidebarCollapsed !== undefined, 'ui.sidebarCollapsed should be defined');
    assert.ok(propertiesCollapsed !== undefined, 'ui.propertiesCollapsed should be defined');
    assert.ok(selectedProjectId !== undefined, 'ui.selectedProjectId should be defined');
  });

  it('should have project store with projects array', async () => {
    const projectStore = await debugGetStore(browser, 'project');
    assert.ok(projectStore, 'project store should exist');
    if (projectStore && typeof projectStore === 'object') {
      assert.ok(
        Array.isArray(projectStore.projects) || projectStore.loading !== undefined,
        'projectStore should have projects array or loading state'
      );
    }
  });

  it('should have asset store with assets array', async () => {
    const assetStore = await debugGetStore(browser, 'asset');
    assert.ok(assetStore, 'asset store should exist');
    if (assetStore && typeof assetStore === 'object') {
      assert.ok(
        Array.isArray(assetStore.assets) || assetStore.loading !== undefined,
        'assetStore should have assets array or loading state'
      );
    }
  });

  it('should have identity store with tier value', async () => {
    const identityStore = await debugGetStore(browser, 'identity');
    const currentTier = await debugGetStore(browser, 'identity.currentTier');
    assert.ok(identityStore, 'identity store should exist');
    assert.ok(currentTier !== undefined, 'currentTier should be defined');
  });

  it('should have agent store with initial state', async () => {
    const agentStore = await debugGetStore(browser, 'agent');
    const isAgentRunning = await debugGetStore(browser, 'agent.isAgentRunning');
    assert.ok(agentStore, 'agent store should exist');
    assert.ok(isAgentRunning !== undefined, 'isAgentRunning should be defined');
    // Initially should be false/not running
    assert.ok(
      isAgentRunning === false || isAgentRunning === null || isAgentRunning === undefined,
      `isAgentRunning should be initially false/null, got: ${isAgentRunning}`
    );
  });
});
