---
number: 796
title: "jet build: expand mini-react example with advanced JSX/TS patterns"
state: open
labels: [enhancement, crate:jet]
group: "jet-bundler-fixes"
---

# #796 — jet build: expand mini-react example with advanced JSX/TS patterns

## Summary

Expand the existing `examples/mini-react` TodoMVC to exercise more real-world patterns that the jet bundler must handle correctly.

## Patterns to Add

- [ ] Multi-file components with cross-imports and re-exports (`components/index.ts` barrel)
- [ ] Custom hooks in separate files (`hooks/useLocalStorage.ts`)
- [ ] Spread props: `<Component {...props} />`
- [ ] Conditional JSX: `{condition && <X/>}`, ternary `{a ? <X/> : <Y/>}`
- [ ] Fragment: `<></>`
- [ ] Dynamic `import()` (code splitting boundary)
- [ ] Advanced TS: generics, union types, utility types (`Partial<T>`, `Pick<T, K>`)
- [ ] Template literal in JSX: `` className={`foo ${bar}`} ``
- [ ] Array.map with JSX keys
- [ ] CSS modules or multiple CSS imports
- [ ] Default + named exports from same file

## Acceptance Criteria

- All new patterns build successfully with `cclab jet build`
- Playwright DOM snapshot tests pass for both Vite and Jet builds
- No new transformer bugs (or bugs found are fixed in the same PR)

## Context

Follow-up from #765 (AOT build). The current mini-react example is minimal — single component file, basic hooks. Expanding it will catch transformer edge cases before users hit them.
