/**
 * E2E Test: 22-model-config
 * Verifies providers, profiles, routing rules with mock backend
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugEnableMock,
  debugDisableMock,
  debugGetMockCalls,
  debugResetMockCalls,
} from '../helpers/debug-api.js';

describe('22 Model Config', () => {
  before(async () => {
    await waitForAppReady(browser);
    await debugEnableMock(browser);
    await browser.url('/settings');
    await waitForAppReady(browser);
  });

  after(async () => {
    await debugDisableMock(browser);
  });

  beforeEach(async () => {
    await debugResetMockCalls(browser);
  });

  it('should load settings page', async () => {
    const route = await browser.execute(() => window.location.pathname);
    assert.ok(route === '/settings', `Should be on /settings, got ${route}`);
  });

  it('should have providers section', async () => {
    const hasProviders = await browser.execute(() => {
      const text = document.body.textContent?.toLowerCase() ?? '';
      return text.includes('provider') || text.includes('api key');
    });
    assert.ok(hasProviders, 'Should have providers section');
  });

  it('should have model profiles section', async () => {
    const hasProfiles = await browser.execute(() => {
      const text = document.body.textContent?.toLowerCase() ?? '';
      return text.includes('profile') || text.includes('model');
    });
    assert.ok(hasProfiles, 'Should have model profiles section');
  });

  it('should record list_providers in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'list_providers'),
      'Should call list_providers'
    );
  });

  it('should record list_model_profiles in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'list_model_profiles'),
      'Should call list_model_profiles'
    );
  });

  it('should record list_routing_rules in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'list_routing_rules'),
      'Should call list_routing_rules'
    );
  });

  it('should record list_prompt_templates in mock call history', async () => {
    const calls = await debugGetMockCalls(browser);
    assert.ok(
      calls.some(c => c.command === 'list_prompt_templates'),
      'Should call list_prompt_templates'
    );
  });
});
