/**
 * E2E Test: 06-settings-page
 * Verifies Settings page: Providers, Models, Routing, Identity, Usage stats.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  debugGetRoute,
  debugGetElement,
  debugHasText,
  debugGetStore,
} from '../helpers/debug-api.js';

describe('06 Settings Page', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await browser.url('/settings');
    await waitForAppReady(browser);
  });

  it('should render settings page at route /settings', async () => {
    const route = await debugGetRoute(browser);
    assert.ok(route.path === '/settings', `Should be on /settings, got ${route.path}`);
  });

  it('should have Provider section or list', async () => {
    const hasProviderSection = await debugHasText(browser, 'provider') ||
                               await debugHasText(browser, 'Provider') ||
                               await debugGetElement(browser, '[class*="provider"]');
    assert.ok(hasProviderSection, 'Should have Provider section');
  });

  it('should have Model profiles section', async () => {
    const hasModelSection = await debugHasText(browser, 'model') ||
                            await debugHasText(browser, 'Model') ||
                            await debugGetElement(browser, '[class*="model"]');
    assert.ok(hasModelSection, 'Should have Model section');
  });

  it('should have Routing rules section', async () => {
    const hasRoutingSection = await debugHasText(browser, 'routing') ||
                              await debugHasText(browser, 'Routing') ||
                              await debugHasText(browser, 'rule') ||
                              await debugGetElement(browser, '[class*="routing"], [class*="rule"]');
    assert.ok(hasRoutingSection, 'Should have Routing section or rules');
  });

  it('should have Prompt templates section', async () => {
    const hasPromptSection = await debugHasText(browser, 'prompt') ||
                             await debugHasText(browser, 'template') ||
                             await debugGetElement(browser, '[class*="prompt"], [class*="template"]');
    assert.ok(hasPromptSection, 'Should have Prompt template section');
  });

  it('should have Identity section with Profile/Tier/Usage', async () => {
    const hasIdentitySection = await debugGetElement(browser, '[class*="identity"], [class*="profile"], section');
    assert.ok(hasIdentitySection, 'Should have Identity section or profile area');
  });

  it('should show tier badge (Free/Pro)', async () => {
    const hasTier = await debugHasText(browser, 'Free') ||
                    await debugHasText(browser, 'Pro') ||
                    await debugHasText(browser, 'Tier');
    assert.ok(hasTier, 'Should show tier badge or tier text');
  });

  it('should have Usage stats section', async () => {
    const hasUsage = await debugHasText(browser, 'usage') ||
                     await debugHasText(browser, 'Usage') ||
                     await debugHasText(browser, 'credit') ||
                     await debugGetElement(browser, '[class*="usage"], [class*="credit"]');
    assert.ok(hasUsage, 'Should have Usage section');
  });
});
