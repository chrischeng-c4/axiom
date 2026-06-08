---
id: projects-sdd-packages-sdd-core-src-types-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/core/src/types.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/core/src/types.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Change` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 49 |  |
| `ChangePhase` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | type | pub | 38 |  |
| `ChangeSummary` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 59 |  |
| `Issue` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 76 |  |
| `IssuePriority` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | type | pub | 74 |  |
| `IssueStatus` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | type | pub | 72 |  |
| `IssueSummary` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 89 |  |
| `LineageEdge` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 111 |  |
| `LineageGraph` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 117 |  |
| `LineageNode` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 105 |  |
| `LineageNodeKind` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | type | pub | 103 |  |
| `ProjectInfo` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 126 |  |
| `TechDesign` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 12 |  |
| `TechDesignSummary` | projects/agentic-workflow/packages/@sdd/core/src/types.ts | interface | pub | 26 |  |
## Source
<!-- type: source lang: ts -->

```ts
/**
 * Shared TypeScript types for the SDD frontend.
 * These types are backend-agnostic -- used by both LocalDataSource and RemoteDataSource.
 */

// ---------------------------------------------------------------------------
// Tech Designs (specs)
// ---------------------------------------------------------------------------

export interface TechDesign {
  id: string
  /** Crate or module this spec belongs to */
  crate: string
  /** Relative path within .aw/tech-design/ */
  path: string
  /** Human-readable title derived from filename or frontmatter */
  title: string
  /** Spec content (Markdown) */
  content: string
  /** Last modification timestamp (ISO 8601) */
  updated_at: string
}

export interface TechDesignSummary {
  id: string
  crate: string
  path: string
  title: string
  updated_at: string
}

// ---------------------------------------------------------------------------
// Changes
// ---------------------------------------------------------------------------

export type ChangePhase =
  | 'init'
  | 'restructure'
  | 'pre_clarify'
  | 'reference_context'
  | 'post_clarify'
  | 'change_spec'
  | 'implementation'
  | 'review'
  | 'merge'

export interface Change {
  id: string
  description: string | null
  phase: ChangePhase
  issue_ids: string[]
  spec_ids: string[]
  created_at: string
  updated_at: string
}

export interface ChangeSummary {
  id: string
  description: string | null
  phase: ChangePhase
  issue_ids: string[]
  created_at: string
  updated_at: string
}

// ---------------------------------------------------------------------------
// Issues
// ---------------------------------------------------------------------------

export type IssueStatus = 'open' | 'in_progress' | 'closed'

export type IssuePriority = 'low' | 'medium' | 'high' | 'critical'

export interface Issue {
  id: string
  issue_number: number
  title: string
  description: string | null
  status: IssueStatus
  priority: IssuePriority | null
  labels: string[]
  created_at: string
  updated_at: string
  closed_at: string | null
}

export interface IssueSummary {
  id: string
  issue_number: number
  title: string
  status: IssueStatus
  priority: IssuePriority | null
  labels: string[]
  created_at: string
}

// ---------------------------------------------------------------------------
// Lineage
// ---------------------------------------------------------------------------

export type LineageNodeKind = 'issue' | 'change' | 'spec' | 'artifact'

export interface LineageNode {
  id: string
  kind: LineageNodeKind
  label: string
}

export interface LineageEdge {
  from: string
  to: string
  relation: string
}

export interface LineageGraph {
  nodes: LineageNode[]
  edges: LineageEdge[]
}

// ---------------------------------------------------------------------------
// Project Info
// ---------------------------------------------------------------------------

export interface ProjectInfo {
  name: string
  root: string
  has_score: boolean
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/core/src/types.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
