---
id: semantic-lumen-bin
summary: Semantic coverage for "apps/lumen/src/bin"
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/lumen/src/bin`."
fill_sections: [schema, changes]
---

# Semantic TD: lumen/bin

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/bin"
  source_group: "apps/lumen/src/bin"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/src/bin/lumen.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "Command"
            kind: "enum"
            public: false
          - name: "DockerfileArgs"
            kind: "struct"
            public: false
          - name: "DockerfileCmd"
            kind: "enum"
            public: false
          - name: "DockerfileRenderArgs"
            kind: "struct"
            public: false
          - name: "DockerfileVariant"
            kind: "enum"
            public: false
          - name: "K8sArgs"
            kind: "struct"
            public: false
          - name: "K8sCmd"
            kind: "enum"
            public: false
          - name: "K8sCrdArgs"
            kind: "struct"
            public: false
          - name: "K8sCrdCmd"
            kind: "enum"
            public: false
          - name: "K8sOperatorArgs"
            kind: "struct"
            public: false
          - name: "K8sOperatorCmd"
            kind: "enum"
            public: false
          - name: "K8sOperatorRenderArgs"
            kind: "struct"
            public: false
          - name: "K8sOperatorResizeStorageArgs"
            kind: "struct"
            public: false
          - name: "K8sInstanceArgs"
            kind: "struct"
            public: false
          - name: "K8sInstanceCmd"
            kind: "enum"
            public: false
          - name: "K8sInstanceRenderArgs"
            kind: "struct"
            public: false
          - name: "K8sInstanceProfile"
            kind: "enum"
            public: false
          - name: "K8sFileOutputArgs"
            kind: "struct"
            public: false
          - name: "UpgradeArgs"
            kind: "struct"
            public: false
          - name: "IssueArgs"
            kind: "struct"
            public: false
          - name: "IssueCommand"
            kind: "enum"
            public: false
          - name: "IssueSearchArgs"
            kind: "struct"
            public: false
          - name: "IssueViewArgs"
            kind: "struct"
            public: false
          - name: "IssueCreateArgs"
            kind: "struct"
            public: false
          - name: "BackupArgs"
            kind: "struct"
            public: false
          - name: "LlmTopic"
            kind: "enum"
            public: false
          - name: "LlmFormat"
            kind: "enum"
            public: false
          - name: "LlmArgs"
            kind: "struct"
            public: false
          - name: "WalBackend"
            kind: "enum"
            public: false
          - name: "resolve_wal_backend"
            kind: "function"
            public: false
          - name: "LogFormat"
            kind: "enum"
            public: false
          - name: "Persistence"
            kind: "enum"
            public: false
          - name: "SpecFormat"
            kind: "enum"
            public: false
          - name: "SpecArgs"
            kind: "struct"
            public: false
          - name: "SpecSub"
            kind: "enum"
            public: false
          - name: "GenArgs"
            kind: "struct"
            public: false
          - name: "GenLang"
            kind: "enum"
            public: false
          - name: "GenHttp"
            kind: "enum"
            public: false
          - name: "ServeArgs"
            kind: "struct"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen/src/bin"
      - path: "apps/lumen/src/bin/lumen-bench.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "DEFAULT_DOCUMENTS"
            kind: "constant"
            public: false
          - name: "DEFAULT_PAGE_SIZE"
            kind: "constant"
            public: false
          - name: "DEFAULT_QUERIES"
            kind: "constant"
            public: false
          - name: "SORTED_PAGE_BUDGET_US"
            kind: "constant"
            public: false
          - name: "Cli"
            kind: "struct"
            public: false
          - name: "Command"
            kind: "enum"
            public: false
          - name: "RunArgs"
            kind: "struct"
            public: false
          - name: "BenchReport"
            kind: "struct"
            public: false
          - name: "main"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: false
          - name: "parse_types"
            kind: "function"
            public: false
          - name: "print_report"
            kind: "function"
            public: false
          - name: "run_sorted_page_deep"
            kind: "function"
            public: false
          - name: "run_bool_filter"
            kind: "function"
            public: false
          - name: "summarize"
            kind: "function"
            public: false
          - name: "sorted_page_request"
            kind: "function"
            public: false
          - name: "build_corpus"
            kind: "function"
            public: false
          - name: "spec"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/lumen/src/bin"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/lumen/src/bin/lumen.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/lumen/src/bin/lumen-bench.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
