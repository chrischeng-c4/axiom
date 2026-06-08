---
id: enhancement-remove-playwright-test-dependency-from-e2e-harness-spec
main_spec_ref: ".score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes, interaction, test-plan, cli, doc]
filled_sections: [overview, requirements, scenarios, changes, interaction, test-plan, cli, doc]
create_complete: true
review_verdict: APPROVED
---

# Enhancement Remove Playwright Test Dependency From E2e Harness Spec

## Overview
<!-- type: overview lang: markdown -->

Final deliverable of the Replace-Playwright epic. All `e2e/**/*.spec.ts` files and `e2e/jet/tests/test-utils.ts` currently import `@playwright/test`; this change rewrites every import to `@jet/test` (the native runner runtime shim at `crates/jet/runtime/test/index.ts`), deletes `e2e/playwright.config.ts`, ports its project-level settings into `jet.test.config.ts`, and removes `@playwright/test` from `e2e/jet/package.json`. The compat shim (`--playwright` flag, `crates/jet/src/test_runner/playwright_shim.rs`) is left untouched — only in-tree usage is removed. After this change `jet test` with no flags executes the full e2e suite under the native runner; `npx playwright` and `playwright install` are no longer referenced anywhere in-tree. Spec files affected: `e2e/jet/tests/{build,hmr,css,dev-server}.spec.ts`, `e2e/jet/tests/test-utils.ts`, `e2e/grid/{app,cell-editing}.spec.ts` (7 files). Atomic single PR — no intermediate state where some specs are migrated and Playwright still appears in `package.json`.
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "All imports of '@playwright/test' in e2e/jet/tests/{build,hmr,css,dev-server}.spec.ts, e2e/jet/tests/test-utils.ts, and e2e/grid/{app,cell-editing}.spec.ts must be replaced with 'import { test, expect } from \"@jet/test\"'. The Page type import in test-utils.ts must use the native Page proxy type exported from @jet/test."
  risk: high
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "'@playwright/test' must be absent from e2e/jet/package.json devDependencies after migration. pnpm-lock.yaml must be regenerated so no transitive playwright-core entry remains from the e2e/jet workspace."
  risk: high
  verifymethod: inspection
}

requirement R3 {
  id: R3
  text: "e2e/playwright.config.ts must be deleted. Its settings must be ported into jet.test.config.ts at repo root: testDir='.', timeout=30000, use.headless=true, use.trace='retain-on-failure', use.screenshot='only-on-failure', plus the three project-named baseURL blocks (vite-build=http://localhost:4174, jet-build=http://localhost:4175, jet-dev=http://localhost:3000) expressed as jet.test.config.ts project entries."
  risk: high
  verifymethod: inspection
}

requirement R4 {
  id: R4
  text: "crates/jet/src/test_runner/playwright_shim.rs and the --playwright CLI flag must not be modified or removed. The compat shim remains functional for out-of-tree users throughout the deprecation window."
  risk: high
  verifymethod: inspection
}

requirement R5 {
  id: R5
  text: "'jet test' with no flags must discover all 7 migrated spec files, execute under the native runner, and produce pass/fail parity with the pre-migration Playwright run (72 tests total: 22 build, 4 HMR, 3 CSS, 5 dev-server, 6 grid-app, 10 grid-cell-editing, plus any new build.spec.ts additions from Phase 4 baseline)."
  risk: high
  verifymethod: test
}

requirement R6 {
  id: R6
  text: "Any CI workflow step that calls 'playwright install', 'npx playwright test', or 'npx playwright' must be replaced with 'jet test' (or removed if the step is superseded). No CI step may reference @playwright/test or playwright-core after this change."
  risk: high
  verifymethod: inspection
}

requirement R7 {
  id: R7
  text: "Trace capture behaviour previously in playwright.config.ts use.trace='on-first-retry' must be re-expressed as use.trace='retain-on-failure' in jet.test.config.ts (semantically equivalent via the native --trace flag implemented in Phase 4a). The HTML reporter (Phase 4c) must be listed in jet.test.config.ts reporter array."
  risk: medium
  verifymethod: test
}

requirement R8 {
  id: R8
  text: "The PR must be a single atomic commit set: no intermediate commits where some spec files still import @playwright/test while others have been migrated. Enforced by a pre-commit grep check: 'grep -r @playwright/test e2e/ && exit 1 || exit 0'."
  risk: medium
  verifymethod: inspection
}

requirement R9 {
  id: R9
  text: "crates/jet/docs/migration-from-playwright.md must be updated to remove inline @playwright/test code examples that imply it is a current in-tree dependency. The migration guide may retain references framed as legacy context ('previously used', 'before migration'). No other jet docs file may contain an uncommented @playwright/test import example after this change."
  risk: low
  verifymethod: inspection
}
```
## Scenarios
<!-- type: scenarios lang: markdown -->

```yaml
- id: S1
  given: "e2e/jet/tests/build.spec.ts line 13 reads: import { test, expect, Page } from '@playwright/test'"
  when: "sed -i '' 's|import { test, expect, Page } from .@playwright/test.|import { test, expect, Page } from "@jet/test"|' e2e/jet/tests/build.spec.ts"
  then: "File imports { test, expect, Page } from '@jet/test'; no @playwright/test string remains in file"

- id: S2
  given: "e2e/jet/tests/hmr.spec.ts line 19 reads: import { test, expect } from '@playwright/test'"
  when: "sed -i '' 's|import { test, expect } from .@playwright/test.|import { test, expect } from "@jet/test"|' e2e/jet/tests/hmr.spec.ts"
  then: "File imports { test, expect } from '@jet/test'; no @playwright/test string remains in file"

- id: S3
  given: "e2e/jet/tests/css.spec.ts line 15 reads: import { test, expect } from '@playwright/test'"
  when: "sed -i '' 's|import { test, expect } from .@playwright/test.|import { test, expect } from "@jet/test"|' e2e/jet/tests/css.spec.ts"
  then: "File imports { test, expect } from '@jet/test'; no @playwright/test string remains in file"

- id: S4
  given: "e2e/jet/tests/dev-server.spec.ts line 17 reads: import { test, expect } from '@playwright/test'"
  when: "sed -i '' 's|import { test, expect } from .@playwright/test.|import { test, expect } from "@jet/test"|' e2e/jet/tests/dev-server.spec.ts"
  then: "File imports { test, expect } from '@jet/test'; no @playwright/test string remains in file"

- id: S5
  given: "e2e/jet/tests/test-utils.ts line 3 reads: import { Page } from '@playwright/test'"
  when: "sed -i '' 's|import { Page } from .@playwright/test.|import { Page } from "@jet/test"|' e2e/jet/tests/test-utils.ts"
  then: "File imports Page type proxy from '@jet/test'; no @playwright/test string remains in file"

- id: S6
  given: "e2e/grid/app.spec.ts line 1 reads: import { test, expect } from '@playwright/test'"
  when: "sed -i '' \"s|import { test, expect } from '@playwright/test'|import { test, expect } from '@jet/test'|\" e2e/grid/app.spec.ts"
  then: "File imports { test, expect } from '@jet/test'; no @playwright/test string remains in file"

- id: S7
  given: "e2e/grid/cell-editing.spec.ts line 1 reads: import { test, expect } from '@playwright/test'"
  when: "sed -i '' \"s|import { test, expect } from '@playwright/test'|import { test, expect } from '@jet/test'|\" e2e/grid/cell-editing.spec.ts"
  then: "File imports { test, expect } from '@jet/test'; no @playwright/test string remains in file"

- id: S8
  given: "All 7 spec files have been migrated (S1-S7 complete); e2e/playwright.config.ts exists; jet.test.config.ts does not yet contain ported settings; @playwright/test still in e2e/jet/package.json devDependencies"
  when: "Atomic single commit: (1) delete e2e/playwright.config.ts, (2) write jet.test.config.ts with ported settings (testDir='.', timeout=30000, headless=true, trace='retain-on-failure', screenshot='only-on-failure', reporter=['html'], three project blocks with baseURLs), (3) remove @playwright/test from e2e/jet/package.json devDependencies and scripts, (4) run pnpm -C e2e/jet install to refresh pnpm-lock.yaml"
  then: "e2e/playwright.config.ts absent from repo; jet.test.config.ts contains all three project blocks; e2e/jet/package.json devDependencies has no @playwright/test key; pnpm-lock.yaml has no playwright-core entry from e2e/jet workspace; grep -r '@playwright/test' e2e/ exits 1 (no matches)"
  diagram_ref: interaction
```
## Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

## State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: interaction
---
sequenceDiagram
    actor Dev as Developer
    participant CLI as jet CLI
    participant Config as jet.test.config.ts
    participant Runner as Native Test Runner
    participant CDP as CDP Client
    participant Reporter as HTML Reporter

    Dev->>CLI: jet test
    CLI->>Config: load jet.test.config.ts from cwd
    Config-->>CLI: {testDir, timeout, projects, reporter, use:{headless,trace,screenshot}}
    CLI->>Runner: discover **/*.spec.ts under testDir
    Runner-->>CLI: [build.spec.ts, hmr.spec.ts, css.spec.ts, dev-server.spec.ts, test-utils.ts, grid/app.spec.ts, grid/cell-editing.spec.ts]
    CLI->>Runner: execute project matrix (vite-build, jet-build, jet-dev)
    loop for each spec file in project
        Runner->>CDP: launch Chromium headless, baseURL per project
        CDP-->>Runner: Browser ready
        Runner->>Runner: import { test, expect } from '@jet/test'
        Runner->>Runner: run test fixtures (page, context)
        Runner->>CDP: page.goto, page.locator, expect assertions
        CDP-->>Runner: pass / fail per assertion
        Runner->>Reporter: emit test result event
    end
    Runner->>Reporter: finalize
    Reporter-->>Dev: HTML report at test-results/index.html
    Runner-->>CLI: exit 0 (all pass) or exit 1 (failures)
    CLI-->>Dev: pass/fail summary to stdout
```
## Logic
<!-- type: logic lang: mermaid -->
<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
```mermaid
---
id: logic
---
flowchart TD
    A([Start]) --> B{Decision}
```
-->

## Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: dependency
---
classDiagram
    class ComponentA
    class ComponentB
    ComponentA --> ComponentB
```
-->

## Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: db-model
---
erDiagram
    ENTITY {
        string id PK
    }
```
-->

## REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

## RPC API
<!-- type: rpc-api lang: yaml -->
<!-- TODO: OpenRPC 1.3 as YAML. Example:
```yaml
openrpc: "1.3.2"
info:
  title: Service Name
  version: "1.0.0"
methods: []
```
-->

## Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

## CLI
<!-- type: cli lang: yaml -->

```yaml
new_commands: []
modified_commands: []
note: "No new CLI surface. This change uses the existing 'jet test' command from Phase 4 parity. The jet.test.config.ts config file is discovered automatically by the existing config-loader logic — no new flags or subcommands are introduced."
```
## Schema
<!-- type: schema lang: yaml -->
<!-- TODO: JSON Schema as YAML. Example:
```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
type: object
properties:
  id:
    type: string
required: [id]
```
-->

## Config
<!-- type: config lang: yaml -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

```mermaid
---
id: test-plan
---
requirementDiagram

requirement R1 {
  id: R1
  text: "All 7 spec files import from @jet/test"
  risk: high
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "@playwright/test absent from e2e/jet/package.json"
  risk: high
  verifymethod: inspection
}

requirement R3 {
  id: R3
  text: "playwright.config.ts deleted; settings in jet.test.config.ts"
  risk: high
  verifymethod: inspection
}

requirement R5 {
  id: R5
  text: "jet test passes all 7 migrated specs"
  risk: high
  verifymethod: test
}

requirement R7 {
  id: R7
  text: "trace=retain-on-failure and HTML reporter in jet.test.config.ts"
  risk: medium
  verifymethod: test
}

requirement R8 {
  id: R8
  text: "No @playwright/test residue in e2e/ at any commit"
  risk: medium
  verifymethod: test
}

requirement R9 {
  id: R9
  text: "No active @playwright/test import examples in non-excluded docs"
  risk: low
  verifymethod: inspection
}

element T1 {
  type: "Test"
  docref: "cargo test -p jet --test e2e_playwright_residue"
}

element T2 {
  type: "Test"
  docref: "jet test e2e/jet/tests/build.spec.ts"
}

element T3 {
  type: "Test"
  docref: "jet test e2e/jet/tests/hmr.spec.ts"
}

element T4 {
  type: "Test"
  docref: "jet test e2e/jet/tests/css.spec.ts"
}

element T5 {
  type: "Test"
  docref: "jet test e2e/jet/tests/dev-server.spec.ts"
}

element T6 {
  type: "Test"
  docref: "jet test e2e/grid/app.spec.ts"
}

element T7 {
  type: "Test"
  docref: "jet test e2e/grid/cell-editing.spec.ts"
}

element T8 {
  type: "Test"
  docref: "pnpm -C e2e/jet install --frozen-lockfile"
}

element T9 {
  type: "Test"
  docref: "test -f jet.test.config.ts && ! test -f e2e/playwright.config.ts"
}

element T10 {
  type: "Test"
  docref: "jet test --project vite-build --project jet-build --project jet-dev"
}

T1 - verifies -> R1
T1 - verifies -> R8
T2 - verifies -> R5
T3 - verifies -> R5
T4 - verifies -> R5
T5 - verifies -> R5
T6 - verifies -> R5
T7 - verifies -> R5
T8 - verifies -> R2
T9 - verifies -> R3
T10 - verifies -> R7
```

BDD scenarios:

- T1: Given the migration is applied; When `cargo test -p jet --test e2e_playwright_residue` runs; Then exit 0 with message "0 files contain @playwright/test" — any remaining import causes exit 1 with offending file:line.
- T2: Given dev server not required; When `jet test e2e/jet/tests/build.spec.ts`; Then 22 tests pass across vite-build and jet-build projects.
- T3: Given dev server at localhost:3000; When `jet test e2e/jet/tests/hmr.spec.ts`; Then 4 HMR tests pass under jet-dev project.
- T4: Given dev server at localhost:3000; When `jet test e2e/jet/tests/css.spec.ts`; Then 3 CSS tests pass under jet-dev project.
- T5: Given no server required (server lifecycle tested internally); When `jet test e2e/jet/tests/dev-server.spec.ts`; Then 5 dev-server tests pass.
- T6: Given grid app served at baseURL; When `jet test e2e/grid/app.spec.ts`; Then 6 grid-app tests pass.
- T7: Given grid app served at baseURL; When `jet test e2e/grid/cell-editing.spec.ts`; Then 10 cell-editing tests pass.
- T8: Given C10 applied (playwright removed from package.json); When `pnpm -C e2e/jet install --frozen-lockfile`; Then exit 0 — lockfile is consistent; no playwright-core in e2e/jet workspace.
- T9: Given C8 and C9 applied; When `test -f jet.test.config.ts && ! test -f e2e/playwright.config.ts`; Then exit 0.
- T10: Given jet.test.config.ts project blocks; When `jet test --project vite-build --project jet-build --project jet-dev`; Then HTML report emitted to test-results/index.html with trace artifacts on failure.
## Changes
<!-- type: changes lang: yaml -->

```yaml
- id: C1
  file: e2e/jet/tests/build.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect, Page } from '@playwright/test'. After: import { test, expect, Page } from '@jet/test'. Page type is the native CDP Page proxy exported from @jet/test — no other lines change."

- id: C2
  file: e2e/jet/tests/hmr.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect } from '@playwright/test'. After: import { test, expect } from '@jet/test'."

- id: C3
  file: e2e/jet/tests/css.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect } from '@playwright/test'. After: import { test, expect } from '@jet/test'."

- id: C4
  file: e2e/jet/tests/dev-server.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect } from '@playwright/test'. After: import { test, expect } from '@jet/test'."

- id: C5
  file: e2e/jet/tests/test-utils.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { Page } from '@playwright/test'. After: import { Page } from '@jet/test'. Page is re-exported as the native CDP Page proxy type from @jet/test — callers in build.spec.ts continue to use Page without changes."

- id: C6
  file: e2e/grid/app.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect } from '@playwright/test'. After: import { test, expect } from '@jet/test'."

- id: C7
  file: e2e/grid/cell-editing.spec.ts
  op: edit
  req: R1
  description: "Replace import line. Before: import { test, expect } from '@playwright/test'. After: import { test, expect } from '@jet/test'."

- id: C8
  file: e2e/playwright.config.ts
  op: delete
  req: R3
  description: "Delete file entirely. Settings ported to jet.test.config.ts (C9). The defineConfig import from @playwright/test and all config content is removed."

- id: C9
  file: jet.test.config.ts
  op: create
  req: "R3, R7"
  description: |-
    Create new file at repo root with ported playwright.config.ts settings:
    - testDir: '.'
    - timeout: 30000
    - retries: 0
    - outputDir: 'test-results'
    - use.headless: true
    - use.trace: 'retain-on-failure'  (was 'on-first-retry' — semantically equivalent via native --trace flag)
    - use.screenshot: 'only-on-failure'
    - reporter: [['html']]
    - projects: three entries:
        { name: 'vite-build', use: { baseURL: 'http://localhost:4174' }, testMatch: '**/build.spec.ts' }
        { name: 'jet-build',  use: { baseURL: 'http://localhost:4175' }, testMatch: '**/build.spec.ts' }
        { name: 'jet-dev',    use: { baseURL: 'http://localhost:3000' }, testMatch: ['**/dev-server.spec.ts','**/hmr.spec.ts','**/css.spec.ts'] }
    Import: import { defineConfig } from '@jet/test'

- id: C10
  file: e2e/jet/package.json
  op: edit
  req: R2
  description: |-
    Three edits:
    (1) devDependencies: remove '@playwright/test': '^1.49.0' key entirely.
    (2) scripts.test: change 'npx playwright test' -> 'jet test'
    (3) scripts.test:ui: change 'npx playwright test --ui' -> 'jet test --ui'

- id: C11
  file: pnpm-lock.yaml
  op: edit
  req: R2
  description: "Regenerate via 'pnpm -C e2e/jet install' after C10. No manual edits — lockfile is a generated artifact. Post-regeneration: no playwright-core or @playwright/test entries in the e2e/jet workspace section."

- id: C12
  file: .github/workflows/docs.yml
  op: inspect
  req: R6
  description: "No playwright references found via grep. No change required."

- id: C13
  file: .github/workflows/miri.yml
  op: inspect
  req: R6
  description: "No playwright references found via grep. No change required."

- id: C14
  file: .github/workflows/release.yml
  op: inspect
  req: R6
  description: "No playwright references found via grep. No change required."

- id: C15
  file: .github/workflows/test-with-junit.yml
  op: inspect
  req: R6
  description: "No playwright references found via grep. No change required."

- id: C16
  file: CHANGELOG.md
  op: edit
  req: R9
  description: "Line 26 references '@playwright/test import rewrite recipes' in a changelog entry. Rewrite to: 'import rewrite recipes (from @playwright/test to @jet/test, legacy)'. Framed as historical context — not an active dependency claim."

- id: C17
  file: crates/jet/docs/migration-from-playwright.md
  op: no-change
  req: R9
  description: "Owned by the compat-shim sibling issue. All @playwright/test references in this file are intentional legacy-context framing. This change does not touch migration-from-playwright.md."

- id: C18
  file: crates/jet/tests/e2e_playwright_residue.rs
  op: create
  req: "R1, R8"
  description: |-
    New Rust integration test that greps the repo for any remaining @playwright/test import strings.
    Test function: e2e_playwright_residue_absent.
    Logic: walk e2e/ directory recursively, collect all .ts files, assert none contain the literal string '@playwright/test'. On failure, print the offending file path and line number. Registered under [[test]] in crates/jet/Cargo.toml as integration test 'e2e_playwright_residue'. Run with: cargo test -p jet --test e2e_playwright_residue
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: yaml -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: yaml -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

Documentation files that must be updated as part of this change (R9):

**CHANGELOG.md** (repo root)
- Line 26: `@playwright/test import rewrite recipes` — rewrite to `import rewrite recipes (from @playwright/test to @jet/test, legacy)` to frame as historical context rather than an active dependency claim.

**crates/jet/docs/migration-from-playwright.md** — NO CHANGE
- Owned by the compat-shim sibling issue. All references are intentionally framed as legacy migration guidance. This file is excluded per R9.

Files confirmed clean (no updates needed):
- `.github/workflows/docs.yml` — no playwright references
- `.github/workflows/miri.yml` — no playwright references
- `.github/workflows/release.yml` — no playwright references
- `.github/workflows/test-with-junit.yml` — no playwright references
- `crates/jet/docs/` contains only `migration-from-playwright.md` — excluded per R9

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: enhancement-remove-playwright-test-dependency-from-e2e-harness

**Verdict**: APPROVED

### Summary

Spec is implementation-ready. All 9 issue requirements (R1-R9) are addressed with concrete file-level changes (C1-C18). Test Plan lists 10 concrete commands (cargo test, jet test, pnpm) — satisfies the hard-reject rule. Scenarios S1-S8 cover per-file migration and atomic dependency drop. Interaction sequence diagram shows native runner flow. Changes section enumerates all 7 spec file rewrites + playwright.config.ts deletion + jet.test.config.ts creation + package.json + lockfile + CI inspection + docs + new residue-grep integration test (e2e_playwright_residue). Spec correctly leaves the compat shim (playwright_shim.rs, --playwright flag) untouched per R4. Atomicity enforced by T1 residue-grep test tied to R8.

### Issues

No issues found.
