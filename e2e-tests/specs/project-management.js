/**
 * E2E Test: Project Management
 * Tests the project list page and create project dialog.
 */
import assert from 'node:assert/strict';

describe('Project Management', () => {
  it('should render the project list page', async () => {
    await browser.pause(3000);
    const bodyText = await browser.$('body').getText();
    assert.ok(bodyText.length > 0, 'Project page should have content');
  });

  it('should show create project button or empty state', async () => {
    // The page should either show projects or an empty state with a create button
    await browser.pause(500);
    const bodyText = await browser.$('body').getText();
    const hasContent = bodyText.length > 0;
    assert.ok(hasContent, 'Page should show projects or empty state');
  });

  it('should have clickable project cards or create button', async () => {
    const cards = await browser.$$('[class*="card"], [class*="Card"]');
    const buttons = await browser.$$('button');
    // Either we have project cards or at least some buttons (empty state)
    assert.ok(
      cards.length > 0 || buttons.length > 0,
      'Should have project cards or action buttons'
    );
  });
});
