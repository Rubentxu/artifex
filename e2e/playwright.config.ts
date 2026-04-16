import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60000,
  retries: 0,
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  // Use chromium for E2E
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
  ],
});
