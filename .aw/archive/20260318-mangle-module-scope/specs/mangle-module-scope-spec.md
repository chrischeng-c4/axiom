---
id: mangle-module-scope-spec
main_spec_ref: "cclab-jet/mangle-module-scope.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios]
fill_sections: [overview, requirements, scenarios]
create_complete: true
---

# Mangle Module Scope Spec

## Overview

The `cclab-jet` bundler currently produces bundles around 215KB for `react-bench`, leaving a ~15KB size gap compared to webpack (192KB). This gap is primarily due to the minifier's inability to rename variables that reside inside per-module IIFE wrappers (e.g., `!function(module,exports,require){...}()`). For example, identifiers like `workInProgress` appear hundreds of times.

This change extends the bundler to support module-scope variable mangling and true module flattening (scope hoisting). By descending into per-module scopes to mangle variables and merging these scopes into a single flat function scope (with collision-avoidance), we can eliminate overhead from `require()` calls and repetitive variable names. The final goal is to bring the `react-bench` bundle size to ≤ 196KB, passing all tests and verifying cross-module constant inlining/DCE.
## Requirements

### R1: Module-Scope Mangling
The mangler (`mangle.rs`) MUST descend into per-module IIFE wrappers (e.g., `!function(module,exports,require){...}()`) and rename top-level identifiers within those local scopes. This ensures identifiers like `workInProgress` are mangled globally.

### R2: Module Flattening (Scope Hoisting)
The scope hoist module (`scope_hoist.rs`) MUST merge all per-module IIFE wrappers into a single flat function scope when possible.

### R3: Collision-Avoiding Renaming
When flattening modules, each module's top-level variables MUST be renamed using a collision-avoiding prefix (e.g., `_m0_foo`, `_m1_bar`).

### R4: Direct Export References
Calls to `require(N)` within flattened modules MUST be replaced with direct references to the renamed exported variables of the required module.

### R5: Bailout Conditions
Flattening MUST be skipped for modules that contain `eval()`, `with` statements, or dynamic `arguments` access.

### R6: Follow-on Sub-Phases (Optional/Stretch)
The implementation MAY optionally support cross-module constant inlining (inlining unreassigned `const` values at import sites) and cross-module Dead Code Elimination (DCE) for functions only reachable from already eliminated code.

### R7: Performance and Correctness Criteria
The bundle size for `react-bench` MUST drop to ≤ 196KB. All 126 existing bundler tests MUST pass. `mini-react` Playwright tests MUST pass on both Vite and Jet builds.
## Scenarios

### Scenario: Mangle variables inside IIFE wrappers
- **WHEN** a module is wrapped in a standard `!function(module,exports,require){...}()` IIFE and contains long local variables like `workInProgress`
- **THEN** the mangler descends into the IIFE scope and renames `workInProgress` to a short identifier (e.g., `a`).

### Scenario: Scope hoisting flattens standard modules
- **WHEN** multiple modules without bailout conditions require each other
- **THEN** their scopes are merged into a single flat function, top-level variables are renamed with module-specific prefixes (e.g., `_m1_foo`), and `require()` calls are replaced with direct references.

### Scenario: Scope hoisting bailout
- **WHEN** a module contains a dynamic `eval()`, `with` block, or dynamic `arguments` usage
- **THEN** scope hoisting is skipped for that module, and it retains its own isolated scope/wrapper to avoid semantic breakage.

### Scenario: React-bench bundle size drop
- **WHEN** bundling `react-bench` with the updated Jet bundler
- **THEN** the final output size is ≤ 196KB, saving approximately ~15KB through module-scope mangling and scope hoisting.
## Diagrams

## API Spec

# Reviews
