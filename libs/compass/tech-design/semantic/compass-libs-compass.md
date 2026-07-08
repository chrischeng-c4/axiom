---
id: semantic-compass-libs-compass
summary: Semantic coverage for the Compass library source, manifest, tests, and project-root context artifact.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [schema, changes]
---

# Semantic TD: Compass

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "compass"
  source_group: "libs/compass"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/compass/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/check_pipeline.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/checker.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/core/config.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/core/index_config.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/core/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/diagnostic.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/format/detect.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/format/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/meteor.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/nebula.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/photon.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/quasar.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/rust_scanner.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/shield.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/test_extractor.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/python/titan.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/registry.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/rust/axum.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/rust/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/rust/reqwest.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/rust/serde.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/rust/sqlx.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/gen/traits.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/graph/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/graph/resolve.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lens_error.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/asyncapi.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/autofix.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/css.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/css_rules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/custom.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/dockerfile.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/embedded_markdown.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/gitlab_ci.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/gitlab_ci_rules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/go.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/graphql.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/html.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/html_rules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/javascript.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/kubernetes.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/kubernetes_rules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/markdown.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/mdx.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/mermaid.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/openapi.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/openrpc.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/proto.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/python.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/python_security.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/rust_checker.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/sql.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/terraform.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/terraform_rules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/toml_checker.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/typescript.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lint/yaml_dispatch.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lsp/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/lsp/server.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/outline.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/output/agent.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/output/agent_types.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/output/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/output/reporter.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/extract.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/extract_helpers.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/inline.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/move_def.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/rename.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/signature.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/refactoring/signature_helpers.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/schemas/frontmatter.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/schemas/gitlab.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/schemas/k8s.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/schemas/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/search/index.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/search/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/search/query.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/pdg/cfg.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/pdg/data_flow.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/pdg/dominator.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/pdg/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/scope.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/css.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/dockerfile.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/gitlab_ci.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/go.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/graphql_sym.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/html.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/javascript.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/kubernetes.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/markdown.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/mermaid.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/proto_sym.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/python.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/rust.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/sql_sym.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/terraform.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/toml_sym.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/symbols/typescript.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/types/go.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/types/go_advanced.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/types/go_tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/semantic/types/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/auto_discover.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/daemon.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/disk_cache.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/handler.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/incremental.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/protocol.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/server/watch_bridge.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/asyncapi/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/asyncapi/parser.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/ir.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/json_schema/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/json_schema/parser.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/mermaid/generator.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/mermaid/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/mermaid/parser.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/openapi/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/openapi/parser.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/statemachine/mermaid_plus.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/statemachine/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/statemachine/schema.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/spec/statemachine/validator.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/storage.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/syntax/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/syntax/parser.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/annotation.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/builtins.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/cache.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/cfg_narrow.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/check.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/check_tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/class_info.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/codegen.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/config.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/deep_inference.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/env.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/frameworks.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/imports.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/incremental.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/infer.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/infer_tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/model.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/modules.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/mutable_ast.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/narrow.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/narrow_tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/package_managers.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/project.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/propagation.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/refactoring.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/refactoring_multilang.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_advanced.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_infer.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_lifetimes.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_symbols.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_traits.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/rust_types.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/semantic_search.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/semantic_search_rust.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/stubs.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/ts_advanced.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/ts_infer.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/ts_types.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/ty.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/ty_tests.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/type_env.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/type_inference/typeshed.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/src/watch.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
- path: "libs/compass/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/compass"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/compass/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-cargo-toml>"
- path: "libs/compass/src/check_pipeline.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-check-pipeline-rs>"
- path: "libs/compass/src/checker.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-checker-rs>"
- path: "libs/compass/src/core/config.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-core-config-rs>"
- path: "libs/compass/src/core/index_config.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-core-index-config-rs>"
- path: "libs/compass/src/core/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-core-mod-rs>"
- path: "libs/compass/src/diagnostic.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-diagnostic-rs>"
- path: "libs/compass/src/format/detect.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-format-detect-rs>"
- path: "libs/compass/src/format/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-format-mod-rs>"
- path: "libs/compass/src/gen/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-mod-rs>"
- path: "libs/compass/src/gen/python/meteor.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-meteor-rs>"
- path: "libs/compass/src/gen/python/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-mod-rs>"
- path: "libs/compass/src/gen/python/nebula.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-nebula-rs>"
- path: "libs/compass/src/gen/python/photon.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-photon-rs>"
- path: "libs/compass/src/gen/python/quasar.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-quasar-rs>"
- path: "libs/compass/src/gen/python/rust_scanner.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-rust-scanner-rs>"
- path: "libs/compass/src/gen/python/shield.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-shield-rs>"
- path: "libs/compass/src/gen/python/test_extractor.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-test-extractor-rs>"
- path: "libs/compass/src/gen/python/titan.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-python-titan-rs>"
- path: "libs/compass/src/gen/registry.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-registry-rs>"
- path: "libs/compass/src/gen/rust/axum.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-rust-axum-rs>"
- path: "libs/compass/src/gen/rust/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-rust-mod-rs>"
- path: "libs/compass/src/gen/rust/reqwest.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-rust-reqwest-rs>"
- path: "libs/compass/src/gen/rust/serde.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-rust-serde-rs>"
- path: "libs/compass/src/gen/rust/sqlx.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-rust-sqlx-rs>"
- path: "libs/compass/src/gen/traits.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-gen-traits-rs>"
- path: "libs/compass/src/graph/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-graph-mod-rs>"
- path: "libs/compass/src/graph/resolve.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-graph-resolve-rs>"
- path: "libs/compass/src/lens_error.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lens-error-rs>"
- path: "libs/compass/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lib-rs>"
- path: "libs/compass/src/lint/asyncapi.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-asyncapi-rs>"
- path: "libs/compass/src/lint/autofix.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-autofix-rs>"
- path: "libs/compass/src/lint/css.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-css-rs>"
- path: "libs/compass/src/lint/css_rules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-css-rules-rs>"
- path: "libs/compass/src/lint/custom.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-custom-rs>"
- path: "libs/compass/src/lint/dockerfile.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-dockerfile-rs>"
- path: "libs/compass/src/lint/embedded_markdown.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-embedded-markdown-rs>"
- path: "libs/compass/src/lint/gitlab_ci.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-gitlab-ci-rs>"
- path: "libs/compass/src/lint/gitlab_ci_rules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-gitlab-ci-rules-rs>"
- path: "libs/compass/src/lint/go.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-go-rs>"
- path: "libs/compass/src/lint/graphql.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-graphql-rs>"
- path: "libs/compass/src/lint/html.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-html-rs>"
- path: "libs/compass/src/lint/html_rules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-html-rules-rs>"
- path: "libs/compass/src/lint/javascript.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-javascript-rs>"
- path: "libs/compass/src/lint/kubernetes.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-kubernetes-rs>"
- path: "libs/compass/src/lint/kubernetes_rules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-kubernetes-rules-rs>"
- path: "libs/compass/src/lint/markdown.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-markdown-rs>"
- path: "libs/compass/src/lint/mdx.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-mdx-rs>"
- path: "libs/compass/src/lint/mermaid.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-mermaid-rs>"
- path: "libs/compass/src/lint/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-mod-rs>"
- path: "libs/compass/src/lint/openapi.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-openapi-rs>"
- path: "libs/compass/src/lint/openrpc.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-openrpc-rs>"
- path: "libs/compass/src/lint/proto.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-proto-rs>"
- path: "libs/compass/src/lint/python.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-python-rs>"
- path: "libs/compass/src/lint/python_security.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-python-security-rs>"
- path: "libs/compass/src/lint/rust_checker.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-rust-checker-rs>"
- path: "libs/compass/src/lint/sql.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-sql-rs>"
- path: "libs/compass/src/lint/terraform.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-terraform-rs>"
- path: "libs/compass/src/lint/terraform_rules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-terraform-rules-rs>"
- path: "libs/compass/src/lint/toml_checker.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-toml-checker-rs>"
- path: "libs/compass/src/lint/typescript.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-typescript-rs>"
- path: "libs/compass/src/lint/yaml_dispatch.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lint-yaml-dispatch-rs>"
- path: "libs/compass/src/lsp/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lsp-mod-rs>"
- path: "libs/compass/src/lsp/server.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-lsp-server-rs>"
- path: "libs/compass/src/outline.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-outline-rs>"
- path: "libs/compass/src/output/agent.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-output-agent-rs>"
- path: "libs/compass/src/output/agent_types.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-output-agent-types-rs>"
- path: "libs/compass/src/output/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-output-mod-rs>"
- path: "libs/compass/src/output/reporter.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-output-reporter-rs>"
- path: "libs/compass/src/refactoring/extract.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-extract-rs>"
- path: "libs/compass/src/refactoring/extract_helpers.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-extract-helpers-rs>"
- path: "libs/compass/src/refactoring/inline.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-inline-rs>"
- path: "libs/compass/src/refactoring/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-mod-rs>"
- path: "libs/compass/src/refactoring/move_def.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-move-def-rs>"
- path: "libs/compass/src/refactoring/rename.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-rename-rs>"
- path: "libs/compass/src/refactoring/signature.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-signature-rs>"
- path: "libs/compass/src/refactoring/signature_helpers.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-refactoring-signature-helpers-rs>"
- path: "libs/compass/src/schemas/frontmatter.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-schemas-frontmatter-rs>"
- path: "libs/compass/src/schemas/gitlab.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-schemas-gitlab-rs>"
- path: "libs/compass/src/schemas/k8s.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-schemas-k8s-rs>"
- path: "libs/compass/src/schemas/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-schemas-mod-rs>"
- path: "libs/compass/src/search/index.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-search-index-rs>"
- path: "libs/compass/src/search/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-search-mod-rs>"
- path: "libs/compass/src/search/query.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-search-query-rs>"
- path: "libs/compass/src/semantic/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-mod-rs>"
- path: "libs/compass/src/semantic/pdg/cfg.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-pdg-cfg-rs>"
- path: "libs/compass/src/semantic/pdg/data_flow.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-pdg-data-flow-rs>"
- path: "libs/compass/src/semantic/pdg/dominator.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-pdg-dominator-rs>"
- path: "libs/compass/src/semantic/pdg/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-pdg-mod-rs>"
- path: "libs/compass/src/semantic/scope.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-scope-rs>"
- path: "libs/compass/src/semantic/symbols/css.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-css-rs>"
- path: "libs/compass/src/semantic/symbols/dockerfile.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-dockerfile-rs>"
- path: "libs/compass/src/semantic/symbols/gitlab_ci.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-gitlab-ci-rs>"
- path: "libs/compass/src/semantic/symbols/go.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-go-rs>"
- path: "libs/compass/src/semantic/symbols/graphql_sym.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-graphql-sym-rs>"
- path: "libs/compass/src/semantic/symbols/html.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-html-rs>"
- path: "libs/compass/src/semantic/symbols/javascript.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-javascript-rs>"
- path: "libs/compass/src/semantic/symbols/kubernetes.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-kubernetes-rs>"
- path: "libs/compass/src/semantic/symbols/markdown.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-markdown-rs>"
- path: "libs/compass/src/semantic/symbols/mermaid.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-mermaid-rs>"
- path: "libs/compass/src/semantic/symbols/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-mod-rs>"
- path: "libs/compass/src/semantic/symbols/proto_sym.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-proto-sym-rs>"
- path: "libs/compass/src/semantic/symbols/python.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-python-rs>"
- path: "libs/compass/src/semantic/symbols/rust.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-rust-rs>"
- path: "libs/compass/src/semantic/symbols/sql_sym.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-sql-sym-rs>"
- path: "libs/compass/src/semantic/symbols/terraform.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-terraform-rs>"
- path: "libs/compass/src/semantic/symbols/toml_sym.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-toml-sym-rs>"
- path: "libs/compass/src/semantic/symbols/typescript.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-symbols-typescript-rs>"
- path: "libs/compass/src/semantic/tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-tests-rs>"
- path: "libs/compass/src/semantic/types/go.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-types-go-rs>"
- path: "libs/compass/src/semantic/types/go_advanced.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-types-go-advanced-rs>"
- path: "libs/compass/src/semantic/types/go_tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-types-go-tests-rs>"
- path: "libs/compass/src/semantic/types/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-semantic-types-mod-rs>"
- path: "libs/compass/src/server/auto_discover.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-auto-discover-rs>"
- path: "libs/compass/src/server/daemon.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-daemon-rs>"
- path: "libs/compass/src/server/disk_cache.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-disk-cache-rs>"
- path: "libs/compass/src/server/handler.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-handler-rs>"
- path: "libs/compass/src/server/incremental.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-incremental-rs>"
- path: "libs/compass/src/server/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-mod-rs>"
- path: "libs/compass/src/server/protocol.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-protocol-rs>"
- path: "libs/compass/src/server/tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-tests-rs>"
- path: "libs/compass/src/server/watch_bridge.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-server-watch-bridge-rs>"
- path: "libs/compass/src/spec/asyncapi/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-asyncapi-mod-rs>"
- path: "libs/compass/src/spec/asyncapi/parser.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-asyncapi-parser-rs>"
- path: "libs/compass/src/spec/ir.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-ir-rs>"
- path: "libs/compass/src/spec/json_schema/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-json-schema-mod-rs>"
- path: "libs/compass/src/spec/json_schema/parser.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-json-schema-parser-rs>"
- path: "libs/compass/src/spec/mermaid/generator.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-mermaid-generator-rs>"
- path: "libs/compass/src/spec/mermaid/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-mermaid-mod-rs>"
- path: "libs/compass/src/spec/mermaid/parser.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-mermaid-parser-rs>"
- path: "libs/compass/src/spec/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-mod-rs>"
- path: "libs/compass/src/spec/openapi/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-openapi-mod-rs>"
- path: "libs/compass/src/spec/openapi/parser.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-openapi-parser-rs>"
- path: "libs/compass/src/spec/statemachine/mermaid_plus.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-statemachine-mermaid-plus-rs>"
- path: "libs/compass/src/spec/statemachine/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-statemachine-mod-rs>"
- path: "libs/compass/src/spec/statemachine/schema.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-statemachine-schema-rs>"
- path: "libs/compass/src/spec/statemachine/validator.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-spec-statemachine-validator-rs>"
- path: "libs/compass/src/storage.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-storage-rs>"
- path: "libs/compass/src/syntax/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-syntax-mod-rs>"
- path: "libs/compass/src/syntax/parser.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-syntax-parser-rs>"
- path: "libs/compass/src/type_inference/annotation.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-annotation-rs>"
- path: "libs/compass/src/type_inference/builtins.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-builtins-rs>"
- path: "libs/compass/src/type_inference/cache.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-cache-rs>"
- path: "libs/compass/src/type_inference/cfg_narrow.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-cfg-narrow-rs>"
- path: "libs/compass/src/type_inference/check.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-check-rs>"
- path: "libs/compass/src/type_inference/check_tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-check-tests-rs>"
- path: "libs/compass/src/type_inference/class_info.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-class-info-rs>"
- path: "libs/compass/src/type_inference/codegen.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-codegen-rs>"
- path: "libs/compass/src/type_inference/config.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-config-rs>"
- path: "libs/compass/src/type_inference/deep_inference.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-deep-inference-rs>"
- path: "libs/compass/src/type_inference/env.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-env-rs>"
- path: "libs/compass/src/type_inference/frameworks.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-frameworks-rs>"
- path: "libs/compass/src/type_inference/imports.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-imports-rs>"
- path: "libs/compass/src/type_inference/incremental.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-incremental-rs>"
- path: "libs/compass/src/type_inference/infer.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-infer-rs>"
- path: "libs/compass/src/type_inference/infer_tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-infer-tests-rs>"
- path: "libs/compass/src/type_inference/mod.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-mod-rs>"
- path: "libs/compass/src/type_inference/model.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-model-rs>"
- path: "libs/compass/src/type_inference/modules.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-modules-rs>"
- path: "libs/compass/src/type_inference/mutable_ast.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-mutable-ast-rs>"
- path: "libs/compass/src/type_inference/narrow.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-narrow-rs>"
- path: "libs/compass/src/type_inference/narrow_tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-narrow-tests-rs>"
- path: "libs/compass/src/type_inference/package_managers.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-package-managers-rs>"
- path: "libs/compass/src/type_inference/project.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-project-rs>"
- path: "libs/compass/src/type_inference/propagation.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-propagation-rs>"
- path: "libs/compass/src/type_inference/refactoring.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-refactoring-rs>"
- path: "libs/compass/src/type_inference/refactoring_multilang.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-refactoring-multilang-rs>"
- path: "libs/compass/src/type_inference/rust_advanced.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-advanced-rs>"
- path: "libs/compass/src/type_inference/rust_infer.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-infer-rs>"
- path: "libs/compass/src/type_inference/rust_lifetimes.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-lifetimes-rs>"
- path: "libs/compass/src/type_inference/rust_symbols.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-symbols-rs>"
- path: "libs/compass/src/type_inference/rust_traits.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-traits-rs>"
- path: "libs/compass/src/type_inference/rust_types.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-rust-types-rs>"
- path: "libs/compass/src/type_inference/semantic_search.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-semantic-search-rs>"
- path: "libs/compass/src/type_inference/semantic_search_rust.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-semantic-search-rust-rs>"
- path: "libs/compass/src/type_inference/stubs.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-stubs-rs>"
- path: "libs/compass/src/type_inference/ts_advanced.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-ts-advanced-rs>"
- path: "libs/compass/src/type_inference/ts_infer.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-ts-infer-rs>"
- path: "libs/compass/src/type_inference/ts_types.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-ts-types-rs>"
- path: "libs/compass/src/type_inference/ty.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-ty-rs>"
- path: "libs/compass/src/type_inference/ty_tests.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-ty-tests-rs>"
- path: "libs/compass/src/type_inference/type_env.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-type-env-rs>"
- path: "libs/compass/src/type_inference/typeshed.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-type-inference-typeshed-rs>"
- path: "libs/compass/src/watch.rs"
  action: modify
  section: schema
  description: |
    Existing Compass library behavior is covered by this semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-compass-src-watch-rs>"
- path: "libs/compass/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map, TD root, and workspace test command.
  impl_mode: codegen
```
