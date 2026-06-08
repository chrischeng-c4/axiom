---
id: semantic-jet-parity-data-fixtures-libraries
summary: Semantic coverage for "projects/jet/parity/data/fixtures/libraries"
fill_sections: [manifest, changes]
capability_refs:
  - id: browser-trace-parity
    role: primary
    gap: parity-corpus-gates
    claim: parity-corpus-gates
    coverage: partial
    rationale: "The library fixture manifests feed the live DOM/WASM parity corpus gate."
---

# Semantic TD: jet/parity/data/fixtures/libraries

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
frontend_semantic:
  section_type: "manifest"
  key: "jet/parity/data/fixtures/libraries"
  source_group: "projects/jet/parity/data/fixtures/libraries"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/parity/data/fixtures/libraries/fixtures.toml"
        language: "toml"
        ownership_state: "handwrite"
        generator_primitives: ["frontend_fixture-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "fixture-matrix"
          section_type: "manifest"
          domain: "projects/jet/parity/data/fixtures/libraries"
          workspace_root: "projects/jet/parity/data/fixtures/libraries"
        frontend_node:
          workspace_root: "projects/jet/parity/data/fixtures/libraries"
          role: "fixture-matrix"
          section_type: "manifest"
          artifact_kind: "fixture-manifest"
      - path: "projects/jet/parity/data/fixtures/libraries/package.json"
        language: "json"
        ownership_state: "handwrite"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "workspace-manifest"
          section_type: "manifest"
          domain: "projects/jet/parity/data/fixtures/libraries"
          workspace_root: "projects/jet/parity/data/fixtures/libraries"
        frontend_node:
          workspace_root: "projects/jet/parity/data/fixtures/libraries"
          role: "workspace-manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
  frontend_ast:
    nodes:
      - path: "projects/jet/parity/data/fixtures/libraries/fixtures.toml"
        workspace_root: "projects/jet/parity/data/fixtures/libraries"
        role: "fixture-matrix"
        artifact_kind: "fixture-manifest"
        section_type: "manifest"
      - path: "projects/jet/parity/data/fixtures/libraries/package.json"
        workspace_root: "projects/jet/parity/data/fixtures/libraries"
        role: "workspace-manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/parity/data/fixtures/libraries/fixtures.toml"
    action: create
    section: manifest
    description: |
      Third-party library fixture matrix for executable and planned DOM/WASM parity cases.
    impl_mode: hand-written
  - path: "projects/jet/parity/data/fixtures/libraries/package.json"
    action: create
    section: manifest
    description: |
      Dependency manifest for third-party React component library fixture references.
    impl_mode: hand-written
```
