// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/pages/TechDesignList.md#source
// CODEGEN-BEGIN
import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { TechDesignSummary } from '@sdd/core'

export default function TechDesignList() {
  const ds = useDataSource()
  const [designs, setDesigns] = useState<TechDesignSummary[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    ds.listTechDesigns()
      .then(setDesigns)
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
        <p>Failed to load tech designs: {error}</p>
      </div>
    )
  }

  // Group by crate
  const byCrate = new Map<string, TechDesignSummary[]>()
  for (const d of designs) {
    const list = byCrate.get(d.crate) || []
    list.push(d)
    byCrate.set(d.crate, list)
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Tech Designs</h1>

      {designs.length === 0 ? (
        <div className="p-8 text-center border rounded-lg bg-white">
          <p className="text-sm text-gray-500">No tech designs found in this project.</p>
        </div>
      ) : (
        Array.from(byCrate.entries()).map(([crate, items]) => (
          <div key={crate} className="space-y-2">
            <h2 className="text-sm font-semibold text-gray-600 uppercase tracking-wide">{crate}</h2>
            <div className="space-y-1">
              {items.map((d) => (
                <Link
                  key={d.id}
                  to={`tech-designs/${encodeURIComponent(d.id)}`}
                  className="block p-4 bg-white border rounded-lg hover:shadow transition-shadow"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-gray-900">{d.title}</span>
                    <span className="text-xs text-gray-400 font-mono">{d.path}</span>
                  </div>
                </Link>
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  )
}


// CODEGEN-END
