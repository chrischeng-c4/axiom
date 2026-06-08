---
id: projects-sdd-packages-sdd-app-src-pages-techdesigndetail-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/pages/TechDesignDetail.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/pages/TechDesignDetail.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TechDesignDetail` | projects/agentic-workflow/packages/@sdd/app/src/pages/TechDesignDetail.tsx | function | pub | 9 | TechDesignDetail() |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { TechDesign } from '@sdd/core'
import { SpecViewer } from '@sdd/spec-viewer'

export default function TechDesignDetail() {
  const { id } = useParams<{ id: string }>()
  const ds = useDataSource()
  const [design, setDesign] = useState<TechDesign | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) return
    ds.getTechDesign(id)
      .then(setDesign)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [ds, id])

  if (loading) {
    return <div className="h-64 bg-gray-200 rounded-lg animate-pulse" />
  }

  if (error || !design) {
    return (
      <div className="p-6 text-center text-red-600">
        <p>{error || 'Tech design not found'}</p>
        <Link to=".." className="text-sm text-gray-500 hover:text-gray-700 mt-2 inline-block">
          Back to list
        </Link>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4">
        <Link
          to=".."
          className="text-gray-500 hover:text-gray-900 transition-colors"
          aria-label="Back to tech designs"
        >
          &larr;
        </Link>
        <div>
          <h1 className="text-2xl font-bold text-gray-900">{design.title}</h1>
          <p className="text-sm text-gray-500 font-mono">{design.crate} / {design.path}</p>
        </div>
      </div>

      <div className="bg-white border rounded-lg p-6">
        <SpecViewer content={design.content} />
      </div>
    </div>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/app/src/pages/TechDesignDetail.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
