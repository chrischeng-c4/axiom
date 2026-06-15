---
id: semantic-lumen-src
summary: Semantic coverage for "projects/lumen/src"
capability_refs:
  - id: "search"
    role: primary
    gap: "query-planner-boolean-eval-roaring-postings"
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/src`."
  - id: "agentic-integration"
    role: primary
    gap: "lumen-spec-schema-openapi-json-yaml-json-schema-offline"
    claim: "lumen-spec-schema-openapi-json-yaml-json-schema-offline"
    coverage: full
    rationale: "The source spec module emits OpenAPI JSON, OpenAPI YAML, JSON-schema, and agent-facing schema surfaces."
  - id: "agentic-integration"
    role: primary
    gap: "query-shape-cookbook-field-analyzer-catalog"
    claim: "query-shape-cookbook-field-analyzer-catalog"
    coverage: full
    rationale: "The source spec module emits the query-shape cookbook and field/analyzer/vector metric catalogs."
  - id: "agentic-integration"
    role: primary
    gap: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    claim: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    coverage: full
    rationale: "The source spec module emits the LLM topic set for offline agent onboarding."
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: lumen/src

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/src"
  source_group: "projects/lumen/src"
  coverage_kind: semantic
  evidence:
    source_units:
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
coverage_kind: semantic
strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
evidence:
  source_tests: []
---
requirementDiagram

element UT_SOURCE_TESTS {
  type: "TestEvidence"
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."
```
