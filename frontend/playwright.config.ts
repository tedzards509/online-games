import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  use: { baseURL: 'http://localhost:15173', trace: 'retain-on-failure' },
  webServer: [
    {
      command: 'cargo run --manifest-path ../Cargo.toml -p arena-server',
      url: 'http://127.0.0.1:13000/api/health',
      env: {
        DATABASE_URL: 'sqlite::memory:',
        PUBLIC_ORIGIN: 'http://localhost:15173',
        BIND_ADDR: '127.0.0.1:13000',
      },
      timeout: 120000,
    },
    {
      command: 'npm run dev -- --port 15173',
      url: 'http://localhost:15173',
      env: { ARENA_API_TARGET: 'http://127.0.0.1:13000' },
      timeout: 30000,
    },
  ],
});
