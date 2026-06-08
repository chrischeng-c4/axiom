---
id: semantic-agentic-workflow-packages-sdd
summary: Semantic coverage for "projects/agentic-workflow/packages/@sdd"
fill_sections: [manifest, config, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This semantic TD covers AW core/client model source behavior and shared workflow domain primitives."
---

# Semantic TD: agentic-workflow/packages/@sdd

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
frontend_semantic:
  section_type: "manifest"
  key: "agentic-workflow/packages/@sdd"
  source_group: "projects/agentic-workflow/packages/@sdd"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/packages/@sdd/ui/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
      - path: "projects/agentic-workflow/packages/@sdd/pipeline/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
      - path: "projects/agentic-workflow/packages/@sdd/core/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
      - path: "projects/agentic-workflow/packages/@sdd/app/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
      - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/package.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-manifest", "td_section_manifest"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "manifest"
          section_type: "manifest"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "manifest"
          section_type: "manifest"
          artifact_kind: "workspace-manifest"
  frontend_ast:
    nodes:
      - path: "projects/agentic-workflow/packages/@sdd/ui/package.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
      - path: "projects/agentic-workflow/packages/@sdd/pipeline/package.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
      - path: "projects/agentic-workflow/packages/@sdd/core/package.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
      - path: "projects/agentic-workflow/packages/@sdd/app/package.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
      - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/package.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "manifest"
        artifact_kind: "workspace-manifest"
        section_type: "manifest"
```

## Config
<!-- type: config lang: yaml -->

```yaml
frontend_semantic:
  section_type: "config"
  key: "agentic-workflow/packages/@sdd"
  source_group: "projects/agentic-workflow/packages/@sdd"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/agentic-workflow/packages/@sdd/ui/tsconfig.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-config", "td_section_config"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "config"
          section_type: "config"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
      - path: "projects/agentic-workflow/packages/@sdd/pipeline/tsconfig.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-config", "td_section_config"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "config"
          section_type: "config"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
      - path: "projects/agentic-workflow/packages/@sdd/core/tsconfig.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-config", "td_section_config"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "config"
          section_type: "config"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
      - path: "projects/agentic-workflow/packages/@sdd/app/tsconfig.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-config", "td_section_config"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "config"
          section_type: "config"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
      - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/tsconfig.json"
        language: "json"
        ownership_state: "codegen"
        generator_primitives: ["frontend_workspace-config", "td_section_config"]
        source_evidence_node:
          layer: "frontend"
          ecosystem: "config"
          role: "config"
          section_type: "config"
          domain: "projects/agentic-workflow/packages/@sdd"
          workspace_root: "projects/agentic-workflow/packages/@sdd"
        frontend_node:
          workspace_root: "projects/agentic-workflow/packages/@sdd"
          role: "config"
          section_type: "config"
          artifact_kind: "workspace-config"
  frontend_ast:
    nodes:
      - path: "projects/agentic-workflow/packages/@sdd/ui/tsconfig.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
      - path: "projects/agentic-workflow/packages/@sdd/pipeline/tsconfig.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
      - path: "projects/agentic-workflow/packages/@sdd/core/tsconfig.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
      - path: "projects/agentic-workflow/packages/@sdd/app/tsconfig.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
      - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/tsconfig.json"
        workspace_root: "projects/agentic-workflow/packages/@sdd"
        role: "config"
        artifact_kind: "workspace-config"
        section_type: "config"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/agentic-workflow/packages/@sdd/ui/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/ui/tsconfig.json"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/pipeline/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/pipeline/tsconfig.json"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/core/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/core/tsconfig.json"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/app/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/app/tsconfig.json"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/package.json"
    action: modify
    section: manifest
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/agentic-workflow/packages/@sdd/spec-viewer/tsconfig.json"
    action: modify
    section: config
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
