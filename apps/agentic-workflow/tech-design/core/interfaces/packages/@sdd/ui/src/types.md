---
id: projects-sdd-packages-sdd-ui-src-types-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized apps/agentic-workflow/packages/@sdd/ui/src/types.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/packages/@sdd/ui/src/types.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Change` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 50 |  |
| `ConnectRepoData` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 110 |  |
| `DAGNode` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 72 |  |
| `FileContent` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 20 |  |
| `FileEntry` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 13 |  |
| `IssueComment` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 99 |  |
| `ProjectSpec` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 33 |  |
| `SelectOption` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 90 |  |
| `SpecRun` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | interface | pub | 81 |  |
| `StageStatus` | apps/agentic-workflow/packages/@sdd/ui/src/types.ts | type | pub | 70 |  |
## Source
<!-- type: source lang: ts -->

```ts
/**
 * Shared types for @cclab/ui components.
 * Only includes types needed by the UI layer -- not Conductor-specific types
 * like DashboardStats, WorkflowListItem, etc.
 */

// ---------------------------------------------------------------------------
// File browser / workspace
// ---------------------------------------------------------------------------

export interface FileEntry {
  name: string
  path: string
  type: 'tree' | 'blob'
  size?: number
}

export interface FileContent {
  file_name: string
  file_path: string
  size: number
  encoding: 'text' | 'base64'
  content: string
  mime_type?: string
}

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

export interface ProjectSpec {
  id: number
  project_id: string
  path: string
  module_path: string | null
  format: string
  content: string
  content_hash: string
  version: number
  created_at: string
  updated_at: string
}

// ---------------------------------------------------------------------------
// Changes
// ---------------------------------------------------------------------------

export interface Change {
  id: string
  project_id: string
  title: string
  description: string | null
  status: 'draft' | 'in_progress' | 'review' | 'merged' | 'closed'
  branch_name: string | null
  issue_ids: string[]
  spec_ids: string[]
  external_mr_id: string | null
  external_mr_url: string | null
  sync_status: string
  created_at: string
  updated_at: string
}

// ---------------------------------------------------------------------------
// Pipeline DAG
// ---------------------------------------------------------------------------

export type StageStatus = 'pending' | 'running' | 'done' | 'failed' | 'skipped'

export interface DAGNode {
  id: string
  label: string
  status: StageStatus
  deps: string[]
  module_key: string
  artifact?: { spec_id: number; path: string; format: string }
}

export interface SpecRun {
  startedAt: Date
  nodes: DAGNode[]
}

// ---------------------------------------------------------------------------
// Inline edit
// ---------------------------------------------------------------------------

export interface SelectOption {
  value: string
  label: string
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

export interface IssueComment {
  id: number
  body: string
  author: { username: string; name: string }
  created_at: string
}

// ---------------------------------------------------------------------------
// Connect repo
// ---------------------------------------------------------------------------

export interface ConnectRepoData {
  gitlab_url: string
  gitlab_project_id: string
  gitlab_access_token: string
  path?: string
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/packages/@sdd/ui/src/types.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
