---
id: semantic-jet-tests-fixtures-dom-production-build-react-bench
summary: Semantic coverage for "projects/jet/tests/fixtures/dom-production-build/react-bench"
capability_refs:
  - id: "rust-native-frontend-toolchain"
    role: primary
    claim: "production-replacement-readiness"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/jet/tests/fixtures/dom-production-build/react-bench`."
fill_sections: [manifest, config, changes]
---

# Semantic TD: jet/tests/fixtures/dom-production-build/react-bench

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
frontend_semantic:
  section_type: "manifest"
  key: "jet/tests/fixtures/dom-production-build/react-bench"
  source_group: "projects/jet/tests/fixtures/dom-production-build/react-bench"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/package.json"
        language: "json"
        ownership_state: "handwrite"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest", "test_case"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/jet/tests/fixtures/dom-production-build/react-bench"
          workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
  frontend_ast:
    nodes:
      - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/package.json"
        workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
```

## Config
<!-- type: config lang: yaml -->

```yaml
frontend_semantic:
  section_type: "config"
  key: "jet/tests/fixtures/dom-production-build/react-bench"
  source_group: "projects/jet/tests/fixtures/dom-production-build/react-bench"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/vite.config.ts"
        language: "typescript"
        ownership_state: "handwrite"
        generator_primitives: ["frontend_workspace-config", "td_section_config", "test_case"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "typescript-jsx"
          role: "config"
          section_type: "config"
          domain: "projects/jet/tests/fixtures/dom-production-build/react-bench"
          workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
        frontend_node:
          workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
  frontend_ast:
    nodes:
      - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/vite.config.ts"
        workspace_root: "projects/jet/tests/fixtures/dom-production-build/react-bench"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-fixtures-dom-production-build-react-bench-package-json>"
  - path: "projects/jet/tests/fixtures/dom-production-build/react-bench/vite.config.ts"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-jet-tests-fixtures-dom-production-build-react-bench-vite-config-ts>"
```
