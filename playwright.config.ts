import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 15000,
  use: {
    baseURL: 'http://localhost:1420',
    headless: true,
  },
  // Start dev server before tests (requires pnpm tauri dev running separately for Tauri native features)
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: true,
    timeout: 30000,
  },
});
