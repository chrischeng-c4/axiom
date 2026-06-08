---
id: projects-sdd-packages-sdd-app-src-pages-issuelist-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `STATUS_TABS` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 8 |  |
| `[error, setError]` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 25 |  |
| `[issues, setIssues]` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 23 |  |
| `[loading, setLoading]` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 24 |  |
| `[statusFilter, setStatusFilter]` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 26 |  |
| `ds` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 22 |  |
| `filtered` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | constant | pub | 35 |  |
| `statusBadgeClass` | projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx | function | pub | 10 | statusBadgeClass(status: IssueStatus) |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { IssueSummary, IssueStatus } from '@sdd/core'

const STATUS_TABS: Array<'all' | IssueStatus> = ['all', 'open', 'in_progress', 'closed']

function statusBadgeClass(status: IssueStatus): string {
  switch (status) {
    case 'open':
      return 'bg-green-100 text-green-800'
    case 'in_progress':
      return 'bg-yellow-100 text-yellow-800'
    case 'closed':
      return 'bg-gray-100 text-gray-700'
  }
}

export default function IssueList() {
  const ds = useDataSource()
  const [issues, setIssues] = useState<IssueSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [statusFilter, setStatusFilter] = useState<'all' | IssueStatus>('all')

  useEffect(() => {
    ds.listIssues()
      .then(setIssues)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [ds])

  const filtered = statusFilter === 'all'
    ? issues
    : issues.filter((i) => i.status === statusFilter)

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
        <p>Failed to load issues: {error}</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Issues</h1>

      {/* Status tabs */}
      <div className="border-b flex gap-4" role="tablist" aria-label="Issue status filter">
        {STATUS_TABS.map((tab) => (
          <button
            key={tab}
            role="tab"
            aria-selected={statusFilter === tab}
            onClick={() => setStatusFilter(tab)}
            className={`py-2 text-sm font-medium transition-colors cursor-pointer ${
              statusFilter === tab
                ? 'border-b-2 border-blue-600 text-blue-600'
                : 'text-gray-500 hover:text-gray-900'
            }`}
          >
            {tab === 'in_progress' ? 'In Progress' : tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      {filtered.length === 0 ? (
        <div className="p-8 text-center border rounded-lg bg-white">
          <p className="text-sm text-gray-500">No issues found.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((issue) => (
            <Link
              key={issue.id}
              to={`issues/${encodeURIComponent(issue.id)}`}
              className="block p-4 bg-white border rounded-lg hover:shadow transition-shadow"
            >
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-3 min-w-0">
                  <span className="text-sm text-gray-500 font-mono flex-shrink-0">
                    #{issue.issue_number}
                  </span>
                  <span className="font-medium text-gray-900 truncate">{issue.title}</span>
                  {issue.priority && (
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                      issue.priority === 'critical' ? 'bg-red-100 text-red-800' :
                      issue.priority === 'high' ? 'bg-orange-100 text-orange-800' :
                      'bg-gray-100 text-gray-700'
                    }`}>
                      {issue.priority}
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  {issue.labels.slice(0, 3).map((label) => (
                    <span key={label} className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded">
                      {label}
                    </span>
                  ))}
                  <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${statusBadgeClass(issue.status)}`}>
                    {issue.status === 'in_progress' ? 'in progress' : issue.status}
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
  - path: projects/agentic-workflow/packages/@sdd/app/src/pages/IssueList.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
