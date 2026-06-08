---
id: projects-jet-logic-parity-dom-reference-runner-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# parity-dom-reference-runner

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/parity-dom-reference-runner.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

<!-- HANDWRITE-BEGIN gap="missing-generator:hand-written:69bbb816" tracker="pending-tracker" reason="Mirror logic doc. Cites this spec, summarises the five-channel
capture order, the determinism contract, and the artifact layout.
Lives in the logic tree so future cross-channel specs (#2151 pixel
tolerance, #2160 CDP AX, #2167 pointer comparator, #2174 IME replay)
can `$ref` it without dereferencing the per-issue spec.
" -->
TODO: hand-write content for `.aw/tech-design/projects/jet/logic/parity-dom-reference-runner.md`.
<!-- HANDWRITE-END -->
