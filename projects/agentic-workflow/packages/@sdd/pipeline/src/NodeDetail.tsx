// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/pipeline/src/NodeDetail.md#source
// CODEGEN-BEGIN
import { useState } from 'react'
import type { NodeDetailProps, PipelineJob, NodeStatus } from './types'

const STATUS_BADGE: Record<NodeStatus, { bg: string; color: string }> = {
  pending:   { bg: '#f3f4f6', color: '#6b7280' },
  running:   { bg: '#dbeafe', color: '#1d4ed8' },
  completed: { bg: '#dcfce7', color: '#15803d' },
  failed:    { bg: '#fee2e2', color: '#dc2626' },
  skipped:   { bg: '#f3f4f6', color: '#9ca3af' },
}

function StatusBadge({ status }: { status: NodeStatus }) {
  const style = STATUS_BADGE[status] ?? STATUS_BADGE.pending
  return (
    <span style={{
      display: 'inline-block', padding: '2px 8px', borderRadius: 4,
      fontSize: 11, fontWeight: 600, textTransform: 'capitalize',
      background: style.bg, color: style.color,
    }}>
      {status}
    </span>
  )
}

function JobRow({ job }: { job: PipelineJob }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 8,
      padding: '6px 0', borderBottom: '1px solid #f3f4f6',
    }}>
      <StatusBadge status={job.status} />
      <span style={{ flex: 1, fontSize: 13, fontWeight: 500 }}>{job.label}</span>
      {job.model && (
        <span style={{ fontSize: 11, color: '#6b7280', fontFamily: 'monospace' }}>{job.model}</span>
      )}
      {job.tokens_used != null && (
        <span style={{ fontSize: 11, color: '#9ca3af' }}>{job.tokens_used.toLocaleString()} tokens</span>
      )}
    </div>
  )
}

function formatDuration(startIso?: string, endIso?: string): string | null {
  if (!startIso) return null
  const start = new Date(startIso).getTime()
  const end = endIso ? new Date(endIso).getTime() : Date.now()
  const diffMs = end - start
  if (diffMs < 1000) return '<1s'
  if (diffMs < 60000) return `${Math.round(diffMs / 1000)}s`
  return `${Math.floor(diffMs / 60000)}m ${Math.round((diffMs % 60000) / 1000)}s`
}

/**
 * Expanded node detail view showing status, timing, jobs, artifacts, errors (REQ-PL-04).
 */
export default function NodeDetail({ node, onClose }: NodeDetailProps) {
  const [jobsExpanded, setJobsExpanded] = useState(false)
  const jobs = node.jobs || []
  const duration = formatDuration(node.started_at, node.completed_at)
  const showJobsCollapsed = jobs.length > 5

  return (
    <div style={{
      border: '1px solid #e5e7eb',
      borderRadius: 8,
      background: '#ffffff',
      padding: 16,
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      maxWidth: 400,
    }}>
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
        <h3 style={{ fontSize: 15, fontWeight: 600, margin: 0 }}>{node.label}</h3>
        <button
          onClick={onClose}
          style={{
            background: 'none', border: 'none', cursor: 'pointer',
            fontSize: 18, color: '#9ca3af', lineHeight: 1,
          }}
          aria-label="Close detail"
        >
          x
        </button>
      </div>

      {/* Status + timing */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, marginBottom: 12 }}>
        <div>
          <div style={{ fontSize: 11, color: '#9ca3af', marginBottom: 2 }}>Status</div>
          <StatusBadge status={node.status} />
        </div>
        {node.started_at && (
          <div>
            <div style={{ fontSize: 11, color: '#9ca3af', marginBottom: 2 }}>Started</div>
            <div style={{ fontSize: 12 }}>{new Date(node.started_at).toLocaleTimeString()}</div>
          </div>
        )}
        {duration && (
          <div>
            <div style={{ fontSize: 11, color: '#9ca3af', marginBottom: 2 }}>Duration</div>
            <div style={{ fontSize: 12 }}>{duration}</div>
          </div>
        )}
      </div>

      {/* Error */}
      {node.error && (
        <div style={{
          background: '#fef2f2', border: '1px solid #fecaca', borderRadius: 6,
          padding: '8px 12px', marginBottom: 12,
        }}>
          <div style={{ fontSize: 11, fontWeight: 600, color: '#dc2626', marginBottom: 2 }}>Error</div>
          <div style={{ fontSize: 12, color: '#991b1b', whiteSpace: 'pre-wrap' }}>{node.error}</div>
        </div>
      )}

      {/* Jobs list (REQ-PL-04) */}
      {jobs.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
            <div style={{ fontSize: 12, fontWeight: 600, color: '#374151' }}>
              Jobs ({jobs.length})
            </div>
            {showJobsCollapsed && (
              <button
                onClick={() => setJobsExpanded(!jobsExpanded)}
                style={{
                  background: 'none', border: 'none', cursor: 'pointer',
                  fontSize: 11, color: '#6b7280',
                }}
              >
                {jobsExpanded ? 'Collapse' : 'Expand all'}
              </button>
            )}
          </div>
          <div>
            {(showJobsCollapsed && !jobsExpanded ? jobs.slice(0, 5) : jobs).map((job) => (
              <JobRow key={job.id} job={job} />
            ))}
            {showJobsCollapsed && !jobsExpanded && (
              <div style={{ fontSize: 11, color: '#9ca3af', padding: '4px 0' }}>
                +{jobs.length - 5} more jobs
              </div>
            )}
          </div>
        </div>
      )}

      {/* Artifact preview */}
      {node.artifact && (
        <div>
          <div style={{ fontSize: 12, fontWeight: 600, color: '#374151', marginBottom: 4 }}>Artifact</div>
          <pre style={{
            fontSize: 11, background: '#f9fafb', padding: 8, borderRadius: 6,
            overflow: 'auto', maxHeight: 200,
            fontFamily: 'ui-monospace, SFMono-Regular, monospace',
            color: '#374151',
          }}>
            {typeof node.artifact === 'string' ? node.artifact : JSON.stringify(node.artifact, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}


// CODEGEN-END
