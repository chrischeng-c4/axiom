# Migration Guide: From @playwright/test to jet native runner

<!-- REQ: R5, R8 -->

This guide helps you move from `@playwright/test` (legacy Playwright runner) to
the `jet` native test runner. The `--playwright` escape hatch lets you run
existing Playwright specs unchanged during the transition window.

## Deprecation Timeline

| Milestone | Version | Description |
|-----------|---------|-------------|
| Deprecated | v0.5.x (current) | `jet test --playwright` is a supported escape hatch. Deprecation warning printed on stderr. |
| Planned removal | v0.7.x | `--playwright` flag removed. Migrate before this release. |

> To suppress the deprecation warning during the migration window, set
> `JET_SUPPRESS_PLAYWRIGHT_WARNING=1` in your environment.

## Flag Mapping Table

Use this table to rewrite Playwright CLI flags to jet native equivalents.

<!-- REQ: R8 -->

| Playwright / `npx playwright test` flag | Jet native equivalent | Notes |
|-----------------------------------------|-----------------------|-------|
| `--reporter=html` | `--reporter=html` | HTML reporter is built in to `jet test`. |
| `--reporter=list` | `--reporter=list` (or `term`) | Terminal list reporter. |
| `--reporter=json` | `--reporter=json` | JSON reporter writes `.jet/test-results.json`. |
| `--workers=N` | `--workers=N` | Parallel worker count. |
| `--shard=i/N` | `--shard=i/N` | Shard index / total (same format). |
| `--output=<dir>` | `--report-dir=<dir>` | Report output directory. |
| `--trace=on` | `--trace=on` | Trace capture mode (`on`, `retain-on-failure`, `off`). |
| `--grep=<pattern>` | `--grep=<pattern>` | Filter tests by name regex. |
| `--timeout=<ms>` | `--timeout=<ms>` | Per-test timeout in milliseconds. |
| `--update-snapshots` | `--update-snapshots` (`-u`) | Overwrite snapshot files on mismatch. |

### Incompatible native-only flags with `--playwright`

The following flags are **not forwarded** to the Playwright subprocess and
produce a hard error (exit code 2) if combined with `--playwright`:

- `--reporter`
- `--trace`
- `--workers`
- `--shard`
- `--report-dir`

## Rewriting `@playwright/test` Imports

### Before (Playwright)

```typescript
import { test, expect, Page } from '@playwright/test';

test('homepage loads', async ({ page }: { page: Page }) => {
  await page.goto('http://localhost:3000');
  await expect(page).toHaveTitle('My App');
});
```

### After (jet native runner — browser fixtures)

```typescript
import { test, expect } from 'jet/test';

test('homepage loads', async ({ page }) => {
  await page.goto('http://localhost:3000');
  await expect(page).toHaveTitle('My App');
});
```

> Browser fixture support (`page`, `browser`, `context`) is available in
> jet native runner Phase 4+.

### Pure unit / API tests (no browser)

```typescript
// Before
import { test, expect } from '@playwright/test';

// After — drop-in replacement for non-browser tests
import { test, expect } from 'jet/test';

test('adds numbers', () => {
  expect(1 + 1).toBe(2);
});
```

## Using the HTML Reporter

After migration, generate an HTML report with:

```bash
jet test --reporter=html --report-dir=./test-results/report
```

Open it:

```bash
jet report view ./test-results/report
```

## Deep-Linking from HTML Report into `jet trace view`

When trace capture is enabled (`--trace=on` or `--trace=retain-on-failure`),
each failed test in the HTML report includes a link to open the trace archive
directly in the jet trace viewer.

From the command line:

```bash
jet trace view ./test-results/traces/my-test.zip
```

This starts a local HTTP server and opens the trace viewer in your browser.
The viewer shows network requests, DOM snapshots, and console output at each
step of the test.

## Step-by-Step Migration

1. **Audit your specs**: Find all files importing `@playwright/test`:

   ```bash
   grep -r "@playwright/test" tests/ --include="*.spec.ts" -l
   ```

2. **Use the escape hatch** during migration:

   ```bash
   jet test --playwright tests/legacy.spec.ts
   ```

3. **Rewrite imports** file by file using the table above.

4. **Run with the native runner** (no `--playwright`):

   ```bash
   jet test tests/migrated.spec.ts
   ```

5. **Remove `--playwright`** from CI once all specs are migrated.

## Seeking Help

- Open an issue: <https://github.com/cclab/jet/issues>
- Migration guide updates: <https://github.com/cclab/jet/blob/main/projects/jet/docs/migration-from-playwright.md>
