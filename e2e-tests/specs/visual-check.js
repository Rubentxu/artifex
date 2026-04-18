import assert from 'node:assert/strict';

describe('App Visual Check', () => {
  it('should render the full app UI', async () => {
    await browser.pause(3000);

    const bodyText = await browser.$('body').getText();

    // Verify the app rendered properly (not stuck on "Loading...")
    assert.ok(!bodyText.startsWith('Loading...'), 'App should not be stuck on Loading...');
    assert.ok(bodyText.includes('Artifex'), 'App should show Artifex branding');
    assert.ok(bodyText.includes('Projects'), 'App should show Projects navigation');
  });
});
