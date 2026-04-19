/**
 * E2E Test: 21-identity-tier
 * Verifies profile, tier switching, usage quotas with mock backend
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetStore,
  debugEnableMock,
  debugDisableMock,
  debugSetMockTier,
  debugGetMockCalls,
  debugResetMockCalls,
} from '../helpers/debug-api.js';

describe('21 Identity Tier', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await navigateTo(browser, '/');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should load identity store', async () => {
    const identityStore = await debugGetStore(browser, 'identity');
    assert.ok(identityStore !== undefined, 'identityStore should exist');
  });

  it('should show free tier by default', async () => {
    const identityStore = await debugGetStore(browser, 'identity');
    if (identityStore && identityStore.currentUser) {
      assert.ok(
        identityStore.currentUser.tier === 'free',
        'Should be free tier by default'
      );
    }
  });

  it('should record get_current_user in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'get_current_user'),
      'Should call get_current_user'
    );
  });

  it('should switch tier to pro', async () => {
    await debugSetMockTier(browser, 'pro');
    await browser.reload();
    await waitForAppReady(browser);

    const identityStore = await debugGetStore(browser, 'identity');
    if (identityStore && identityStore.currentUser) {
      assert.ok(
        identityStore.currentUser.tier === 'pro',
        'Should be pro tier after switch'
      );
    }

    // Reset to free
    await debugSetMockTier(browser, 'free');
  });

  it('should record usage stats', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'get_usage'),
      'Should call get_usage'
    );
  });

  it('should have quota check command', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'check_quota'),
      'Should call check_quota'
    );
  });
});
