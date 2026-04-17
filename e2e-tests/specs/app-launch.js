/**
 * E2E Test: App Launch
 * Verifies the app starts and shows the project list page.
 */
import assert from 'node:assert/strict';

describe('App Launch', () => {
  it('should launch and create a window', async () => {
    // getWindowHandle proves the app window exists
    const handle = await browser.getWindowHandle();
    assert.ok(handle, 'Should have a valid window handle');
  });

  it('should render the body element', async () => {
    const body = await browser.$('body');
    assert.ok(await body.isDisplayed(), 'Body should be visible');
  });

  it('should have page content after loading', async () => {
    // Wait for SvelteKit hydration
    await browser.pause(3000);
    const bodyText = await browser.$('body').getText();
    assert.ok(bodyText.length > 0, 'Page should have content after loading');
  });
});
