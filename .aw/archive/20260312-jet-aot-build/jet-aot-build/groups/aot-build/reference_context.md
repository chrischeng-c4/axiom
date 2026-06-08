---
change: jet-aot-build
group: aot-build
date: 2026-03-12
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| jet-jit-runner-spec | cclab-jet | medium | JitEngine uses Tree-sitter transform pipeline which AOT build will also use, TransformOptions and Transformer are shared infrastructure |
| jet-pkg-perf-spec | cclab-jet | low | PackageJson schema defines module/main fields used for entry point resolution, Bundler module graph and import resolution are existing infrastructure to build on |
| jet-pnpm-parity-spec | cclab-jet | low | Workspace discovery provides multi-entry-point context for code splitting |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: jet-aot-build

**Verdict**: APPROVED

### Summary

AOT build is a new spec area — no existing spec directly covers tree shaking, code splitting, minification, or source maps. The referenced specs provide correct background context: jit-runner-spec (medium) shares the Tree-sitter transform pipeline, pkg-manager specs (low) provide entry point resolution and workspace context. The primary implementation will build on existing bundler/ and transform/ modules which are code-level infrastructure, not spec-level.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - AOT build is a new area with no prior spec. Existing specs provide supporting context only, which is correct.
- ✅ Relevance scores are reasonable
  - jit-runner medium (shares transform infra), pkg-manager/pnpm-parity low (background) — accurate
- ✅ Key requirements listed per spec are accurate
- ✅ No irrelevant specs included

### Issues

No issues found.
