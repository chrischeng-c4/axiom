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
id: sort-doc-refresh-contract
entry: docs
nodes:
  docs:     { kind: start, label: "Agent reads lumen spec / OpenAPI / llm workflow" }
  missing:  { kind: process, label: "sort.missing says exclude drops; first/last keep and count" }
  arity:    { kind: process, label: "sort docs say up to MAX_SORT_KEYS=4" }
  nested:   { kind: process, label: "has_child can filter parents before parent-field sort" }
  verify:   { kind: terminal, label: "spec_cli locks these strings" }
edges:
  - { from: docs, to: missing }
  - { from: missing, to: arity }
  - { from: arity, to: nested }
  - { from: nested, to: verify }
---
flowchart TD
    docs([offline agent docs]) --> missing[sort.missing first/last placement + total]
    missing --> arity[up to 4 sort keys]
    arity --> nested[has_child + parent sort supported]
    nested --> verify([spec_cli assertions])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: sort-doc-refresh-contract-tests
requirements:
  openapi_sort_doc_current:
    id: R1
    text: "The OpenAPI schema text for SearchRequest.sort is current for missing rows and sort key count"
    kind: documentation
    risk: medium
    verify: test
  query_shape_has_child_sort_current:
    id: R2
    text: "The query shape cookbook confirms has_child can be combined with parent-field sort"
    kind: documentation
    risk: medium
    verify: test
  llm_workflow_has_child_sort_current:
    id: R3
    text: "The LLM workflow topic confirms nested list-row search can filter via has_child and sort parents"
    kind: documentation
    risk: medium
    verify: test
elements:
  spec_cli_doc_contracts:
    kind: test
    path: projects/lumen/tests/spec_cli.rs
relations:
  - { from: spec_cli_doc_contracts, verifies: openapi_sort_doc_current }
  - { from: spec_cli_doc_contracts, verifies: query_shape_has_child_sort_current }
  - { from: spec_cli_doc_contracts, verifies: llm_workflow_has_child_sort_current }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "SearchRequest.sort schema text current"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "query shape says has_child + parent sort"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "llm workflow says has_child + parent sort"
      risk: medium
      verifymethod: test
    }
    element spec_cli_doc_contracts {
      type: "rs/#[test]"
    }
    spec_cli_doc_contracts - verifies -> R1
    spec_cli_doc_contracts - verifies -> R2
    spec_cli_doc_contracts - verifies -> R3
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: spec-cli-agent-doc-contract
    name: "spec cli agent doc contract"
    runner: cargo
    path: projects/lumen/tests/spec_cli.rs
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    verifies:
      - "OpenAPI JSON/YAML remain valid after doc string changes."
      - "Query shape and LLM workflow text expose current sort behavior."
  - id: storage-has-child-sort-contract
    name: "storage has_child sort contract"
    runner: cargo
    path: projects/lumen/src/storage.rs
    command: "cargo test -p lumen storage::tests::has_child_sort_tests -- --nocapture"
    verifies:
      - "Runtime support for has_child + parent sorting remains covered."
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/types.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update SearchRequest.sort docs to say up to four keys, missing=first/last keep and count rows, and sorted has_child queries are supported through materialization."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update query shape and LLM workflow text for has_child + parent-field sort."
  - path: projects/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Lock the corrected agent-facing docs with spec_cli assertions."
```
