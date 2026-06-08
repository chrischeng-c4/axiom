import { defineConfig } from '@jet/test';

export default defineConfig({
  testDir: '.',
  timeout: 30000,
  retries: 0,
  outputDir: 'test-results',
  use: {
    headless: true,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  reporter: [['html']],
  projects: [
    { name: 'vite-build', use: { baseURL: 'http://localhost:4174' }, testMatch: '**/build.spec.ts' },
    { name: 'jet-build',  use: { baseURL: 'http://localhost:4175' }, testMatch: '**/build.spec.ts' },
    { name: 'jet-dev',    use: { baseURL: 'http://localhost:3000' }, testMatch: ['**/dev-server.spec.ts', '**/hmr.spec.ts', '**/css.spec.ts'] },
  ],
});
