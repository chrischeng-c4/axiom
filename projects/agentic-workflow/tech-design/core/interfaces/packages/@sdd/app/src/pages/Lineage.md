---
id: projects-sdd-packages-sdd-app-src-pages-lineage-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/pages/Lineage.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/pages/Lineage.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Lineage` | projects/agentic-workflow/packages/@sdd/app/src/pages/Lineage.tsx | function | pub | 8 | Lineage() |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useEffect, useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { LineageGraph } from '@sdd/core'

export default function Lineage() {
  const ds = useDataSource()
  const [searchParams, setSearchParams] = useSearchParams()
  const artifactId = searchParams.get('artifact') || ''
  const [graph, setGraph] = useState<LineageGraph | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [inputValue, setInputValue] = useState(artifactId)

  const loadLineage = useCallback(
    (id: string) => {
      if (!id.trim()) return
      setLoading(true)
      setError(null)
      ds.getLineage(id.trim())
        .then(setGraph)
        .catch((e: Error) => setError(e.message))
        .finally(() => setLoading(false))
    },
    [ds],
  )

  useEffect(() => {
    if (artifactId) {
      loadLineage(artifactId)
    }
  }, [artifactId, loadLineage])

  const handleSearch = () => {
    if (inputValue.trim()) {
      setSearchParams({ artifact: inputValue.trim() })
    }
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Lineage</h1>

      {/* Search */}
      <div className="flex gap-2">
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          placeholder="Enter artifact ID (e.g., change ID or spec path)"
          className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <button
          onClick={handleSearch}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors cursor-pointer"
        >
          Trace
        </button>
      </div>

      {/* Graph */}
      {loading && <div className="h-48 bg-gray-200 rounded-lg animate-pulse" />}

      {error && (
        <div className="p-6 text-center text-red-600">
          <p>Failed to load lineage: {error}</p>
        </div>
      )}

      {graph && !loading && (
        <div className="bg-white border rounded-lg p-6">
          {graph.nodes.length === 0 ? (
            <p className="text-sm text-gray-500 text-center">No lineage data found for this artifact.</p>
          ) : (
            <div className="space-y-4">
              <h2 className="text-sm font-medium text-gray-700">
                {graph.nodes.length} node{graph.nodes.length !== 1 ? 's' : ''},{' '}
                {graph.edges.length} edge{graph.edges.length !== 1 ? 's' : ''}
              </h2>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {graph.nodes.map((node) => (
                  <div
                    key={node.id}
                    className={`p-3 border rounded-lg ${
                      node.kind === 'issue' ? 'border-green-200 bg-green-50' :
                      node.kind === 'change' ? 'border-blue-200 bg-blue-50' :
                      node.kind === 'spec' ? 'border-purple-200 bg-purple-50' :
                      'border-gray-200 bg-gray-50'
                    }`}
                  >
                    <div className="text-xs font-medium text-gray-500 uppercase">{node.kind}</div>
                    <div className="text-sm font-medium text-gray-900 mt-1">{node.label}</div>
                    <div className="text-xs text-gray-400 font-mono mt-0.5">{node.id}</div>
                  </div>
                ))}
              </div>
              {graph.edges.length > 0 && (
                <div className="mt-4">
                  <h3 className="text-xs font-medium text-gray-500 uppercase mb-2">Relationships</h3>
                  <div className="space-y-1">
                    {graph.edges.map((edge, i) => (
                      <div key={i} className="text-xs text-gray-600 font-mono">
                        {edge.from} &rarr; {edge.to} <span className="text-gray-400">({edge.relation})</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {!graph && !loading && !error && (
        <div className="p-8 text-center border rounded-lg bg-white">
          <p className="text-sm text-gray-500">
            Enter an artifact ID above to trace its lineage through issues, changes, and specs.
          </p>
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
  - path: projects/agentic-workflow/packages/@sdd/app/src/pages/Lineage.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
