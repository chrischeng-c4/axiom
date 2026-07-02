---
id: semantic-lumen-operator
summary: Semantic coverage for "projects/lumen/src/operator"
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/src/operator`."
fill_sections: [schema, changes]
---

# Semantic TD: lumen/operator

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/operator"
  source_group: "projects/lumen/src/operator"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/src/operator/render.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "APP"
            kind: "constant"
            public: false
          - name: "API_VERSION"
            kind: "constant"
            public: false
          - name: "KIND"
            kind: "constant"
            public: false
          - name: "CLIENT_PORT"
            kind: "constant"
            public: false
          - name: "BACKUP_COMPONENT"
            kind: "constant"
            public: false
          - name: "TOKEN_REGISTRY_VOLUME"
            kind: "constant"
            public: false
          - name: "TOKEN_REGISTRY_KEY"
            kind: "constant"
            public: false
          - name: "TOKEN_REGISTRY_MOUNT_DIR"
            kind: "constant"
            public: false
          - name: "TOKEN_REGISTRY_FILE"
            kind: "constant"
            public: false
          - name: "instance"
            kind: "function"
            public: false
          - name: "namespace"
            kind: "function"
            public: false
          - name: "ctx"
            kind: "function"
            public: false
          - name: "labels"
            kind: "function"
            public: false
          - name: "selector"
            kind: "function"
            public: false
          - name: "owner_ref"
            kind: "function"
            public: false
          - name: "token_registry_secret"
            kind: "function"
            public: false
          - name: "meta"
            kind: "function"
            public: false
          - name: "render"
            kind: "function"
            public: true
          - name: "backup_cron_job"
            kind: "function"
            public: false
          - name: "downward_api_env"
            kind: "function"
            public: false
          - name: "serving_statefulset"
            kind: "function"
            public: false
          - name: "serving_headless_service"
            kind: "function"
            public: false
          - name: "service_account"
            kind: "function"
            public: false
          - name: "serving_configmap"
            kind: "function"
            public: false
          - name: "serving_env"
            kind: "function"
            public: false
          - name: "serving_deployment"
            kind: "function"
            public: false
          - name: "serving_service"
            kind: "function"
            public: false
          - name: "serving_hpa"
            kind: "function"
            public: false
          - name: "serving_pdb"
            kind: "function"
            public: false
          - name: "service_monitor"
            kind: "function"
            public: false
          - name: "prometheus_rule"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
      - path: "projects/lumen/src/operator/crd.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "LumenSpec"
            kind: "struct"
            public: true
          - name: "LogFormat"
            kind: "enum"
            public: true
          - name: "as_env"
            kind: "function"
            public: true
          - name: "AuthMode"
            kind: "enum"
            public: true
          - name: "as_env"
            kind: "function"
            public: true
          - name: "ServingSpec"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "ServingBackupSpec"
            kind: "struct"
            public: true
          - name: "Autoscaling"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "LumenStatus"
            kind: "struct"
            public: true
          - name: "default_shard_count"
            kind: "function"
            public: false
          - name: "default_replicas_per_shard"
            kind: "function"
            public: false
          - name: "default_serving_cpu"
            kind: "function"
            public: false
          - name: "default_serving_memory"
            kind: "function"
            public: false
          - name: "default_grace_secs"
            kind: "function"
            public: false
          - name: "default_raft_storage"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
      - path: "projects/lumen/src/operator/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "crd"
            kind: "module"
            public: true
          - name: "lease"
            kind: "module"
            public: true
          - name: "reconcile"
            kind: "module"
            public: true
          - name: "render"
            kind: "module"
            public: true
          - name: "resize"
            kind: "module"
            public: true
          - name: "crd_yaml"
            kind: "function"
            public: true
          - name: "normalize_kubernetes_schema_formats"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
      - path: "projects/lumen/src/operator/reconcile.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "render"
            kind: "function"
            public: false
          - name: "readiness_targets"
            kind: "function"
            public: false
          - name: "status_patch"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
      - path: "projects/lumen/src/operator/lease.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
      - path: "projects/lumen/src/operator/resize.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "ResizeAction"
            kind: "enum"
            public: true
          - name: "parse_storage_bytes"
            kind: "function"
            public: true
          - name: "decide"
            kind: "function"
            public: true
          - name: "PvcResizeOutcome"
            kind: "struct"
            public: true
          - name: "resize_instance"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/lumen/src/operator"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/src/operator/render.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/lumen/src/operator/crd.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/lumen/src/operator/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/lumen/src/operator/reconcile.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/lumen/src/operator/lease.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/lumen/src/operator/resize.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
