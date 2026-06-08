// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/pipeline/src/PipelineNode.md#source
// CODEGEN-BEGIN
import type { PipelineNodeProps, NodeStatus } from './types'

const STATUS_COLORS: Record<NodeStatus, { bg: string; border: string; text: string; dot: string }> = {
  pending:   { bg: '#f9fafb', border: '#d1d5db', text: '#6b7280', dot: '#9ca3af' },
  running:   { bg: '#eff6ff', border: '#93c5fd', text: '#1d4ed8', dot: '#3b82f6' },
  completed: { bg: '#f0fdf4', border: '#86efac', text: '#15803d', dot: '#22c55e' },
  failed:    { bg: '#fef2f2', border: '#fca5a5', text: '#dc2626', dot: '#ef4444' },
  skipped:   { bg: '#f9fafb', border: '#e5e7eb', text: '#9ca3af', dot: '#d1d5db' },
}

/**
 * Individual pipeline node card with status color coding (REQ-PL-02).
 */
export default function PipelineNodeCard({ node, isSelected, onClick, style }: PipelineNodeProps) {
  const colors = STATUS_COLORS[node.status] ?? STATUS_COLORS.pending

  return (
    <button
      onClick={() => onClick(node)}
      style={{
        position: 'absolute',
        ...style,
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '8px 12px',
        background: colors.bg,
        border: `2px solid ${isSelected ? '#6366f1' : colors.border}`,
        borderRadius: 8,
        cursor: 'pointer',
        outline: 'none',
        width: style?.width ?? 180,
        boxSizing: 'border-box',
        transition: 'border-color 0.2s, box-shadow 0.2s',
        boxShadow: isSelected ? '0 0 0 2px rgba(99,102,241,0.3)' : '0 1px 3px rgba(0,0,0,0.08)',
        borderStyle: node.status === 'skipped' ? 'dashed' : 'solid',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      }}
    >
      {/* Status dot / spinner */}
      <span style={{
        width: 10, height: 10, borderRadius: '50%',
        background: node.status === 'running' ? undefined : colors.dot,
        border: node.status === 'running' ? `2px solid ${colors.dot}` : undefined,
        borderTopColor: node.status === 'running' ? 'transparent' : undefined,
        animation: node.status === 'running' ? 'spin 1s linear infinite' : undefined,
        flexShrink: 0,
      }} />
      <div style={{ overflow: 'hidden', textAlign: 'left', flex: 1 }}>
        <div style={{
          fontSize: 13, fontWeight: 600, color: colors.text,
          whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
        }}>
          {node.label}
        </div>
        <div style={{ fontSize: 11, color: '#9ca3af', textTransform: 'capitalize' }}>
          {node.status}
        </div>
      </div>
    </button>
  )
}


// CODEGEN-END
