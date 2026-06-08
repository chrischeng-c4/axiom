---
id: jet-remaining-issues-spec
main_spec_ref: "cclab/specs/jet-remaining-issues.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test_plan]
fill_sections: [overview, requirements, scenarios, test_plan]
create_complete: true
---

# Jet Remaining Issues Spec

## Overview

This specification addresses the remaining critical capabilities and bug fixes for the Jet build and package management tool. The scope includes implementing a full Ahead-Of-Time (AOT) production build pipeline to support tree shaking, code splitting, minification, source maps, and robust CSS/asset handling. Additionally, it implements advanced scope hoisting to reduce bundle sizes below those of Webpack.

On the package management side, it resolves outstanding bugs in the dependency resolver and introduces significant optimizations to cold install performance via disk metadata caching, HTTP/2 connection reuse, and speculative prefetching. Lastly, it removes the external dependency on the `nx` CLI by parsing `project.json` natively and adds real-world validation against production-scale React applications to ensure reliability.
## Requirements

### R1: AOT Production Build Pipeline
Implement a comprehensive build pipeline including tree shaking (ESM graph analysis, DCE), code splitting (dynamic imports, shared-chunks), minification (mangling, whitespace, CSS/HTML minification), source maps (VLQ-encoded, chained), CSS pipeline (PostCSS, Tailwind, modules), asset pipeline (images, fonts, JSON), and build configuration (env replacement, ES target).

### R2: Real-World Validation
Build "TodoMVC React" and "Realworld React+Redux" using Jet and validate via Playwright tests to ensure zero JS runtime errors.

### R3: Scope Hoisting
Implement Phase 1 (module concatenation for non-circular single-import modules) and Phase 2 (true module flattening, single function scope, cross-module constant inlining, and DCE). Achieve a `react-bench` bundle size of ≤ 196KB.

### R4: Resolver Bug Fixes
Correct the dependency resolver to handle version conflicts, OR range syntax (`||`), pre-release matching, space-separated ranges, `npm:` aliases, and optional dependencies. Ensure all `pkg_manager` tests pass.

### R5: Cold Install Performance
Optimize cold install to ≤ 3.0s by introducing a disk metadata cache (`~/.jet-store/.metadata/`), HTTP/2 connection reuse, speculative prefetching of transitive deps, and pipelined tarball downloading.

### R6: Direct Workspace Parsing
Eliminate the external `nx` CLI dependency; `jet` must read and parse `project.json` files directly during workspace discovery.
## Scenarios

### Scenario: Production AOT Build
- **WHEN** a user executes `jet build` on a real-world application like TodoMVC React
- **THEN** the AOT pipeline runs tree shaking, code splitting, minification, and CSS/asset processing to produce an optimized `dist/` directory, free of JS runtime errors.

### Scenario: Bundle Size Optimization
- **WHEN** building a project such as `react-bench` with `jet build`
- **THEN** Jet effectively flattens modules into a unified scope and eliminates dead code, achieving a total bundle size of ≤ 196KB.

### Scenario: Fast Cold Install
- **WHEN** a user runs `jet install` on a project with a completely unpopulated HTTP cache
- **THEN** it completes in under 3.0s, leveraging disk metadata caching, HTTP/2 connection pooling, and speculative prefetching without resolution errors.

### Scenario: Direct Nx Workspace Parsing
- **WHEN** Jet is executed within an Nx-managed monorepo environment
- **THEN** it resolves the workspace topology directly by reading `project.json` without spawning the `nx` CLI.
## Diagrams

## API Spec

## Test Plan

- **AOT Pipeline & Validation**: Execute Playwright smoke tests against `TodoMVC React` and `Realworld React+Redux` built with Jet. Verify `dist/` correctness and lack of console errors.
- **Scope Hoisting**: Build `react-bench` and verify the output JS bundle size is strictly ≤ 196KB. Run all 126 existing bundler tests to ensure full backward compatibility.
- **Resolver Logic**: Execute the 34 `pkg_manager` unit tests specifically targeting version conflicts, `||` syntax, optional deps, and `npm:` aliases.
- **Cold Install Benchmark**: Run a clean install script multiple times on a target benchmark application and enforce the median install time is ≤ 3.0s.
- **Workspace Discovery**: Provide a dummy Nx workspace fixture and assert that Jet correctly identifies projects strictly through file parsing, verifying no `nx` process starts.
# Reviews
