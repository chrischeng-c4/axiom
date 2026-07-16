---
id: apps-courier-build-rs
summary: Lossless rust-source-unit coverage for `apps/courier/build.rs`.
fill_sections: [changes, schema]
capability_refs:
  - id: github-issues-proxy
    role: primary
    claim: github-issues-proxy-service
    coverage: full
---

# Fillback apps/courier/build.rs

## Schema
<!-- type: schema lang: yaml -->

```yaml
schemas: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
- action: modify
  anchor: main
  description: 'Lossless rust-source-unit ownership created from explicit file fillback.

    '
  impl_mode: hand-written
  path: apps/courier/build.rs
  section: schema
- action: modify
  anchor: trap
  description: |
    Project build entrypoint.
  impl_mode: hand-written
  path: apps/courier/build.sh
  section: schema
- action: modify
  anchor: say
  description: |
    Project installation entrypoint.
  impl_mode: hand-written
  path: apps/courier/install.sh
  section: schema
coverage_kind: semantic
```
