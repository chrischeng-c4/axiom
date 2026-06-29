---
id: lumen-sort-docs-has-child-sort
summary: >
  Refresh the agent-facing search documentation for issue #718. The runtime
  already supports `sort.missing=first|last` with total inclusion and supports
  `has_child` queries combined with parent-field sorting; the slice updates the
  OpenAPI/schema comments and offline LLM/spec cookbook text so agents no longer
  see the old restrictions.
capability_refs:
  - id: "agent-offline-integration"
    role: primary
    claim: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    coverage: partial
    rationale: >
      The requested fix is agent-facing offline documentation: OpenAPI schema,
      query-shape cookbook, and LLM workflow wording.
  - id: "search-core"
    role: contributes
    claim: "filter-sort-early-termination"
    coverage: partial
    rationale: >
      The docs describe supported filter+sort composition for data-table search.
fill_sections: [logic, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sort-doc-refresh-applicability
entry: start
nodes:
  start:      { kind: start, label: "issue #718: docs lag runtime behavior" }
  schema:     { kind: process, label: "Update SearchRequest.sort schema wording" }
  cookbook:   { kind: process, label: "Add has_child + parent sort cookbook cue" }
  workflow:   { kind: process, label: "Add LLM workflow line for nested filter + parent sort" }
  verify:     { kind: terminal, label: "spec_cli asserts agent-facing text" }
edges:
  - { from: start, to: schema }
  - { from: schema, to: cookbook }
  - { from: cookbook, to: workflow }
  - { from: workflow, to: verify }
---
flowchart TD
    start([#718 stale docs]) --> schema[OpenAPI/SearchRequest.sort wording]
    schema --> cookbook[lumen spec query shape mentions has_child + sort]
    cookbook --> workflow[lumen llm workflow mentions nested filter + parent sort]
    workflow --> verify([spec_cli locks wording])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sort-doc-refresh-unit-evidence
requirements:
  sort_missing_schema_docs:
    id: R1
    text: "SearchRequest.sort schema text documents missing=exclude versus first/last total inclusion"
    kind: documentation
    risk: medium
    verify: test
  has_child_sort_cookbook:
    id: R2
    text: "The offline query-shape cookbook says has_child can filter parents and sort by a parent field"
    kind: documentation
    risk: medium
    verify: test
  has_child_sort_workflow:
    id: R3
    text: "The LLM workflow topic confirms nested data-table rows can be filtered by has_child and sorted by parent fields"
    kind: documentation
    risk: medium
    verify: test
elements:
  spec_cli_docs_assertions:
    kind: test
    path: projects/lumen/tests/spec_cli.rs
relations:
  - { from: spec_cli_docs_assertions, verifies: sort_missing_schema_docs }
  - { from: spec_cli_docs_assertions, verifies: has_child_sort_cookbook }
  - { from: spec_cli_docs_assertions, verifies: has_child_sort_workflow }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "sort.missing docs are accurate"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "has_child + sort cookbook documented"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "LLM workflow mentions parent sort"
      risk: medium
      verifymethod: test
    }
    element spec_cli_docs_assertions {
      type: "rs/#[test]"
    }
    spec_cli_docs_assertions - verifies -> R1
    spec_cli_docs_assertions - verifies -> R2
    spec_cli_docs_assertions - verifies -> R3
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: spec-cli-doc-surface
    name: "spec cli doc surface"
    runner: cargo
    path: projects/lumen/tests/spec_cli.rs
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    verifies:
      - "OpenAPI/schema JSON is valid."
      - "Query-shape cookbook includes the updated has_child + sort wording."
      - "LLM workflow includes the nested filter + parent sort confirmation."
  - id: has-child-sort-runtime-regression
    name: "has_child sort runtime regression"
    runner: cargo
    path: projects/lumen/src/storage.rs
    command: "cargo test -p lumen storage::tests::has_child_sort_tests -- --nocapture"
    verifies:
      - "The runtime behavior documented here remains covered by existing storage tests."
```
