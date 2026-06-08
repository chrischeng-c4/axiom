---
change: jet-remaining
group: bundle-optimization-hoisting
date: 2026-03-19
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Flattening Strategy
- **Question**: Specs conflict on flattening strategy (blocks vs. single scope) and mangling rules. Should we prioritize the 'true flattening' and prefix-renaming proposed in #903 to reach the 196KB target?
- **Answer**: Proceed with module-prefixed renaming (_m0_foo) and single-scope flattening. Skip flattening only for unsafe modules (eval, with, arguments). Use the more aggressive 196KB target.
- **Rationale**: To reach Webpack-level bundle size, we must eliminate all wrapper overhead and allow the mangler to compress module-level identifiers.

## Contradictions

### C1: scope-hoisting-spec vs requirement
- **Spec**: scope-hoisting-spec
- **Requirement**: True Module Flattening (Phase 2a)
- **Conflict**: scope-hoisting.md Phase 2 uses '{ }' blocks for scoping, while #903 Phase 2a proposes 'true module flattening' with no per-module wrappers (merging into a single function scope).
- **Resolution**: We will proceed with true module flattening (no blocks) by renaming all top-level variables with module prefixes to avoid collisions, as proposed in #903.

### C2: jet-variable-mangling-spec vs requirement
- **Spec**: jet-variable-mangling-spec
- **Requirement**: Mangler Visibility (Phase 2a)
- **Conflict**: variable-mangling.md R3 explicitly forbids mangling module-level variables, but #903 Phase 2a requires them to be module-prefixed and made visible to the mangler to achieve the size target.
- **Resolution**: Module-level variables will be renamed with prefixes and then mangled by the unified mangler to reach the ≤ 196KB target. variable-mangling.md R3 will be updated to allow mangling of these prefixed module-level variables.

### C3: jet-variable-mangling-spec vs requirement
- **Spec**: jet-variable-mangling-spec
- **Requirement**: Bundle Size Target
- **Conflict**: variable-mangling.md (R7) sets a bundle size target of ≤ 210KB, while #903 and #882 set a more aggressive target of ≤ 196KB (matching Webpack).
- **Resolution**: The target is ≤ 196KB for react-bench, as specified in the most recent issues #882 and #903.

