---
change: jet-test-gaps
group: jet-test-gaps
date: 2026-03-27
---

# Requirements

Add Rust unit/integration tests to close the three largest coverage gaps in jet:

1. **CSS pipeline output validation** — verify Tailwind JIT generates correct CSS selectors and properties for utility classes (bg-blue-500, text-lg, flex, etc.), PostCSS @import chain resolution produces merged output, CSS module scoping renames classes correctly

2. **Dev server module graph** — test invalidation when a file changes (dependents are marked dirty), rebuild triggering after invalidation, circular dependency handling in the module graph, HMR boundary detection (self-accepting modules vs full reload)

3. **Code splitting multi-entry** — multiple entry points produce separate chunks with correct shared module extraction, async chunk preloading metadata, dynamic import() with circular dependencies falls back correctly

All tests are Rust #[test] functions in crates/cclab-jet/src/ (unit tests) or crates/cclab-jet/tests/ (integration tests). No E2E/Playwright tests in this change.
