---
id: semantic-preview-src
summary: Semantic coverage for "projects/preview/src"
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "mr-scoped-namespace-projection"
    claim: "mr-scoped-namespace-projection"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/preview/src`."
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "base-workload-discovery"
    claim: "base-workload-discovery"
    coverage: partial
    rationale: "Base workload discovery is implemented by the discover module and CLI/render integration."
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "local-apply-and-gitops-execution"
    claim: "local-apply-and-gitops-execution"
    coverage: partial
    rationale: "Local apply and GitOps execution are implemented by the apply module plus CLI integration."
  - id: "preview-external-contracts"
    role: primary
    gap: "local-apply-gitops-execution-ec"
    claim: "local-apply-gitops-execution-ec"
    coverage: partial
    rationale: "Local apply and GitOps execution EC coverage is implemented by the apply module plus CLI integration."
  - id: "preview-external-contracts"
    role: primary
    gap: "local-router-adapter"
    claim: "local-router-adapter"
    coverage: partial
    rationale: "Local router adapter behavior is implemented by the router module and CLI integration."
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "guarded-cleanup-janitor"
    claim: "guarded-cleanup-janitor"
    coverage: partial
    rationale: "Guarded cleanup janitor behavior is implemented by the cleanup module and CLI integration."
fill_sections: [schema, changes]
---

# Semantic TD: preview/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "preview/src"
  source_group: "projects/preview/src"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/preview/src/render.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "RenderInput"
            kind: "struct"
            public: true
          - name: "RenderFile"
            kind: "struct"
            public: true
          - name: "render_files"
            kind: "function"
            public: true
          - name: "render_single_manifest"
            kind: "function"
            public: true
          - name: "preview_environment"
            kind: "function"
            public: true
          - name: "cleanup_plan"
            kind: "function"
            public: true
          - name: "mr_comment"
            kind: "function"
            public: true
          - name: "phase_name"
            kind: "function"
            public: false
          - name: "labels"
            kind: "function"
            public: false
          - name: "label_map"
            kind: "function"
            public: false
          - name: "selector"
            kind: "function"
            public: false
          - name: "namespace"
            kind: "function"
            public: false
          - name: "deployment"
            kind: "function"
            public: false
          - name: "service"
            kind: "function"
            public: false
          - name: "route_binding"
            kind: "function"
            public: false
          - name: "yaml"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/lib.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        symbols:
          - name: "apply"
            kind: "module"
            public: true
          - name: "cleanup"
            kind: "module"
            public: true
          - name: "discover"
            kind: "module"
            public: true
          - name: "model"
            kind: "module"
            public: true
          - name: "render"
            kind: "module"
            public: true
          - name: "router"
            kind: "module"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/apply.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ManifestInventory"
            kind: "struct"
            public: true
          - name: "ManifestInventoryEntry"
            kind: "struct"
            public: true
          - name: "ApplyOptions"
            kind: "struct"
            public: true
          - name: "ApplySummary"
            kind: "struct"
            public: true
          - name: "GitopsBundleFile"
            kind: "struct"
            public: true
          - name: "apply_manifest_paths"
            kind: "function"
            public: true
          - name: "manifest_inventory_for_env"
            kind: "function"
            public: true
          - name: "manifest_inventory_from_dir"
            kind: "function"
            public: true
          - name: "apply_rendered_manifests"
            kind: "function"
            public: true
          - name: "render_gitops_bundle"
            kind: "function"
            public: true
          - name: "apply_summary_markdown"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/cleanup.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "JanitorPlan"
            kind: "struct"
            public: true
          - name: "JanitorInput"
            kind: "struct"
            public: true
          - name: "CleanupApplyOptions"
            kind: "struct"
            public: true
          - name: "CleanupApplySummary"
            kind: "struct"
            public: true
          - name: "plan_guarded_cleanup"
            kind: "function"
            public: true
          - name: "apply_guarded_cleanup"
            kind: "function"
            public: true
          - name: "read_janitor_plan"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/discover.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "BaseWorkloadContract"
            kind: "struct"
            public: true
          - name: "BaseContainerContract"
            kind: "struct"
            public: true
          - name: "BaseContainerPort"
            kind: "struct"
            public: true
          - name: "BaseEnvVar"
            kind: "struct"
            public: true
          - name: "BaseServicePort"
            kind: "struct"
            public: true
          - name: "discover_base_with_kubectl"
            kind: "function"
            public: true
          - name: "normalize_base_workload"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/router.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "RouteBinding"
            kind: "struct"
            public: true
          - name: "BaseRoute"
            kind: "struct"
            public: true
          - name: "RouteRequest"
            kind: "struct"
            public: true
          - name: "ResolvedRoute"
            kind: "struct"
            public: true
          - name: "RouteDecision"
            kind: "struct"
            public: true
          - name: "RouteOutcome"
            kind: "enum"
            public: true
          - name: "resolve_route"
            kind: "function"
            public: true
          - name: "resolve_route_with_base"
            kind: "function"
            public: true
          - name: "load_route_table_from_rendered_dir"
            kind: "function"
            public: true
          - name: "load_route_table_from_kubectl"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/main.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "Command"
            kind: "enum"
            public: false
          - name: "IssueCommand"
            kind: "enum"
            public: false
          - name: "GitopsCommand"
            kind: "enum"
            public: false
          - name: "RouterCommand"
            kind: "enum"
            public: false
          - name: "CleanupCommand"
            kind: "enum"
            public: false
          - name: "RenderArgs"
            kind: "struct"
            public: false
          - name: "DiscoverBaseArgs"
            kind: "struct"
            public: false
          - name: "ApplyArgs"
            kind: "struct"
            public: false
          - name: "GitopsRenderArgs"
            kind: "struct"
            public: false
          - name: "RouterResolveArgs"
            kind: "struct"
            public: false
          - name: "CleanupJanitorPlanArgs"
            kind: "struct"
            public: false
          - name: "CleanupApplyArgs"
            kind: "struct"
            public: false
          - name: "CleanupArgs"
            kind: "struct"
            public: false
          - name: "main"
            kind: "function"
            public: false
          - name: "render"
            kind: "function"
            public: false
          - name: "discover_base"
            kind: "function"
            public: false
          - name: "apply"
            kind: "function"
            public: false
          - name: "gitops_render"
            kind: "function"
            public: false
          - name: "router_resolve"
            kind: "function"
            public: false
          - name: "cleanup_janitor_plan"
            kind: "function"
            public: false
          - name: "cleanup_apply"
            kind: "function"
            public: false
          - name: "print_llm"
            kind: "function"
            public: false
          - name: "into_input"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
      - path: "projects/preview/src/model.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model"]
        symbols:
          - name: "PreviewEnvironment"
            kind: "struct"
            public: true
          - name: "PreviewMetadata"
            kind: "struct"
            public: true
          - name: "Label"
            kind: "struct"
            public: true
          - name: "PreviewSpec"
            kind: "struct"
            public: true
          - name: "RouteSpec"
            kind: "struct"
            public: true
          - name: "GkeSpec"
            kind: "struct"
            public: true
          - name: "PreviewStatus"
            kind: "struct"
            public: true
          - name: "PreviewPhase"
            kind: "enum"
            public: true
          - name: "CleanupPlan"
            kind: "struct"
            public: true
          - name: "CleanupAction"
            kind: "enum"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/preview/src"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/preview/src/render.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-render-rs>"
  - path: "projects/preview/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-lib-rs>"
  - path: "projects/preview/src/apply.rs"
    action: add
    section: schema
    description: |
      Local apply, manifest inventory, and GitOps execution behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-apply-rs>"
  - path: "projects/preview/src/cleanup.rs"
    action: add
    section: schema
    description: |
      Guarded cleanup janitor source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-cleanup-rs>"
  - path: "projects/preview/src/discover.rs"
    action: add
    section: schema
    description: |
      Base workload discovery and normalization source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-discover-rs>"
  - path: "projects/preview/src/router.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-router-rs>"
  - path: "projects/preview/src/main.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-main-rs>"
  - path: "projects/preview/src/model.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-model-rs>"
```
