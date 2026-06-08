---
id: projects-jet-src-css-tailwind-config-rs
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Standardized projects/jet/src/css/tailwind/config.rs

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/jet/src/css/tailwind/config.rs
    action: modify
    section: design-token
    impl_mode: hand-written
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```
