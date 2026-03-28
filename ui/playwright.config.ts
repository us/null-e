import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:1420',
    headless: true,
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'bun run dev',
    port: 1420,
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
