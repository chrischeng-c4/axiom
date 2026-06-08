---
id: sdd-services-platform-sync-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Platform Sync Types

## Overview
<!-- type: overview lang: markdown -->

Common types for the platform-sync service in
`projects/agentic-workflow/src/services/platform_sync/types.rs`. Five shapes:

- `SyncResult` — serialisable record of a single sync invocation
  (status enum, optional issue url/number, message, optional spec
  results).
- `SpecSyncResult` — serialisable record of one spec issue's sync
  outcome.
- `SyncStatus` — 4-variant unit enum (`Created`/`Updated`/`Partial`/`Error`)
  with container-level `serde(rename_all = "lowercase")`. Copy.
- `SyncPayload` — non-serialisable build-time payload for a parent
  issue (Debug + Clone only).
- `SpecPayload` — non-serialisable build-time payload for a spec issue.

Codegen replaces all five type declarations. No impl blocks exist in
the source — there is nothing to mark hand-written.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SyncStatus:
    type: string
    enum: [Created, Updated, Partial, Error]
    description: |
      Status of the sync operation.
      Created: new issue created on platform.
      Updated: existing issue updated.
      Partial: parent succeeded but some specs failed.
      Error: sync failed.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase
      variants:
        - { name: Created, doc: "New issue created on platform." }
        - { name: Updated, doc: "Existing issue updated." }
        - { name: Partial, doc: "Parent succeeded but some specs failed." }
        - { name: Error,   doc: "Sync failed." }

  SyncResult:
    type: object
    required: [status, message, spec_results]
    description: Result of a sync operation.
    properties:
      status:
        $ref: "#/definitions/SyncStatus"
      issue_url:
        type: string
        description: "Optional URL of the synced issue on the platform."
      issue_number:
        type: integer
        x-rust-type: u64
        description: "Optional platform-side issue number."
      message:
        type: string
        description: "Human-readable message describing the outcome."
      spec_results:
        type: array
        items:
          $ref: "#/definitions/SpecSyncResult"
        description: "Results for spec issues."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecSyncResult:
    type: object
    required: [spec_id, status]
    description: Result of syncing a spec issue.
    properties:
      spec_id:
        type: string
      status:
        $ref: "#/definitions/SyncStatus"
      issue_url:
        type: string
      issue_number:
        type: integer
        x-rust-type: u64
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SyncPayload:
    type: object
    required: [change_id, title, body, labels, specs]
    description: Payload to sync to platform.
    properties:
      change_id:
        type: string
      title:
        type: string
      body:
        type: string
      labels:
        type: array
        items: { type: string }
      existing_issue:
        type: integer
        x-rust-type: u64
        description: "Existing issue number (from frontmatter), None if new."
      specs:
        type: array
        items:
          $ref: "#/definitions/SpecPayload"
    x-rust-struct:
      derive: [Debug, Clone]

  SpecPayload:
    type: object
    required: [spec_id, title, body, labels]
    description: Payload for a spec issue.
    properties:
      spec_id:
        type: string
      title:
        type: string
      body:
        type: string
      labels:
        type: array
        items: { type: string }
      existing_issue:
        type: integer
        x-rust-type: u64
    x-rust-struct:
      derive: [Debug, Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/types.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SyncStatus
      - SyncResult
      - SpecSyncResult
      - SyncPayload
      - SpecPayload
    description: |
      Codegen replaces all 5 type declarations. No impl blocks exist in
      the source so nothing else needs preserving.
  - path: projects/agentic-workflow/src/services/platform_sync/types.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Module docstring remains untouched outside CODEGEN-BEGIN/END. The
      serde import is owned by the generated schema block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All five types match the source file exactly — fields, field types (`u64` via `x-rust-type`, `Option<>` via absence from `required`), derive lists, and serde attributes (`rename_all: lowercase`, `x-serde-default: true`) are all correct.
- [changes] Two-entry changes block correctly separates the codegen block (replaces all 5 type declarations) from the hand-written block (module docstring + `use serde` import), sufficient for the round-trip codegen workflow.
