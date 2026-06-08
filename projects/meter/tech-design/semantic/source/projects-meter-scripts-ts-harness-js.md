---
id: projects-meter-scripts-ts-harness-js
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/scripts/ts_harness.js

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/scripts/ts_harness.js` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `args` | projects/meter/scripts/ts_harness.js | constant | pub | 21 |  |
| `collectV8Metrics` | projects/meter/scripts/ts_harness.js | constant | pub | 24 |  |
| `findTestFiles` | projects/meter/scripts/ts_harness.js | function | pub | 42 | findTestFiles(pattern) |
| `fs` | projects/meter/scripts/ts_harness.js | constant | pub | 15 |  |
| `initialHeapStats` | projects/meter/scripts/ts_harness.js | constant | pub | 36 |  |
| `main` | projects/meter/scripts/ts_harness.js | function | pub | 113 | main() |
| `path` | projects/meter/scripts/ts_harness.js | constant | pub | 16 |  |
| `projectPath` | projects/meter/scripts/ts_harness.js | constant | pub | 22 |  |
| `results` | projects/meter/scripts/ts_harness.js | constant | pub | 27 |  |
| `runTest` | projects/meter/scripts/ts_harness.js | function | pub | 74 | runTest(testFile) |
| `testPattern` | projects/meter/scripts/ts_harness.js | constant | pub | 23 |  |
| `v8` | projects/meter/scripts/ts_harness.js | constant | pub | 18 |  |
| `{ glob }` | projects/meter/scripts/ts_harness.js | constant | pub | 17 |  |
## Source
<!-- type: source lang: javascript -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/scripts/ts_harness.js -->
````javascript
#!/usr/bin/env node
/**
 * Probe TypeScript Test Harness
 *
 * A lightweight test harness for running TypeScript/JavaScript tests
 * when no standard test framework (jest, vitest, mocha) is detected.
 *
 * Usage: node ts_harness.js <project_path> <test_pattern> [--v8-metrics]
 *
 * Output: JSON stream of test results
 */

const fs = require('fs');
const path = require('path');
const { glob } = require('glob') || { glob: null };
const v8 = require('v8');

// Parse arguments
const args = process.argv.slice(2);
const projectPath = args[0] || '.';
const testPattern = args[1] || '**/*.{test,spec}.{ts,tsx,js,jsx}';
const collectV8Metrics = args.includes('--v8-metrics');

// Simple test result structure
const results = {
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: [],
  v8_metrics: null,
};

// Collect initial V8 metrics
let initialHeapStats = null;
if (collectV8Metrics) {
  initialHeapStats = v8.getHeapStatistics();
}

// Simple glob implementation if glob package not available
async function findTestFiles(pattern) {
  if (typeof glob === 'function') {
    return await glob(pattern, { cwd: projectPath, absolute: true });
  }

  // Fallback: simple recursive search
  const files = [];
  const extensions = ['.test.ts', '.spec.ts', '.test.tsx', '.spec.tsx', '.test.js', '.spec.js'];

  function walkDir(dir) {
    try {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory() && !entry.name.startsWith('.') && entry.name !== 'node_modules') {
          walkDir(fullPath);
        } else if (entry.isFile()) {
          if (extensions.some(ext => entry.name.endsWith(ext))) {
            files.push(fullPath);
          }
        }
      }
    } catch (e) {
      // Ignore permission errors
    }
  }

  walkDir(projectPath);
  return files;
}

// Simple test execution
async function runTest(testFile) {
  const startTime = Date.now();
  const testName = path.relative(projectPath, testFile);

  try {
    // Try to require the test file
    // Note: This is a simplified approach - real implementation would need
    // ts-node or esbuild for TypeScript files
    if (testFile.endsWith('.ts') || testFile.endsWith('.tsx')) {
      // Skip TypeScript files without transpilation
      return {
        name: testName,
        status: 'skipped',
        duration_ms: 0,
        error: 'TypeScript files require ts-node or esbuild',
      };
    }

    // For JS files, we can require directly
    delete require.cache[require.resolve(testFile)];
    require(testFile);

    return {
      name: testName,
      status: 'passed',
      duration_ms: Date.now() - startTime,
    };
  } catch (error) {
    return {
      name: testName,
      status: 'failed',
      duration_ms: Date.now() - startTime,
      error: error.message,
      stack_trace: error.stack,
    };
  }
}

// Main execution
async function main() {
  const testFiles = await findTestFiles(testPattern);

  for (const testFile of testFiles) {
    const result = await runTest(testFile);
    results.tests.push(result);

    switch (result.status) {
      case 'passed':
        results.passed++;
        break;
      case 'failed':
        results.failed++;
        break;
      case 'skipped':
        results.skipped++;
        break;
    }
  }

  // Collect final V8 metrics
  if (collectV8Metrics && initialHeapStats) {
    const finalHeapStats = v8.getHeapStatistics();
    results.v8_metrics = {
      heap_total: finalHeapStats.total_heap_size,
      heap_used: finalHeapStats.used_heap_size,
      external: finalHeapStats.external_memory,
      gc_count: 0, // Would need performance hooks for accurate count
      gc_pause_ms: 0,
      event_loop_lag_ms: 0,
    };
  }

  // Output results as JSON
  console.log(JSON.stringify(results, null, 2));
}

main().catch(error => {
  console.error(JSON.stringify({ error: error.message }));
  process.exit(1);
});
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/scripts/ts_harness.js
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/scripts/ts_harness.js` captured during meter full-codegen standardization.
```
