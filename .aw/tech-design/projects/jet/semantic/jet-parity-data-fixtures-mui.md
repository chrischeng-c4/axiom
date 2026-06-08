---
id: semantic-jet-parity-data-fixtures-mui
summary: Semantic coverage for "projects/jet/parity/data/fixtures/mui"
fill_sections: [manifest, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/parity/data/fixtures/mui

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
frontend_semantic:
  section_type: "manifest"
  key: "jet/parity/data/fixtures/mui"
  source_group: "projects/jet/parity/data/fixtures/mui"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/data/fixtures/mui/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/jet/parity/data/fixtures/mui"
          workspace_root: "projects/jet/parity/data/fixtures/mui"
        frontend_node:
          workspace_root: "projects/jet/parity/data/fixtures/mui"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
  frontend_ast:
    nodes:
      - path: "projects/jet/parity/data/fixtures/mui/package.json"
        workspace_root: "projects/jet/parity/data/fixtures/mui"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/data/fixtures/mui/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
