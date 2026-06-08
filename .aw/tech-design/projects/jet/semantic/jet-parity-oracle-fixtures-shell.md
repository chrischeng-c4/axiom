---
id: semantic-jet-parity-oracle-fixtures-shell
summary: Semantic coverage for "projects/jet/parity/oracle/fixtures/__shell__"
fill_sections: [logic, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/oracle/fixtures/__shell__

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: frontend-jet-parity-oracle-fixtures-shell-logic
entry: preserve_frontend_behavior
nodes:
  preserve_frontend_behavior:
    kind: start
    label: "Preserve observed frontend source behavior"
  classify_frontend_ast:
    kind: process
    label: "Map FE ecosystem AST group \"projects/jet/parity/oracle/fixtures/__shell__\" to logic emitter primitives"
  generator_gap:
    kind: terminal
    label: "Promote from semantic coverage to deterministic frontend codegen"
edges:
  - { from: preserve_frontend_behavior, to: classify_frontend_ast }
  - { from: classify_frontend_ast, to: generator_gap }
---
flowchart TD
  preserve_frontend_behavior --> classify_frontend_ast --> generator_gap
```

<!-- frontend_source_evidence
- projects/jet/parity/oracle/fixtures/__shell__/build.mjs
-->

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/oracle/fixtures/__shell__/build.mjs"
    action: modify
    section: logic
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
