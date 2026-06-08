---
id: projects-sdd-packages-sdd-pipeline-src-layout-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `H_GAP` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | constant | pub | 7 |  |
| `NODE_HEIGHT` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | constant | pub | 6 |  |
| `NODE_WIDTH` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | constant | pub | 5 |  |
| `V_GAP` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | constant | pub | 8 |  |
| `applySkippedStatus` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | function | pub | 52 | applySkippedStatus(nodes: PipelineNode[]) |
| `computeLayout` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | function | pub | 89 | computeLayout(nodes: PipelineNode[]) |
| `detectCycles` | projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts | function | pub | 14 | detectCycles(nodes: PipelineNode[]) |
## Source
<!-- type: source lang: ts -->

```ts
import type { PipelineNode, LayoutNode, LayoutEdge, LayoutResult } from './types'

const NODE_WIDTH = 180
const NODE_HEIGHT = 56
const H_GAP = 40
const V_GAP = 60

/**
 * Detect cycles in the dependency graph (REQ-PL-07).
 * Returns set of node IDs involved in cycles.
 */
export function detectCycles(nodes: PipelineNode[]): Set<string> {
  const nodeMap = new Map(nodes.map(n => [n.id, n]))
  const cycleNodes = new Set<string>()
  const visited = new Set<string>()
  const stack = new Set<string>()

  function dfs(id: string): boolean {
    if (stack.has(id)) {
      cycleNodes.add(id)
      return true
    }
    if (visited.has(id)) return false
    visited.add(id)
    stack.add(id)

    const node = nodeMap.get(id)
    if (node) {
      for (const depId of node.deps) {
        if (dfs(depId)) {
          cycleNodes.add(id)
        }
      }
    }

    stack.delete(id)
    return false
  }

  for (const node of nodes) {
    dfs(node.id)
  }
  return cycleNodes
}

/**
 * Mark downstream nodes of failed nodes as "skipped" (REQ-PL-09).
 * Returns a new array with updated statuses (does not mutate input).
 */
export function applySkippedStatus(nodes: PipelineNode[]): PipelineNode[] {
  const failedIds = new Set(nodes.filter(n => n.status === 'failed').map(n => n.id))
  if (failedIds.size === 0) return nodes

  // Build reverse adjacency: node -> nodes that depend on it
  const dependents = new Map<string, string[]>()
  for (const n of nodes) {
    for (const dep of n.deps) {
      if (!dependents.has(dep)) dependents.set(dep, [])
      dependents.get(dep)!.push(n.id)
    }
  }

  // BFS from failed nodes to mark downstream as skipped
  const skippedIds = new Set<string>()
  const queue = [...failedIds]
  while (queue.length > 0) {
    const current = queue.shift()!
    const children = dependents.get(current) || []
    for (const childId of children) {
      if (!skippedIds.has(childId) && !failedIds.has(childId)) {
        skippedIds.add(childId)
        queue.push(childId)
      }
    }
  }

  return nodes.map(n =>
    skippedIds.has(n.id) ? { ...n, status: 'skipped' as const } : n
  )
}

/**
 * Simple topological sort + row/column assignment.
 * Nodes with no deps go to row 0. Each node's row = max(dep rows) + 1.
 * Within each row, nodes are laid out side by side.
 */
export function computeLayout(nodes: PipelineNode[]): LayoutResult {
  const nodeMap = new Map(nodes.map(n => [n.id, n]))
  const cycleNodes = detectCycles(nodes)

  // Assign rows via topological ordering
  const rowMap = new Map<string, number>()

  function getRow(id: string, visiting = new Set<string>()): number {
    if (rowMap.has(id)) return rowMap.get(id)!
    if (visiting.has(id) || cycleNodes.has(id)) {
      // Cycle detected -- assign row 0 to break infinite recursion
      rowMap.set(id, 0)
      return 0
    }
    visiting.add(id)

    const node = nodeMap.get(id)
    if (!node || node.deps.length === 0) {
      rowMap.set(id, 0)
      return 0
    }

    let maxDepRow = -1
    for (const depId of node.deps) {
      if (nodeMap.has(depId)) {
        maxDepRow = Math.max(maxDepRow, getRow(depId, visiting))
      }
    }
    const row = maxDepRow + 1
    rowMap.set(id, row)
    return row
  }

  for (const node of nodes) {
    getRow(node.id)
  }

  // Group nodes by row
  const rowGroups = new Map<number, string[]>()
  for (const [id, row] of rowMap) {
    if (!rowGroups.has(row)) rowGroups.set(row, [])
    rowGroups.get(row)!.push(id)
  }

  const maxRow = Math.max(0, ...rowGroups.keys())
  const maxCols = Math.max(1, ...Array.from(rowGroups.values()).map(g => g.length))

  // Compute positions
  const layoutNodes = new Map<string, LayoutNode>()

  for (let row = 0; row <= maxRow; row++) {
    const ids = rowGroups.get(row) || []
    const rowWidth = ids.length * NODE_WIDTH + (ids.length - 1) * H_GAP
    const startX = (maxCols * (NODE_WIDTH + H_GAP) - H_GAP - rowWidth) / 2

    ids.forEach((id, col) => {
      layoutNodes.set(id, {
        id,
        row,
        col,
        x: startX + col * (NODE_WIDTH + H_GAP),
        y: row * (NODE_HEIGHT + V_GAP),
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
      })
    })
  }

  // Compute edges
  const edges: LayoutEdge[] = []
  for (const node of nodes) {
    const toLayout = layoutNodes.get(node.id)
    if (!toLayout) continue
    for (const depId of node.deps) {
      const fromLayout = layoutNodes.get(depId)
      if (!fromLayout) continue
      edges.push({
        from: depId,
        to: node.id,
        fromX: fromLayout.x + fromLayout.width / 2,
        fromY: fromLayout.y + fromLayout.height,
        toX: toLayout.x + toLayout.width / 2,
        toY: toLayout.y,
      })
    }
  }

  const totalWidth = maxCols * (NODE_WIDTH + H_GAP) - H_GAP + 40 // 20px padding each side
  const totalHeight = (maxRow + 1) * (NODE_HEIGHT + V_GAP) - V_GAP + 40

  return { nodes: layoutNodes, edges, width: totalWidth, height: totalHeight }
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/pipeline/src/layout.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
