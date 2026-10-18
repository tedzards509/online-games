import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  use: { baseURL: 'http://localhost:5173', trace: 'retain-on-failure' },
  webServer: [
    {
      command: 'cargo run --manifest-path ../Cargo.toml -p arena-server',
      url: 'http://127.0.0.1:3000/api/health',
      env: {
        DATABASE_URL: 'sqlite::memory:',
        PUBLIC_ORIGIN: 'http://localhost:5173',
      },
      timeout: 120000,
    },
    { command: 'npm run dev', url: 'http://localhost:5173', timeout: 30000 },
  ],
});
