// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/pipeline/src/types.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
