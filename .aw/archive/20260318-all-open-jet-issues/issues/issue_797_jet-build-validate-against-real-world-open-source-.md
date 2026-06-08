---
number: 797
title: "jet build: validate against real-world open-source React apps"
state: open
labels: [enhancement, crate:jet]
group: "jet-build-real-world-validation"
---

# #797 — jet build: validate against real-world open-source React apps

## Summary

Validate jet build correctness by building real-world open-source React applications and comparing output with Vite/webpack.

## Candidate Projects

| Project | Size | Why |
|---------|------|-----|
| [TodoMVC React (nicknisi)](https://github.com/nicknisi/todomvc-react) | Small | Standard benchmark, TS + hooks |
| [Realworld React+Redux](https://github.com/gothinkster/react-redux-realworld-example-app) | Medium | Routing, API layer, Redux, real app structure |
| [cal.com](https://github.com/calcom/cal.com) (subset) | Large | Monorepo, complex TS, tRPC |

## Approach

1. Clone project → `cclab jet install` → `cclab jet build`
2. Fix any bundler/transformer errors found
3. Serve both jet and original (Vite/CRA) builds
4. Compare via Playwright smoke tests (page loads, basic navigation, no console errors)

## Acceptance Criteria

- At least 2 real-world projects build and run successfully with jet
- No JS runtime errors in the built output
- Document any unsupported patterns as known limitations

## Context

Follow-up from #765 (AOT build) and #796 (expanded mini-react). After synthetic pattern coverage (#796), real-world validation catches integration issues that synthetic tests miss (e.g., node_modules resolution, complex re-exports, CSS-in-JS).
