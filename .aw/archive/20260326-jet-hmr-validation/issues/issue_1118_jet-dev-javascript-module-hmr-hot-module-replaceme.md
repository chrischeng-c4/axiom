---
number: 1118
title: "jet dev: JavaScript module HMR (hot module replacement)"
state: open
labels: [type:enhancement, priority:p1, crate:jet]
group: "jet-hmr-validation"
---

# #1118 — jet dev: JavaScript module HMR (hot module replacement)

## Problem

Jet dev server has CSS HMR (via Tailwind rebuild + WebSocket push) but no JavaScript module HMR. When editing a `.tsx`/`.ts`/`.jsx` file, the browser must full-reload to see changes. This makes the dev experience significantly slower than Vite.

## Success Criteria

1. Editing a React component `.tsx` file triggers a partial update without full page reload
2. React component state is preserved across edits (via React Fast Refresh integration)
3. HMR boundary detection: if a module has no HMR boundary, propagate up to the nearest boundary or full-reload
4. HMR client runtime injected via `/__jet_hmr` WebSocket protocol (already exists for CSS)
5. `import.meta.hot` API available in user code for custom HMR handling

## Boundary Conditions

- CSS changes → existing CSS HMR path (already works)
- JS/TS/JSX/TSX changes → new JS HMR path
- Non-component files (utils, constants) → propagate to importing components or full-reload
- Syntax errors → overlay error display, no reload, recover on fix
- New file added → detected by watcher, no special handling needed
- File deleted → full-reload (can't hot-remove)

## Implementation Notes

- Extend existing `/__jet_hmr` WebSocket with JS update messages
- Inject HMR client runtime into served `index.html`
- React Fast Refresh: inject `$RefreshReg$` / `$RefreshSig$` calls during transform_tsx
- Module graph tracking: know which modules import which, to propagate invalidation
- Partial re-transform: only re-serve the changed module, browser re-imports it

## References

- Vite HMR: https://vitejs.dev/guide/api-hmr.html
- React Fast Refresh: https://github.com/facebook/react/tree/main/packages/react-refresh
