// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/types.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
