---
id: projects-sdd-packages-sdd-app-src-pages-changedetail-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PHASES` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 8 |  |
| `PHASE_LABELS` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 20 |  |
| `[change, setChange]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 35 |  |
| `[error, setError]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 37 |  |
| `[loading, setLoading]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 36 |  |
| `currentIndex` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 62 |  |
| `ds` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 34 |  |
| `isCompleted` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 89 |  |
| `isCurrent` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 88 |  |
| `{ id }` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx | constant | pub | 33 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { Change, ChangePhase } from '@sdd/core'

const PHASES: ChangePhase[] = [
  'init',
  'restructure',
  'pre_clarify',
  'reference_context',
  'post_clarify',
  'change_spec',
  'implementation',
  'review',
  'merge',
]

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

export default function ChangeDetail() {
  const { id } = useParams<{ id: string }>()
  const ds = useDataSource()
  const [change, setChange] = useState<Change | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) return
    ds.getChange(id)
      .then(setChange)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [ds, id])

  if (loading) {
    return <div className="h-64 bg-gray-200 rounded-lg animate-pulse" />
  }

  if (error || !change) {
    return (
      <div className="p-6 text-center text-red-600">
        <p>{error || 'Change not found'}</p>
        <Link to=".." className="text-sm text-gray-500 hover:text-gray-700 mt-2 inline-block">
          Back to list
        </Link>
      </div>
    )
  }

  const currentIndex = PHASES.indexOf(change.phase)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Link
          to=".."
          className="text-gray-500 hover:text-gray-900 transition-colors"
          aria-label="Back to changes"
        >
          &larr;
        </Link>
        <div className="flex-1 min-w-0">
          <h1 className="text-2xl font-bold text-gray-900 font-mono">{change.id}</h1>
          {change.description && (
            <p className="text-sm text-gray-500 mt-1">{change.description}</p>
          )}
        </div>
      </div>

      {/* Phase Timeline */}
      <div className="bg-white border rounded-lg p-6">
        <h2 className="text-sm font-medium text-gray-700 mb-4">Phase Timeline</h2>
        <div className="flex items-center gap-1 overflow-x-auto pb-2">
          {PHASES.map((phase, index) => {
            const isCurrent = phase === change.phase
            const isCompleted = index < currentIndex

            return (
              <div key={phase} className="flex items-center">
                <div className="flex flex-col items-center min-w-[80px]">
                  <div
                    className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium ${
                      isCurrent
                        ? 'bg-blue-600 text-white ring-2 ring-blue-300'
                        : isCompleted
                        ? 'bg-green-500 text-white'
                        : 'bg-gray-200 text-gray-500'
                    }`}
                  >
                    {isCompleted ? '\u2713' : index + 1}
                  </div>
                  <span
                    className={`mt-1.5 text-xs text-center ${
                      isCurrent ? 'text-blue-600 font-medium' : 'text-gray-500'
                    }`}
                  >
                    {PHASE_LABELS[phase]}
                  </span>
                </div>
                {index < PHASES.length - 1 && (
                  <div
                    className={`w-6 h-0.5 mt-[-16px] ${
                      isCompleted ? 'bg-green-500' : 'bg-gray-200'
                    }`}
                  />
                )}
              </div>
            )
          })}
        </div>
      </div>

      {/* Metadata */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {change.issue_ids.length > 0 && (
          <div className="bg-white border rounded-lg p-4">
            <h3 className="text-sm font-medium text-gray-700 mb-2">Linked Issues</h3>
            <div className="flex flex-wrap gap-2">
              {change.issue_ids.map((issueId) => (
                <Link
                  key={issueId}
                  to={`../issues/${encodeURIComponent(issueId)}`}
                  className="text-xs font-mono text-blue-600 hover:text-blue-800 bg-blue-50 px-2 py-1 rounded"
                >
                  {issueId}
                </Link>
              ))}
            </div>
          </div>
        )}

        {change.spec_ids.length > 0 && (
          <div className="bg-white border rounded-lg p-4">
            <h3 className="text-sm font-medium text-gray-700 mb-2">Spec Artifacts</h3>
            <div className="flex flex-wrap gap-2">
              {change.spec_ids.map((specId) => (
                <Link
                  key={specId}
                  to={`../tech-designs/${encodeURIComponent(specId)}`}
                  className="text-xs font-mono text-blue-600 hover:text-blue-800 bg-blue-50 px-2 py-1 rounded"
                >
                  {specId}
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeDetail.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
