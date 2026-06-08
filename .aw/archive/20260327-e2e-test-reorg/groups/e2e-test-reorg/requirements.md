---
change: e2e-test-reorg
group: e2e-test-reorg
date: 2026-03-26
---

# Requirements

Reorganize the E2E test infrastructure into a unified e2e/ directory structure using Playwright projects:

1. **Move grid tests**: e2e/app.spec.ts + cell-editing.spec.ts → e2e/grid/
2. **Move jet test app**: examples/mini-react/ → e2e/jet/ (entire app with src/, package.json, configs)
3. **Rename**: dom-snapshot.spec.ts → build.spec.ts
4. **Expand Playwright projects config**: vite-build (port 4174, build.spec.ts), jet-build (port 4175, build.spec.ts), jet-dev (port 3000, dev-server/hmr/css specs)
5. **Add new test files**:
   - dev-server.spec.ts: TS stripping, import.meta.env, path alias, proxy, Node.js polyfills
   - hmr.spec.ts: HMR module updates + React Fast Refresh state preservation
   - css.spec.ts: PostCSS pipeline + Tailwind CSS JIT in dev mode

Constraints:
- examples/jet/ and examples/react-bench/ stay in examples/ (different purpose)
- All jet E2E tests share the same mini-react TodoMVC app as fixture
- Playwright projects isolate build vs dev test scenarios
