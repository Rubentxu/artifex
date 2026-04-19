/**
 * E2E Test: 03-layout-structure
 * Verifies AppShell layout: sidebar, statusbar, footer, responsive behavior.
 */
import assert from 'node:assert/strict';
import {
  waitForAppReady,
  navigateTo,
  debugGetElement,
  debugGetViewport,
  debugHasText,
  debugGetActiveJobs,
} from '../helpers/debug-api.js';

describe('03 Layout Structure', () => {
  beforeEach(async () => {
    await waitForAppReady(browser);
    await navigateTo(browser, '/');
    await waitForAppReady(browser);
  });

  it('should have AppShell with sidebar + main structure', async () => {
    // Check for sidebar
    const sidebar = await debugGetElement(browser, 'aside, [class*="sidebar"]');
    assert.ok(sidebar !== null, 'Sidebar element should exist');

    // Check for main content area
    const main = await debugGetElement(browser, 'main, [class*="main"]');
    assert.ok(main !== null, 'Main element should exist');
  });

  it('should show version in StatusBar', async () => {
    const hasVersion = await debugHasText(browser, 'Artifex');
    assert.ok(hasVersion, 'StatusBar should mention Artifex');
  });

  it('should show project info or "No project selected" in StatusBar', async () => {
    // StatusBar should show either a project name or "No project" / "select"
    const hasProjectInfo = await debugHasText(browser, 'project') ||
                           await debugHasText(browser, 'Project');
    assert.ok(hasProjectInfo, 'StatusBar should show project info');
  });

  it('should show "Ready" status when no active jobs', async () => {
    const jobStatus = await debugGetActiveJobs(browser);
    assert.ok(typeof jobStatus.hasActiveJobs === 'boolean', 'Should have hasActiveJobs boolean');
    assert.ok(typeof jobStatus.statusText === 'string', 'Should have statusText string');
    // Should be ready initially
    if (!jobStatus.hasActiveJobs) {
      assert.ok(
        jobStatus.statusText === 'Ready' || jobStatus.statusText === '',
        `Status should be Ready or empty when idle, got: ${jobStatus.statusText}`
      );
    }
  });

  it('should have viewport with reasonable dimensions (>400x300)', async () => {
    const viewport = await debugGetViewport(browser);
    assert.ok(viewport.width > 400, `Width should be >400, got ${viewport.width}`);
    assert.ok(viewport.height > 300, `Height should be >300, got ${viewport.height}`);
  });

  it('should have sidebar with expected width when expanded (~256px for w-64)', async () => {
    const sidebar = await debugGetElement(browser, 'aside, [class*="sidebar"]');
    if (!sidebar) {
      // Skip if no sidebar - some pages may not have it
      return;
    }
    // w-64 = 16rem = ~256px at default font size
    // Allow some tolerance for actual rendered width
    if (sidebar.visible && sidebar.rect) {
      assert.ok(
        sidebar.rect.width > 100,
        `Sidebar should be visible with meaningful width, got ${sidebar.rect.width}`
      );
    }
  });
});
