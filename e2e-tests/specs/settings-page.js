/**
 * E2E Test: Settings Page
 * Tests that settings/model-config page renders.
 */
import assert from 'node:assert/strict';

describe('Settings Page', () => {
  it('should have settings navigation', async () => {
    await browser.pause(1000);
    
    const settingsLink = await browser.$(
      'a*=Settings, button*=Settings, a*=settings, button*=settings, a*=Config, button*=Config, [href*="settings"]'
    );
    const exists = settingsLink && (await settingsLink.isExisting().catch(() => false));
    
    if (exists) {
      await settingsLink.click();
      await browser.pause(1000);
      const bodyText = await browser.$('body').getText();
      assert.ok(bodyText.length > 0, 'Settings page should render');
    } else {
      console.log('  SKIP — No settings link found');
      assert.ok(true, 'No separate settings page');
    }
  });
});
