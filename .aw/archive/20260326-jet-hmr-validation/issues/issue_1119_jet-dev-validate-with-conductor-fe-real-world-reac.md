---
number: 1119
title: "jet dev: validate with Conductor FE (real-world React/TS project)"
state: open
labels: [priority:p1, crate:jet, project:conductor, type:test]
group: "jet-hmr-validation"
---

# #1119 — jet dev: validate with Conductor FE (real-world React/TS project)

## Problem

All jet-dev-server-v2 improvements (CJS pre-bundling, TS type stripping, polyfills, store nested deps) were implemented and unit-tested (586 tests) but never validated on a real project. Conductor FE is the dogfood target.

## Success Criteria

1. `cd projects/conductor/fe && cclab jet dev` starts without errors
2. Dashboard page renders correctly in browser
3. All `@cclab/ui` components display without syntax errors
4. API proxy to backend works (`dev.proxy` config)
5. CSS/Tailwind styles render correctly
6. No `Unexpected token` errors from CJS or TypeScript syntax in browser console
7. Hot reload works for CSS changes

## Steps

1. `cclab jet install` in Conductor FE (test nested deps fix #1092)
2. `cclab jet dev` — verify pre-bundling completes
3. Open browser — verify React app renders
4. Check console for errors — fix any remaining transform/polyfill gaps
5. Edit a component — verify CSS HMR works

## Blocking Issues

- None (all prerequisites implemented in jet-dev-server-v2)
