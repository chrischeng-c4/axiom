// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/spec-viewer/OpenAPIViewer.md#source
// CODEGEN-BEGIN
import { useState, useMemo } from 'react'
import { ChevronDown, ChevronRight } from 'lucide-react'
import yaml from 'js-yaml'

interface OpenAPIViewerProps {
  content: string
}

const HTTP_METHODS = ['get', 'post', 'put', 'patch', 'delete', 'head', 'options'] as const
type HttpMethod = (typeof HTTP_METHODS)[number]

const METHOD_COLORS: Record<HttpMethod, string> = {
  get:     'bg-blue-100 text-blue-700',
  post:    'bg-green-100 text-green-700',
  put:     'bg-amber-100 text-amber-700',
  patch:   'bg-orange-100 text-orange-700',
  delete:  'bg-red-100 text-red-700',
  head:    'bg-purple-100 text-purple-700',
  options: 'bg-gray-100 text-gray-700',
}

interface Endpoint {
  method: HttpMethod
  path: string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  op: Record<string, any>
}

function schemaToString(schema: unknown): string {
  try { return JSON.stringify(schema, null, 2) } catch { return String(schema) }
}

function EndpointRow({ endpoint }: { endpoint: Endpoint }) {
  const [expanded, setExpanded] = useState(false)
  const { method, path, op } = endpoint

  const parameters: Array<Record<string, unknown>> = op.parameters ?? []
  const requestBody = op.requestBody
  const responses: Record<string, Record<string, unknown>> = op.responses ?? {}

  return (
    <div>
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-2 hover:bg-gray-50 transition-colors text-left cursor-pointer"
      >
        {expanded
          ? <ChevronDown  className="h-3.5 w-3.5 text-gray-400 flex-shrink-0" />
          : <ChevronRight className="h-3.5 w-3.5 text-gray-400 flex-shrink-0" />
        }
        <span className={`text-[11px] font-bold uppercase px-1.5 py-0.5 rounded font-mono flex-shrink-0 ${METHOD_COLORS[method]}`}>
          {method}
        </span>
        <span className="text-xs font-mono text-gray-700">{path}</span>
        {op.summary && (
          <span className="text-xs text-gray-400 ml-2 truncate">{op.summary}</span>
        )}
      </button>

      {expanded && (
        <div className="px-9 pb-3 space-y-3 bg-gray-50 border-t border-gray-100">
          {op.description && (
            <p className="text-xs text-gray-600 pt-2">{op.description}</p>
          )}

          {/* Parameters */}
          {parameters.length > 0 && (
            <div>
              <p className="text-[11px] font-semibold text-gray-500 uppercase tracking-wide mb-1">Parameters</p>
              <table className="w-full text-xs border border-gray-200 rounded">
                <thead className="bg-gray-100">
                  <tr>
                    <th className="text-left px-2 py-1 font-medium text-gray-600">Name</th>
                    <th className="text-left px-2 py-1 font-medium text-gray-600">In</th>
                    <th className="text-left px-2 py-1 font-medium text-gray-600">Required</th>
                    <th className="text-left px-2 py-1 font-medium text-gray-600">Type</th>
                    <th className="text-left px-2 py-1 font-medium text-gray-600">Description</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {parameters.map((p, i) => (
                    <tr key={i}>
                      <td className="px-2 py-1 font-mono text-gray-800">{String(p.name ?? '')}</td>
                      <td className="px-2 py-1 text-gray-600">{String(p.in ?? '')}</td>
                      <td className="px-2 py-1 text-gray-600">{p.required ? 'yes' : 'no'}</td>
                      <td className="px-2 py-1 text-gray-600 font-mono">
                        {String((p.schema as Record<string, unknown> | undefined)?.type ?? p.type ?? '')}
                      </td>
                      <td className="px-2 py-1 text-gray-500">{String(p.description ?? '')}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Request Body */}
          {requestBody && (
            <div>
              <p className="text-[11px] font-semibold text-gray-500 uppercase tracking-wide mb-1">Request Body</p>
              <pre className="text-xs font-mono bg-white border border-gray-200 rounded p-2 overflow-x-auto whitespace-pre-wrap">
                {schemaToString(requestBody)}
              </pre>
            </div>
          )}

          {/* Responses */}
          {Object.keys(responses).length > 0 && (
            <div>
              <p className="text-[11px] font-semibold text-gray-500 uppercase tracking-wide mb-1">Responses</p>
              <div className="space-y-1">
                {Object.entries(responses).map(([status, resp]) => (
                  <div key={status} className="border border-gray-200 rounded overflow-hidden">
                    <div className="flex items-center gap-2 px-2 py-1 bg-gray-100">
                      <span className="text-xs font-mono font-semibold text-gray-700">{status}</span>
                      {Boolean(resp.description) && (
                        <span className="text-xs text-gray-500">{String(resp.description)}</span>
                      )}
                    </div>
                    {Boolean(resp.content) && (
                      <pre className="text-xs font-mono p-2 overflow-x-auto whitespace-pre-wrap bg-white">
                        {schemaToString(resp.content)}
                      </pre>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default function OpenAPIViewer({ content }: OpenAPIViewerProps) {
  const [showSource, setShowSource] = useState(false)

  const { parsed, error } = useMemo(() => {
    try {
      return { parsed: yaml.load(content) as Record<string, unknown>, error: null }
    } catch (e) {
      return { parsed: null, error: `Parse error: ${e}` }
    }
  }, [content])

  const endpoints: Endpoint[] = useMemo(() => {
    if (!parsed) return []
    const paths = parsed.paths as Record<string, Record<string, unknown>> | undefined
    if (!paths) return []
    return Object.entries(paths).flatMap(([path, methods]) =>
      HTTP_METHODS.filter((m) => m in methods).map((m) => ({
        method: m,
        path,
        op: methods[m] as Record<string, unknown>,
      }))
    )
  }, [parsed])

  return (
    <div className="space-y-2">
      <div className="flex justify-end">
        <button
          onClick={() => setShowSource(!showSource)}
          className="text-xs text-gray-500 hover:text-gray-700 cursor-pointer"
        >
          {showSource ? 'Show Docs' : 'Show Source'}
        </button>
      </div>

      {showSource ? (
        <pre className="text-sm text-gray-800 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg overflow-x-auto">
          {content}
        </pre>
      ) : error ? (
        <div className="text-sm text-red-600 bg-red-50 p-4 rounded-lg">{error}</div>
      ) : endpoints.length === 0 ? (
        <div className="text-sm text-gray-400 bg-gray-50 p-4 rounded-lg">No endpoints found.</div>
      ) : (
        <div className="border border-gray-200 rounded-lg divide-y divide-gray-200">
          {endpoints.map((e, i) => (
            <EndpointRow key={`${e.method}-${e.path}-${i}`} endpoint={e} />
          ))}
        </div>
      )}
    </div>
  )
}


// CODEGEN-END
