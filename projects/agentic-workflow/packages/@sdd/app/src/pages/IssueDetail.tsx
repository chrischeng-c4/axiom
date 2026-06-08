// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/pages/IssueDetail.md#source
// CODEGEN-BEGIN
import { useState, useEffect } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useDataSource } from '@sdd/core'
import type { Issue } from '@sdd/core'

export default function IssueDetail() {
  const { id } = useParams<{ id: string }>()
  const ds = useDataSource()
  const [issue, setIssue] = useState<Issue | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) return
    ds.getIssue(id)
      .then(setIssue)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [ds, id])

  if (loading) {
    return <div className="h-64 bg-gray-200 rounded-lg animate-pulse" />
  }

  if (error || !issue) {
    return (
      <div className="p-6 text-center text-red-600">
        <p>{error || 'Issue not found'}</p>
        <Link to=".." className="text-sm text-gray-500 hover:text-gray-700 mt-2 inline-block">
          Back to list
        </Link>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Link
          to=".."
          className="text-gray-500 hover:text-gray-900 transition-colors"
          aria-label="Back to issues"
        >
          &larr;
        </Link>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-gray-500 font-mono text-lg">#{issue.issue_number}</span>
            <h1 className="text-2xl font-bold text-gray-900">{issue.title}</h1>
          </div>
        </div>
        <span className={`text-xs font-medium px-3 py-1 rounded-full ${
          issue.status === 'open' ? 'bg-green-100 text-green-800' :
          issue.status === 'in_progress' ? 'bg-yellow-100 text-yellow-800' :
          'bg-gray-100 text-gray-700'
        }`}>
          {issue.status === 'in_progress' ? 'in progress' : issue.status}
        </span>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main content */}
        <div className="lg:col-span-2">
          <div className="bg-white border rounded-lg p-6">
            <h2 className="text-sm font-medium text-gray-700 mb-3">Description</h2>
            {issue.description ? (
              <div className="prose prose-sm max-w-none text-gray-700 whitespace-pre-wrap">
                {issue.description}
              </div>
            ) : (
              <p className="text-sm text-gray-400 italic">No description provided.</p>
            )}
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          <div className="bg-white border rounded-lg p-4 space-y-3">
            <h3 className="text-sm font-medium text-gray-700">Details</h3>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500">Priority</span>
              <span className="text-sm text-gray-900">
                {issue.priority ? issue.priority.charAt(0).toUpperCase() + issue.priority.slice(1) : 'None'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500">Created</span>
              <span className="text-sm text-gray-900">
                {new Date(issue.created_at).toLocaleDateString()}
              </span>
            </div>

            {issue.closed_at && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-500">Closed</span>
                <span className="text-sm text-gray-900">
                  {new Date(issue.closed_at).toLocaleDateString()}
                </span>
              </div>
            )}

            {issue.labels.length > 0 && (
              <div>
                <span className="text-sm text-gray-500 block mb-2">Labels</span>
                <div className="flex flex-wrap gap-1">
                  {issue.labels.map((label) => (
                    <span key={label} className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded">
                      {label}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}


// CODEGEN-END
