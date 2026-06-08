---
change: fix
group: jet-bundler-fixes
date: 2026-03-12
---

# Requirements

Fix 4 jet bundler transformer/module bugs discovered by the expanded mini-react example:

1. **Spread props transform** (transform_tsx.rs): `<C {...obj} extra={v}/>` must merge spread into the props object — emit `Object.assign({}, obj, {extra: v})` or `{...obj, extra: v}` instead of `...obj` as a rest arg to createElement.

2. **Barrel re-export resolution** (modules.rs): `export { X } from './X'` must emit `require()` call to resolve the source module — currently emits bare variable reference without import.

3. **Default + named export conflict** (modules.rs): `export default V` must not clobber `module.exports` when named exports exist — use `module.exports["default"] = V` only, don't also set `module.exports = V`.

4. **Dynamic import() in CJS runtime** (modules.rs or bundler/mod.rs): `import('./path')` must be transformed to work with `__jet__` runtime — either eager-resolve to `Promise.resolve(__jet__.require(id))` or implement async chunk loading.

Acceptance: All 13 Playwright E2E tests pass on both Vite and Jet builds.
