---
id: projects-sdd-packages-sdd-pipeline-src-pipelinedag-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ZOOM_THRESHOLD` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 9 |  |
| `[internalSelectedId, setInternalSelectedId]` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 28 |  |
| `[zoom, setZoom]` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 32 |  |
| `containerRef` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 31 |  |
| `cycleNodes` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 38 |  |
| `handleNodeClick` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 45 |  |
| `hasCycles` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 39 |  |
| `isLargeGraph` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 53 |  |
| `lNode` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 199 |  |
| `layout` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 42 |  |
| `midY` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 180 |  |
| `node` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 57 |  |
| `nodeMap` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 43 |  |
| `nodes` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 35 |  |
| `selectedId` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 29 |  |
| `selectedNode` | projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx | constant | pub | 51 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useMemo, useRef, useCallback } from 'react'
import type { PipelineDAGProps, PipelineNode } from './types'
import PipelineNodeCard from './PipelineNode'
import NodeDetail from './NodeDetail'
import { computeLayout, detectCycles, applySkippedStatus } from './layout'

const ZOOM_THRESHOLD = 50 // Enable zoom/pan when nodes > this count (REQ-PL-05)

/**
 * Pipeline DAG visualization.
 *
 * - Sequential nodes render top-to-bottom (REQ-PL-03)
 * - Parallel nodes render side-by-side (REQ-PL-03)
 * - Single node renders as simple card without edges (REQ-PL-08)
 * - Click node expands detail panel (REQ-PL-04)
 * - Live updates via React key stability (REQ-PL-06)
 * - Circular dependency detection with warning (REQ-PL-07)
 * - Failed node marks downstream as skipped (REQ-PL-09)
 */
export default function PipelineDAG({
  nodes: rawNodes,
  onNodeClick,
  selectedNodeId: controlledSelectedId,
  className = '',
}: PipelineDAGProps) {
  const [internalSelectedId, setInternalSelectedId] = useState<string | null>(null)
  const selectedId = controlledSelectedId !== undefined ? controlledSelectedId : internalSelectedId

  const containerRef = useRef<HTMLDivElement>(null)
  const [zoom, setZoom] = useState(1)

  // Apply skipped status to downstream of failed nodes (REQ-PL-09)
  const nodes = useMemo(() => applySkippedStatus(rawNodes), [rawNodes])

  // Detect cycles (REQ-PL-07)
  const cycleNodes = useMemo(() => detectCycles(nodes), [nodes])
  const hasCycles = cycleNodes.size > 0

  // Compute layout
  const layout = useMemo(() => computeLayout(nodes), [nodes])
  const nodeMap = useMemo(() => new Map(nodes.map(n => [n.id, n])), [nodes])

  const handleNodeClick = useCallback((node: PipelineNode) => {
    const newId = selectedId === node.id ? null : node.id
    setInternalSelectedId(newId)
    onNodeClick?.(node)
  }, [selectedId, onNodeClick])

  const selectedNode = selectedId ? nodeMap.get(selectedId) ?? null : null

  const isLargeGraph = nodes.length >= ZOOM_THRESHOLD

  // --- Single-node pipeline: simple card view (REQ-PL-08) ---
  if (nodes.length === 1) {
    const node = nodes[0]
    return (
      <div className={className} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div style={{ position: 'relative', width: 200, height: 60 }}>
          <PipelineNodeCard
            node={node}
            isSelected={selectedId === node.id}
            onClick={handleNodeClick}
            style={{ left: 0, top: 0, width: 200, height: 56 }}
          />
        </div>
        {selectedNode && (
          <NodeDetail node={selectedNode} onClose={() => setInternalSelectedId(null)} />
        )}
      </div>
    )
  }

  // --- Empty pipeline ---
  if (nodes.length === 0) {
    return (
      <div className={className} style={{ padding: 24, textAlign: 'center', color: '#9ca3af', fontSize: 14 }}>
        No pipeline nodes
      </div>
    )
  }

  // --- Full DAG view ---
  const svgWidth = layout.width
  const svgHeight = layout.height
  const padding = 20

  return (
    <div className={className} style={{ display: 'flex', gap: 16, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif' }}>
      <div style={{ flex: 1, minWidth: 0 }}>
        {/* Cycle warning (REQ-PL-07) */}
        {hasCycles && (
          <div style={{
            background: '#fffbeb', border: '1px solid #fde68a', borderRadius: 6,
            padding: '8px 12px', marginBottom: 8, display: 'flex', alignItems: 'center', gap: 8,
          }}>
            <span style={{ fontSize: 16 }} role="img" aria-label="warning">!</span>
            <span style={{ fontSize: 13, color: '#92400e' }}>
              Circular dependency detected in nodes: {Array.from(cycleNodes).join(', ')}
            </span>
          </div>
        )}

        {/* Zoom controls for large graphs (REQ-PL-05) */}
        {isLargeGraph && (
          <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
            <button
              onClick={() => setZoom(z => Math.max(0.2, z - 0.1))}
              style={{
                padding: '4px 8px', border: '1px solid #d1d5db', borderRadius: 4,
                background: '#fff', cursor: 'pointer', fontSize: 13,
              }}
            >
              -
            </button>
            <span style={{ fontSize: 12, color: '#6b7280', lineHeight: '28px' }}>
              {Math.round(zoom * 100)}%
            </span>
            <button
              onClick={() => setZoom(z => Math.min(2, z + 0.1))}
              style={{
                padding: '4px 8px', border: '1px solid #d1d5db', borderRadius: 4,
                background: '#fff', cursor: 'pointer', fontSize: 13,
              }}
            >
              +
            </button>
            <button
              onClick={() => setZoom(1)}
              style={{
                padding: '4px 8px', border: '1px solid #d1d5db', borderRadius: 4,
                background: '#fff', cursor: 'pointer', fontSize: 13,
              }}
            >
              Reset
            </button>
          </div>
        )}

        {/* DAG canvas */}
        <div
          ref={containerRef}
          style={{
            overflow: isLargeGraph ? 'auto' : 'visible',
            maxHeight: isLargeGraph ? 600 : undefined,
            border: '1px solid #e5e7eb',
            borderRadius: 8,
            background: '#fafafa',
          }}
        >
          <div style={{
            position: 'relative',
            width: svgWidth * zoom,
            height: svgHeight * zoom,
            transform: `scale(${zoom})`,
            transformOrigin: 'top left',
            minWidth: svgWidth,
            minHeight: svgHeight,
          }}>
            {/* SVG edges */}
            <svg
              width={svgWidth}
              height={svgHeight}
              style={{ position: 'absolute', top: 0, left: 0, pointerEvents: 'none' }}
            >
              <defs>
                <marker
                  id="arrowhead"
                  markerWidth="8"
                  markerHeight="6"
                  refX="8"
                  refY="3"
                  orient="auto"
                >
                  <polygon points="0 0, 8 3, 0 6" fill="#9ca3af" />
                </marker>
              </defs>
              {layout.edges.map((edge, idx) => {
                const midY = (edge.fromY + edge.toY) / 2
                return (
                  <path
                    key={idx}
                    d={`M ${edge.fromX + padding} ${edge.fromY + padding} C ${edge.fromX + padding} ${midY}, ${edge.toX + padding} ${midY}, ${edge.toX + padding} ${edge.toY + padding - 6}`}
                    fill="none"
                    stroke={
                      cycleNodes.has(edge.from) && cycleNodes.has(edge.to) ? '#f59e0b' : '#d1d5db'
                    }
                    strokeWidth={2}
                    strokeDasharray={cycleNodes.has(edge.from) && cycleNodes.has(edge.to) ? '4 4' : undefined}
                    markerEnd="url(#arrowhead)"
                  />
                )
              })}
            </svg>

            {/* Node cards */}
            {nodes.map((node) => {
              const lNode = layout.nodes.get(node.id)
              if (!lNode) return null
              return (
                <PipelineNodeCard
                  key={node.id}
                  node={node}
                  isSelected={selectedId === node.id}
                  onClick={handleNodeClick}
                  style={{
                    left: lNode.x + padding,
                    top: lNode.y + padding,
                    width: lNode.width,
                    height: lNode.height,
                  }}
                />
              )
            })}
          </div>
        </div>
      </div>

      {/* Detail panel */}
      {selectedNode && (
        <div style={{ flexShrink: 0 }}>
          <NodeDetail node={selectedNode} onClose={() => setInternalSelectedId(null)} />
        </div>
      )}

      {/* Global animation styles */}
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/pipeline/src/PipelineDAG.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
