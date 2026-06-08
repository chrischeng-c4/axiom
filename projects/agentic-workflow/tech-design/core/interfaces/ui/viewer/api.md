---
id: sdd-ui-viewer-api
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# UI Viewer API Response Types

## Overview
<!-- type: overview lang: markdown -->

REST response types for the local viewer in
`projects/agentic-workflow/src/ui/viewer/api.rs`. Eight Serialize-only shapes used as
JSON responses by the `LocalDataSource` TypeScript client:

- `IssueSummaryResponse` — issue summary card.
- `IssueResponse` — full issue detail.
- `TechDesignSummaryResponse` — TD summary card with `serde(rename = "crate")` on `crate_name`.
- `TechDesignResponse` — full TD detail (same rename).
- `ChangeSummaryResponse` — change summary card.
- `ChangeResponse` — full change detail.
- `LineageGraphResponse` — opaque graph nodes/edges (serde_json::Value).
- `ProjectInfoResponse` — project metadata.

All eight derive `[Serialize]` only. Codegen replaces all eight type
declarations. Module imports, the `project_root` helper, and all axum
handler fns + the test module stay hand-written.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  IssueSummaryResponse:
    type: object
    required: [id, issue_number, title, status, priority, labels, created_at]
    description: Issue summary card.
    properties:
      id:
        type: string
        description: "Issue identifier."
      issue_number:
        type: integer
        x-rust-type: "u64"
        description: "Numeric issue number."
      title:
        type: string
        description: "Issue title."
      status:
        type: string
        description: "Status string."
      priority:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional priority."
      labels:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Labels."
      created_at:
        type: string
        description: "Creation timestamp."
    x-rust-struct:
      derive: [Serialize]

  IssueResponse:
    type: object
    required: [id, issue_number, title, description, status, priority, labels, created_at, updated_at, closed_at]
    description: Full issue detail.
    properties:
      id:
        type: string
        description: "Issue identifier."
      issue_number:
        type: integer
        x-rust-type: "u64"
        description: "Numeric issue number."
      title:
        type: string
        description: "Issue title."
      description:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional issue body."
      status:
        type: string
        description: "Status string."
      priority:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional priority."
      labels:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Labels."
      created_at:
        type: string
        description: "Creation timestamp."
      updated_at:
        type: string
        description: "Last update timestamp."
      closed_at:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional closed timestamp."
    x-rust-struct:
      derive: [Serialize]

  TechDesignSummaryResponse:
    type: object
    required: [id, crate_name, path, title, updated_at]
    description: TD summary card.
    properties:
      id:
        type: string
        description: "TD identifier."
      crate_name:
        type: string
        x-serde-rename: "crate"
        description: "Crate name (serialised as `crate`)."
      path:
        type: string
        description: "Spec path."
      title:
        type: string
        description: "TD title."
      updated_at:
        type: string
        description: "Last update timestamp."
    x-rust-struct:
      derive: [Serialize]

  TechDesignResponse:
    type: object
    required: [id, crate_name, path, title, content, updated_at]
    description: Full TD detail.
    properties:
      id:
        type: string
        description: "TD identifier."
      crate_name:
        type: string
        x-serde-rename: "crate"
        description: "Crate name."
      path:
        type: string
        description: "Spec path."
      title:
        type: string
        description: "TD title."
      content:
        type: string
        description: "Spec markdown body."
      updated_at:
        type: string
        description: "Last update timestamp."
    x-rust-struct:
      derive: [Serialize]

  ChangeSummaryResponse:
    type: object
    required: [id, description, phase, issue_ids, created_at, updated_at]
    description: Change summary card.
    properties:
      id:
        type: string
        description: "Change identifier."
      description:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional description."
      phase:
        type: string
        description: "Lifecycle phase."
      issue_ids:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Linked issue IDs."
      created_at:
        type: string
        description: "Creation timestamp."
      updated_at:
        type: string
        description: "Last update timestamp."
    x-rust-struct:
      derive: [Serialize]

  ChangeResponse:
    type: object
    required: [id, description, phase, issue_ids, spec_ids, created_at, updated_at]
    description: Full change detail.
    properties:
      id:
        type: string
        description: "Change identifier."
      description:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional description."
      phase:
        type: string
        description: "Lifecycle phase."
      issue_ids:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Linked issue IDs."
      spec_ids:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Linked spec IDs."
      created_at:
        type: string
        description: "Creation timestamp."
      updated_at:
        type: string
        description: "Last update timestamp."
    x-rust-struct:
      derive: [Serialize]

  LineageGraphResponse:
    type: object
    required: [nodes, edges]
    description: Opaque lineage graph.
    properties:
      nodes:
        type: array
        items: { type: object }
        x-rust-type: "Vec<serde_json::Value>"
        description: "Graph nodes."
      edges:
        type: array
        items: { type: object }
        x-rust-type: "Vec<serde_json::Value>"
        description: "Graph edges."
    x-rust-struct:
      derive: [Serialize]

  ProjectInfoResponse:
    type: object
    required: [name, root, has_score]
    description: Project metadata.
    properties:
      name:
        type: string
        description: "Project name."
      root:
        type: string
        description: "Project root path (string)."
      has_score:
        type: boolean
        description: "Whether `.aw/` exists at root."
    x-rust-struct:
      derive: [Serialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/viewer/api.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - IssueSummaryResponse
      - IssueResponse
      - TechDesignSummaryResponse
      - TechDesignResponse
      - ChangeSummaryResponse
      - ChangeResponse
      - LineageGraphResponse
      - ProjectInfoResponse
    description: |
      Codegen replaces all eight response struct declarations.
  - path: projects/agentic-workflow/src/ui/viewer/api.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports, the
      `project_root` helper, and all axum handler fns + tests.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Eight Serialize-only response structs sharing similar shape (id + metadata + timestamps).
- [schema] All in `required:`; foreign types via x-rust-type; x-serde-rename for `crate` keyword fields.
- [changes] All eight in `replaces`; helper + handlers preserved.
