// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/pages/TechDesignDetail.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
