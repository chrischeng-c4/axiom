---
id: projects-sdd-packages-sdd-pipeline-src-types-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LayoutEdge` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 71 |  |
| `LayoutNode` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 61 |  |
| `LayoutResult` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 80 |  |
| `NodeDetailProps` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 55 |  |
| `NodeStatus` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | type | pub | 3 |  |
| `PipelineDAGProps` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 37 |  |
| `PipelineJob` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 5 |  |
| `PipelineNode` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 16 |  |
| `PipelineNodeProps` | projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts | interface | pub | 48 |  |
## Source
<!-- type: source lang: ts -->

```ts
export type NodeStatus = 'pending' | 'running' | 'completed' | 'failed' | 'skipped'

export interface PipelineJob {
  id: string
  label: string
  status: NodeStatus
  model?: string
  tokens_used?: number
  error?: string | null
  started_at?: string
  completed_at?: string
}

export interface PipelineNode {
  /** Unique node identifier */
  id: string
  /** Display label */
  label: string
  /** Current execution status */
  status: NodeStatus
  /** IDs of dependency nodes (must complete before this node starts) */
  deps: string[]
  /** Jobs within this node */
  jobs?: PipelineJob[]
  /** Artifact preview (any serializable object) */
  artifact?: Record<string, unknown> | null
  /** Error message if status is 'failed' */
  error?: string | null
  /** Start time ISO string */
  started_at?: string
  /** Completion time ISO string */
  completed_at?: string
}

export interface PipelineDAGProps {
  /** Nodes in the pipeline */
  nodes: PipelineNode[]
  /** Callback when a node is clicked */
  onNodeClick?: (node: PipelineNode) => void
  /** Currently selected node ID */
  selectedNodeId?: string | null
  /** Optional CSS class name */
  className?: string
}

export interface PipelineNodeProps {
  node: PipelineNode
  isSelected: boolean
  onClick: (node: PipelineNode) => void
  style?: React.CSSProperties
}

export interface NodeDetailProps {
  node: PipelineNode
  onClose: () => void
}

/** Internal layout types */
export interface LayoutNode {
  id: string
  row: number
  col: number
  x: number
  y: number
  width: number
  height: number
}

export interface LayoutEdge {
  from: string
  to: string
  fromX: number
  fromY: number
  toX: number
  toY: number
}

export interface LayoutResult {
  nodes: Map<string, LayoutNode>
  edges: LayoutEdge[]
  width: number
  height: number
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/pipeline/src/types.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
