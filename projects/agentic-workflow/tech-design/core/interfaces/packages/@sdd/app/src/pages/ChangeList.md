---
id: projects-sdd-packages-sdd-app-src-pages-changelist-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PHASE_LABELS` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | constant | pub | 8 |  |
| `[changes, setChanges]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | constant | pub | 35 |  |
| `[error, setError]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | constant | pub | 37 |  |
| `[loading, setLoading]` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | constant | pub | 36 |  |
| `ds` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | constant | pub | 34 |  |
| `phaseBadgeClass` | projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx | function | pub | 20 | phaseBadgeClass(phase: ChangePhase) |
## Source
<!-- type: source lang: tsx -->

```tsx
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

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/app/src/pages/ChangeList.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
