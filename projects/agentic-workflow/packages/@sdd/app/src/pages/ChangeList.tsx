// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/pages/ChangeList.md#source
// CODEGEN-BEGIN
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { ChangeSummary, ChangePhase } from '@sdd/core'

const PHASE_LABELS: Record<ChangePhase, string> = {
  init: 'Init',
  restructure: 'Restructure',
  pre_clarify: 'Pre-clarify',
  reference_context: 'Ref Context',
  post_clarify: 'Post-clarify',
  change_spec: 'Change Spec',
  implementation: 'Implementation',
  review: 'Review',
  merge: 'Merge',
}

function phaseBadgeClass(phase: ChangePhase): string {
  switch (phase) {
    case 'merge':
      return 'bg-green-100 text-green-800'
    case 'review':
      return 'bg-blue-100 text-blue-800'
    case 'implementation':
      return 'bg-yellow-100 text-yellow-800'
    default:
      return 'bg-gray-100 text-gray-700'
  }
}

export default function ChangeList() {
  const ds = useDataSource()
  const [changes, setChanges] = useState<ChangeSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    ds.listChanges()
      .then(setChanges)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [ds])

  if (loading) {
    return (
      <div className="space-y-2">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-14 bg-gray-200 rounded-lg animate-pulse" />
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className="p-6 text-center text-red-600">
        <p>Failed to load changes: {error}</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Changes</h1>

      {changes.length === 0 ? (
        <div className="p-8 text-center border rounded-lg bg-white">
          <p className="text-sm text-gray-500">No changes found.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {changes.map((c) => (
            <Link
              key={c.id}
              to={`changes/${encodeURIComponent(c.id)}`}
              className="block p-4 bg-white border rounded-lg hover:shadow transition-shadow"
            >
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-3 min-w-0">
                  <span className="font-medium text-gray-900 font-mono">{c.id}</span>
                  {c.description && (
                    <span className="text-sm text-gray-500 truncate">{c.description}</span>
                  )}
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  {c.issue_ids.length > 0 && (
                    <span className="text-xs text-gray-400">
                      {c.issue_ids.length} issue{c.issue_ids.length !== 1 ? 's' : ''}
                    </span>
                  )}
                  <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${phaseBadgeClass(c.phase)}`}>
                    {PHASE_LABELS[c.phase]}
                  </span>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}


// CODEGEN-END
