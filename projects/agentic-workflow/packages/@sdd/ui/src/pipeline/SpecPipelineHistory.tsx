// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/pipeline/SpecPipelineHistory.md#source
// CODEGEN-BEGIN
import { useState, useEffect } from 'react'
import { formatDistanceToNow } from 'date-fns'
import { ChevronDown, ChevronRight, Sparkles, Loader2, Check, X, Minus, Circle } from 'lucide-react'
import { MarkdownSpecRenderer } from '../spec-viewer'
import { OpenAPIViewer } from '../spec-viewer'
import type { DAGNode, StageStatus, SpecRun } from '../types'

// ---------------------------------------------------------------------------
// Status icon
// ---------------------------------------------------------------------------

function NodeStatusIcon({ status }: { status: StageStatus }) {
  switch (status) {
    case 'running': return <Loader2 className="h-3.5 w-3.5 text-blue-500 animate-spin flex-shrink-0" />
    case 'done':    return <Check   className="h-3.5 w-3.5 text-green-600 flex-shrink-0" />
    case 'failed':  return <X       className="h-3.5 w-3.5 text-red-500 flex-shrink-0" />
    case 'skipped': return <Minus   className="h-3.5 w-3.5 text-gray-400 flex-shrink-0" />
    default:        return <Circle  className="h-3.5 w-3.5 text-gray-300 flex-shrink-0" />
  }
}

// ---------------------------------------------------------------------------
// Lazy renderers for right panel
// ---------------------------------------------------------------------------

function MermaidRenderer({ content }: { content: string }) {
  const [showSource, setShowSource] = useState(false)
  const [MermaidDiagram, setMermaidDiagram] = useState<React.ComponentType<{ content: string }> | null>(null)

  useEffect(() => {
    import('../spec-viewer/MermaidDiagram').then((mod) => setMermaidDiagram(() => mod.default))
  }, [])

  return (
    <div className="space-y-2">
      <div className="flex justify-end">
        <button onClick={() => setShowSource(!showSource)} className="text-xs text-gray-500 hover:text-gray-700 cursor-pointer">
          {showSource ? 'Show Diagram' : 'Show Source'}
        </button>
      </div>
      {showSource ? (
        <pre className="text-sm text-gray-800 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg overflow-x-auto">{content}</pre>
      ) : MermaidDiagram ? (
        <MermaidDiagram content={content} />
      ) : (
        <div className="animate-pulse bg-gray-100 rounded-lg h-48" />
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Artifact viewer (right panel)
// ---------------------------------------------------------------------------

function ArtifactViewer({
  node,
  fetchArtifact,
}: {
  node: DAGNode
  fetchArtifact: (specId: number) => Promise<{ content: string; format: string } | null>
}) {
  const [content, setContent] = useState<string | null>(null)
  const [format, setFormat] = useState<string>('markdown')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const specId = node.artifact?.spec_id

  useEffect(() => {
    if (!specId) return
    setContent(null)
    setError(null)
    setLoading(true)
    fetchArtifact(specId)
      .then((data) => {
        if (data) {
          setContent(data.content ?? '')
          setFormat(data.format ?? 'markdown')
        } else {
          setError('Artifact not found')
        }
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false))
  }, [specId, fetchArtifact])

  if (loading) return <div className="animate-pulse h-32 bg-gray-100 rounded-lg" />
  if (error) return <p className="text-xs text-red-500">{error}</p>
  if (content === null) return null

  if (format === 'json') {
    return (
      <pre className="text-xs text-gray-800 font-mono whitespace-pre-wrap overflow-x-auto bg-gray-50 p-3 rounded-lg">
        {(() => { try { return JSON.stringify(JSON.parse(content), null, 2) } catch { return content } })()}
      </pre>
    )
  }
  if (format === 'mermaid') return <MermaidRenderer content={content} />
  if (format === 'openapi') return <OpenAPIViewer content={content} />
  return <MarkdownSpecRenderer content={content} />
}

// ---------------------------------------------------------------------------
// Node row
// ---------------------------------------------------------------------------

function NodeRow({
  node,
  isSelected,
  onSelect,
  indent = false,
}: {
  node: DAGNode
  isSelected: boolean
  onSelect: (node: DAGNode) => void
  indent?: boolean
}) {
  const hasArtifact = !!node.artifact

  return (
    <button
      onClick={() => hasArtifact && onSelect(node)}
      disabled={!hasArtifact}
      className={`w-full text-left py-1.5 text-xs flex items-center gap-2 transition-colors border-b border-gray-100 last:border-b-0 ${indent ? 'pl-6 pr-3' : 'px-3'} ${
        isSelected
          ? 'bg-primary/10 text-primary font-medium border-l-2 border-l-primary'
          : hasArtifact
          ? 'hover:bg-gray-100 cursor-pointer text-gray-700'
          : 'text-gray-400 cursor-default'
      }`}
    >
      <NodeStatusIcon status={node.status} />
      <span className="truncate">{node.label}</span>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Run nodes list: grouped by module when multi-module
// ---------------------------------------------------------------------------

/** One module's bucket of nodes */
interface ModuleBucket {
  moduleKey: string
  fixedNodes: DAGNode[]
  featureNodes: DAGNode[]
}

function bucketByModule(nodes: DAGNode[]): ModuleBucket[] {
  const map = new Map<string, ModuleBucket>()
  for (const n of nodes) {
    const mk = n.module_key || '.'
    if (!map.has(mk)) map.set(mk, { moduleKey: mk, fixedNodes: [], featureNodes: [] })
    if (n.id.includes(':feature:')) map.get(mk)!.featureNodes.push(n)
    else                            map.get(mk)!.fixedNodes.push(n)
  }
  // Stable order: '.' first, then alphabetical, then 'architecture' last
  return [...map.values()].sort((a, b) => {
    if (a.moduleKey === '.')           return -1
    if (b.moduleKey === '.')           return 1
    if (a.moduleKey === 'architecture') return 1
    if (b.moduleKey === 'architecture') return -1
    return a.moduleKey.localeCompare(b.moduleKey)
  })
}

function ModuleBucketRow({
  bucket,
  multiModule,
  selectedNodeId,
  onSelect,
}: {
  bucket: ModuleBucket
  multiModule: boolean
  selectedNodeId: string | null
  onSelect: (node: DAGNode) => void
}) {
  const [featuresExpanded, setFeaturesExpanded] = useState(true)
  const { moduleKey, fixedNodes, featureNodes } = bucket
  const doneFeatures = featureNodes.filter((n) => n.status === 'done').length

  const moduleLabel = moduleKey === '.' ? 'root' : moduleKey

  return (
    <div>
      {/* Module header -- only when multiple modules */}
      {multiModule && (
        <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider bg-gray-50 border-b border-gray-100">
          {moduleLabel}
        </div>
      )}

      {fixedNodes.map((node) => (
        <NodeRow
          key={node.id}
          node={node}
          isSelected={selectedNodeId === node.id}
          onSelect={onSelect}
          indent={multiModule}
        />
      ))}

      {featureNodes.length > 0 && (
        <>
          <button
            onClick={() => setFeaturesExpanded(!featuresExpanded)}
            className={`w-full flex items-center gap-1.5 py-1.5 text-xs text-gray-500 hover:bg-gray-100 transition-colors cursor-pointer border-b border-gray-100 ${multiModule ? 'pl-6 pr-3' : 'px-3'}`}
          >
            {featuresExpanded
              ? <ChevronDown  className="h-3 w-3 flex-shrink-0" />
              : <ChevronRight className="h-3 w-3 flex-shrink-0" />
            }
            <span className="font-medium">Features</span>
            <span className="ml-auto text-gray-400">{doneFeatures}/{featureNodes.length}</span>
          </button>
          {featuresExpanded && featureNodes.map((node) => (
            <NodeRow
              key={node.id}
              node={node}
              isSelected={selectedNodeId === node.id}
              onSelect={onSelect}
              indent={multiModule}
            />
          ))}
        </>
      )}
    </div>
  )
}

function RunNodes({
  nodes,
  selectedNodeId,
  onSelect,
}: {
  nodes: DAGNode[]
  selectedNodeId: string | null
  onSelect: (node: DAGNode) => void
}) {
  if (nodes.length === 0) {
    return <p className="text-xs text-gray-400 px-3 py-2 italic">Waiting for pipeline...</p>
  }

  const buckets = bucketByModule(nodes)
  const multiModule = buckets.length > 1

  return (
    <div>
      {buckets.map((bucket) => (
        <ModuleBucketRow
          key={bucket.moduleKey}
          bucket={bucket}
          multiModule={multiModule}
          selectedNodeId={selectedNodeId}
          onSelect={onSelect}
        />
      ))}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Run entry (collapsible header + nodes)
// ---------------------------------------------------------------------------

function RunEntry({
  label,
  isLive,
  nodes,
  selectedNodeId,
  onSelect,
  defaultExpanded,
}: {
  label: string
  isLive: boolean
  nodes: DAGNode[]
  selectedNodeId: string | null
  onSelect: (node: DAGNode) => void
  defaultExpanded: boolean
}) {
  const [expanded, setExpanded] = useState(defaultExpanded)

  return (
    <div className="border-b border-gray-100 last:border-b-0">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-2 hover:bg-gray-100 transition-colors cursor-pointer text-xs"
      >
        {expanded
          ? <ChevronDown  className="h-3.5 w-3.5 text-gray-400 flex-shrink-0" />
          : <ChevronRight className="h-3.5 w-3.5 text-gray-400 flex-shrink-0" />
        }
        {isLive && <Loader2 className="h-3 w-3 text-blue-500 animate-spin flex-shrink-0" />}
        <span className={`font-medium ${isLive ? 'text-blue-600' : 'text-gray-700'}`}>{label}</span>
      </button>
      {expanded && (
        <RunNodes
          nodes={nodes}
          selectedNodeId={selectedNodeId}
          onSelect={onSelect}
        />
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Props + main component
// ---------------------------------------------------------------------------

export interface SpecPipelineHistoryProps {
  liveNodes: DAGNode[]
  completedRuns: SpecRun[]
  isActive: boolean
  onGenerate: () => void
  projectId: string
  fetchArtifact: (specId: number) => Promise<{ content: string; format: string } | null>
}

export default function SpecPipelineHistory({
  liveNodes,
  completedRuns,
  isActive,
  onGenerate,
  fetchArtifact,
}: SpecPipelineHistoryProps) {
  const [selectedNode, setSelectedNode] = useState<DAGNode | null>(null)

  const hasAnyHistory = isActive || completedRuns.length > 0

  const handleSelectNode = (node: DAGNode) => {
    setSelectedNode((prev) => (prev?.id === node.id ? null : node))
  }

  return (
    <div className="space-y-2">
      {/* Generate button */}
      <button
        onClick={onGenerate}
        disabled={isActive}
        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium border border-gray-300 bg-white rounded-md hover:bg-gray-50 transition-colors cursor-pointer disabled:opacity-50"
      >
        {isActive ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Sparkles className="h-3.5 w-3.5" />}
        Generate
      </button>

      {/* Two-panel layout */}
      {!hasAnyHistory ? (
        <p className="text-xs text-gray-400">No recent pipeline run</p>
      ) : (
        <div className="flex border rounded-lg overflow-hidden min-h-[300px]">
          {/* Left: run history */}
          <div className="w-1/3 overflow-y-auto border-r bg-gray-50 flex-shrink-0">
            {isActive && (
              <RunEntry
                label="Running now"
                isLive={true}
                nodes={liveNodes}
                selectedNodeId={selectedNode?.id ?? null}
                onSelect={handleSelectNode}
                defaultExpanded={true}
              />
            )}
            {completedRuns.map((run, i) => (
              <RunEntry
                key={i}
                label={formatDistanceToNow(run.startedAt, { addSuffix: true })}
                isLive={false}
                nodes={run.nodes}
                selectedNodeId={selectedNode?.id ?? null}
                onSelect={handleSelectNode}
                defaultExpanded={i === 0 && !isActive}
              />
            ))}
          </div>

          {/* Right: artifact viewer */}
          <div className="w-2/3 overflow-auto p-4">
            {selectedNode?.artifact ? (
              <ArtifactViewer node={selectedNode} fetchArtifact={fetchArtifact} />
            ) : (
              <p className="text-xs text-gray-400 mt-4 text-center">
                Select a pipeline node to view its artifact
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  )
}


// CODEGEN-END
