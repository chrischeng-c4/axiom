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
