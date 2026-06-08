---
id: implementation
type: change_implementation
change_id: enhancement-remove-playwright-test-dependency-from-e2e-harness
---

# Implementation

## Summary

Phase 5b of the Replace-Playwright epic: rewrote all 7 @playwright/test import lines in e2e/**/*.spec.ts and test-utils.ts to use @jet/test instead; deleted e2e/playwright.config.ts and ported its settings into a new jet.test.config.ts at repo root with three project blocks (vite-build, jet-build, jet-dev); removed @playwright/test from e2e/jet/package.json devDependencies and updated npm scripts from npx playwright to jet test; regenerated package-lock.json removing playwright, playwright-core, and @playwright/test entries; updated CHANGELOG.md line 26 to frame the import rewrite as legacy context; added [[test]] entry in crates/jet/Cargo.toml for a new Rust integration test (e2e_playwright_residue.rs) that walks e2e/ and asserts no .ts file contains @playwright/test, enforcing atomicity per R8.

## Diff

```diff
diff --git a/.score/issues/open/enhancement-remove-playwright-test-dependency-from-e2e-harness.md b/.score/issues/open/enhancement-remove-playwright-test-dependency-from-e2e-harness.md
index eee308c5..4c7180c8 100644
--- a/.score/issues/open/enhancement-remove-playwright-test-dependency-from-e2e-harness.md
+++ b/.score/issues/open/enhancement-remove-playwright-test-dependency-from-e2e-harness.md
@@ -7,8 +7,17 @@ labels:
 - crate:jet,priority:p2
 - type:enhancement
 created_at: 2026-04-21T03:32:51.703054+00:00
-updated_at: 2026-04-21T03:38:21.380693+00:00
-phase: merged
+updated_at: 2026-04-21T08:14:22.317977+00:00
+phase: change_implementation_created
+branch: cclab/enhancement-remove-playwright-test-dependency-from-e2e-harness
+git_workflow: worktree
+change_id: enhancement-remove-playwright-test-dependency-from-e2e-harness
+iteration: 1
+current_task_id: enhancement-remove-playwright-test-dependency-from-e2e-harness-spec
+impl_spec_phase:
+  enhancement-remove-playwright-test-dependency-from-e2e-harness-spec: code
+task_revisions: {}
+revision_counts: {}
 ---
 
 
@@ -24,6 +33,12 @@ phase: merged
 
 
 
+
+
+
+
+
+
 
 ## Problem
 
diff --git a/CHANGELOG.md b/CHANGELOG.md
index b61ccfea..9faa0576 100644
--- a/CHANGELOG.md
+++ b/CHANGELOG.md
@@ -23,7 +23,7 @@ The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
   subsequent minor release** (planned v0.7.x).
 
   **Migration**: See `crates/jet/docs/migration-from-playwright.md` for the
-  flag mapping table, `@playwright/test` import rewrite recipes, and
+  flag mapping table, import rewrite recipes (from @playwright/test to @jet/test, legacy), and
   trace-viewer / HTML-reporter deep-link usage.
 
   **Incompatible flags**: `--reporter`, `--trace`, `--workers`, `--shard`,
diff --git a/crates/jet/Cargo.toml b/crates/jet/Cargo.toml
index 9c162942..d1799506 100644
--- a/crates/jet/Cargo.toml
+++ b/crates/jet/Cargo.toml
@@ -106,3 +106,7 @@ open = "5"
 [dev-dependencies]
 tracing-subscriber = { workspace = true }
 which = "7"
+
+[[test]]
+name = "e2e_playwright_residue"
+path = "tests/e2e_playwright_residue.rs"
diff --git a/e2e/grid/app.spec.ts b/e2e/grid/app.spec.ts
index c2b17a73..47dee1f7 100644
--- a/e2e/grid/app.spec.ts
+++ b/e2e/grid/app.spec.ts
@@ -1,4 +1,4 @@
-import { test, expect } from '@playwright/test';
+import { test, expect } from '@jet/test';
 
 test.describe('RuSheet Application', () => {
   test.beforeEach(async ({ page }) => {
diff --git a/e2e/grid/cell-editing.spec.ts b/e2e/grid/cell-editing.spec.ts
index 924c7d35..005b5382 100644
--- a/e2e/grid/cell-editing.spec.ts
+++ b/e2e/grid/cell-editing.spec.ts
@@ -1,4 +1,4 @@
-import { test, expect } from '@playwright/test';
+import { test, expect } from '@jet/test';
 
 test.describe('Cell Editing', () => {
   test.beforeEach(async ({ page }) => {
diff --git a/e2e/jet/package-lock.json b/e2e/jet/package-lock.json
index ad467f29..0b587823 100644
--- a/e2e/jet/package-lock.json
+++ b/e2e/jet/package-lock.json
@@ -8,7 +8,6 @@
       "name": "mini-react-todomvc",
       "version": "0.0.0",
       "devDependencies": {
-        "@playwright/test": "^1.49.0",
         "typescript": "^5.7.0",
         "vite": "^6.0.0"
       }
@@ -455,22 +454,6 @@
         "node": ">=18"
       }
     },
-    "node_modules/@playwright/test": {
-      "version": "1.58.2",
-      "resolved": "https://registry.npmjs.org/@playwright/test/-/test-1.58.2.tgz",
-      "integrity": "sha512-akea+6bHYBBfA9uQqSYmlJXn61cTa+jbO87xVLCWbTqbWadRVmhxlXATaOjOgcBaWU4ePo0wB41KMFv3o35IXA==",
-      "dev": true,
-      "license": "Apache-2.0",
-      "dependencies": {
-        "playwright": "1.58.2"
-      },
-      "bin": {
-        "playwright": "cli.js"
-      },
-      "engines": {
-        "node": ">=18"
-      }
-    },
     "node_modules/@rollup/rollup-android-arm-eabi": {
       "version": "4.59.0",
       "resolved": "https://registry.npmjs.org/@rollup/rollup-android-arm-eabi/-/rollup-android-arm-eabi-4.59.0.tgz",
@@ -942,38 +925,6 @@
         "url": "https://github.com/sponsors/jonschlinkert"
       }
     },
-    "node_modules/playwright": {
-      "version": "1.58.2",
-      "resolved": "https://registry.npmjs.org/playwright/-/playwright-1.58.2.tgz",
-      "integrity": "sha512-vA30H8Nvkq/cPBnNw4Q8TWz1EJyqgpuinBcHET0YVJVFldr8JDNiU9LaWAE1KqSkRYazuaBhTpB5ZzShOezQ6A==",
-      "dev": true,
-      "license": "Apache-2.0",
-      "dependencies": {
-        "playwright-core": "1.58.2"
-      },
-      "bin": {
-        "playwright": "cli.js"
-      },
-      "engines": {
-        "node": ">=18"
-      },
-      "optionalDependencies": {
-        "fsevents": "2.3.2"
-      }
-    },
-    "node_modules/playwright-core": {
-      "version": "1.58.2",
-      "resolved": "https://registry.npmjs.org/playwright-core/-/playwright-core-1.58.2.tgz",
-      "integrity": "sha512-yZkEtftgwS8CsfYo7nm0KE8jsvm6i/PTgVtB8DL726wNf6H2IMsDuxCpJj59KDaxCtSnrWan2AeDqM7JBaultg==",
-      "dev": true,
-      "license": "Apache-2.0",
-      "bin": {
-        "playwright-core": "cli.js"
-      },
-      "engines": {
-        "node": ">=18"
-      }
-    },
     "node_modules/postcss": {
       "version": "8.5.8",
       "resolved": "https://registry.npmjs.org/postcss/-/postcss-8.5.8.tgz",
diff --git a/e2e/jet/package.json b/e2e/jet/package.json
index ba9eec82..7066d2b4 100644
--- a/e2e/jet/package.json
+++ b/e2e/jet/package.json
@@ -7,12 +7,11 @@
     "dev:vite": "vite",
     "build:vite": "vite build --outDir dist-vite",
     "preview:vite": "vite preview --outDir dist-vite --port 4174",
-    "test": "npx playwright test",
-    "test:ui": "npx playwright test --ui"
+    "test": "jet test",
+    "test:ui": "jet test --ui"
   },
   "devDependencies": {
     "vite": "^6.0.0",
-    "typescript": "^5.7.0",
-    "@playwright/test": "^1.49.0"
+    "typescript": "^5.7.0"
   }
 }
diff --git a/e2e/jet/tests/build.spec.ts b/e2e/jet/tests/build.spec.ts
index 810cd9c4..3f5dd024 100644
--- a/e2e/jet/tests/build.spec.ts
+++ b/e2e/jet/tests/build.spec.ts
@@ -10,7 +10,7 @@
  *   3. Test:        npx playwright test
  */
 
-import { test, expect, Page } from "@playwright/test";
+import { test, expect, Page } from "@jet/test";
 
 // --- Helpers ---
 
diff --git a/e2e/jet/tests/css.spec.ts b/e2e/jet/tests/css.spec.ts
index 33b6b02b..3266e812 100644
--- a/e2e/jet/tests/css.spec.ts
+++ b/e2e/jet/tests/css.spec.ts
@@ -12,7 +12,7 @@
  *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/css.spec.ts
  */
 
-import { test, expect } from "@playwright/test";
+import { test, expect } from "@jet/test";
 import { readFixture, writeFixture } from "./test-utils";
 
 test.describe("CSS Pipeline", () => {
diff --git a/e2e/jet/tests/dev-server.spec.ts b/e2e/jet/tests/dev-server.spec.ts
index 69178670..300e806c 100644
--- a/e2e/jet/tests/dev-server.spec.ts
+++ b/e2e/jet/tests/dev-server.spec.ts
@@ -14,7 +14,7 @@
  *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/dev-server.spec.ts
  */
 
-import { test, expect } from "@playwright/test";
+import { test, expect } from "@jet/test";
 
 test.describe("Jet Dev Server", () => {
   test("TypeScript type stripping — no Unexpected token errors", async ({
diff --git a/e2e/jet/tests/hmr.spec.ts b/e2e/jet/tests/hmr.spec.ts
index d4c60810..156c2280 100644
--- a/e2e/jet/tests/hmr.spec.ts
+++ b/e2e/jet/tests/hmr.spec.ts
@@ -16,7 +16,7 @@
  *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/hmr.spec.ts
  */
 
-import { test, expect } from "@playwright/test";
+import { test, expect } from "@jet/test";
 import { readFixture, writeFixture, addTodo } from "./test-utils";
 
 test.describe("Jet HMR", () => {
diff --git a/e2e/jet/tests/test-utils.ts b/e2e/jet/tests/test-utils.ts
index 301a2c6b..007d6db6 100644
--- a/e2e/jet/tests/test-utils.ts
+++ b/e2e/jet/tests/test-utils.ts
@@ -1,6 +1,6 @@
 import * as fs from "node:fs";
 import * as path from "node:path";
-import { Page } from "@playwright/test";
+import { Page } from "@jet/test";
 
 export const FIXTURE_DIR = path.resolve(__dirname, "..");
 
diff --git a/e2e/playwright.config.ts b/e2e/playwright.config.ts
deleted file mode 100644
index fc5b559c..00000000
--- a/e2e/playwright.config.ts
+++ /dev/null
@@ -1,34 +0,0 @@
-import { defineConfig } from "@playwright/test";
-
-export default defineConfig({
-  testDir: ".",
-  timeout: 30_000,
-  retries: 0,
-  outputDir: "test-results",
-  use: {
-    headless: true,
-    trace: "on-first-retry",
-    screenshot: "only-on-failure",
-  },
-  projects: [
-    {
-      name: "vite-build",
-      use: { baseURL: "http://localhost:4174" },
-      testMatch: "**/build.spec.ts",
-    },
-    {
-      name: "jet-build",
-      use: { baseURL: "http://localhost:4175" },
-      testMatch: "**/build.spec.ts",
-    },
-    {
-      name: "jet-dev",
-      use: { baseURL: "http://localhost:3000" },
-      testMatch: [
-        "**/dev-server.spec.ts",
-        "**/hmr.spec.ts",
-        "**/css.spec.ts",
-      ],
-    },
-  ],
-});
---NEW FILES---
=== crates/jet/tests/e2e_playwright_residue.rs ===
//! @spec .score/changes/enhancement-remove-playwright-test-dependency-from-e2e-harness/specs/enhancement-remove-playwright-test-dependency-from-e2e-harness-spec.md#R8

use std::fs;
use std::path::Path;

// REQ: R1
// REQ: R8
#[test]
fn e2e_playwright_residue_absent() {
    // Walk e2e/ recursively; assert no .ts file contains '@playwright/test'
    let repo_root = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|p| p.join("Cargo.toml").is_file() && p.join("e2e").is_dir())
        .expect("find repo root")
        .to_path_buf();

    let e2e = repo_root.join("e2e");
    let mut offenders: Vec<String> = Vec::new();
    walk(&e2e, &mut offenders);
    assert!(
        offenders.is_empty(),
        "Found @playwright/test residue:\n{}",
        offenders.join("\n")
    );
}

fn walk(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|s| s.to_str()) == Some("node_modules") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (lineno, line) in content.lines().enumerate() {
                        if line.contains("@playwright/test") {
                            out.push(format!("{}:{}: {}", path.display(), lineno + 1, line.trim()));
                        }
                    }
                }
            }
        }
    }
}

=== jet.test.config.ts ===
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

```

## Review: enhancement-remove-playwright-test-dependency-from-e2e-harness-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: enhancement-remove-playwright-test-dependency-from-e2e-harness

**Summary**: Implementation matches all spec requirements R1-R9. C1-C7 rewrote all 7 spec files from '@playwright/test' to '@jet/test'. C8 deleted e2e/playwright.config.ts. C9 created jet.test.config.ts at repo root with all ported settings (testDir, timeout=30000, headless, trace=retain-on-failure, screenshot=only-on-failure, reporter=[['html']], three project blocks). C10 removed @playwright/test from e2e/jet/package.json devDependencies and changed npx playwright scripts to 'jet test'. C11 refreshed package-lock.json via npm install --package-lock-only — removed all playwright entries. C16 updated CHANGELOG.md line 26. C18 created crates/jet/tests/e2e_playwright_residue.rs with #[test] function e2e_playwright_residue_absent, registered in Cargo.toml. Atomicity verified: grep -rn '@playwright/test' e2e/ returns zero matches. The residue test passes. Hard checklist: (1) code matches spec ✓, (2) Test Plan + #[test] block present in e2e_playwright_residue.rs ✓, (3) residue test passes and no existing tests modified ✓. Compat shim (playwright_shim.rs, --playwright flag) correctly left untouched per R4.



## Alignment Warnings

6 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'REST API' (type: rest-api) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'Async API' (type: async-api) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'Config' (type: config) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-remove-playwright-test-dependency-from-e2e-harness/.score/tech_design/.score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
