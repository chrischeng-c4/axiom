// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/spec-viewer/SpecFileBrowser.md#source
// CODEGEN-BEGIN
import { useState, useEffect, useMemo } from 'react'
import { FileText, Folder, FolderOpen, ChevronRight, ChevronDown, Loader2 } from 'lucide-react'
import type { ProjectSpec } from '../types'
import MarkdownSpecRenderer from './MarkdownSpecRenderer'
import OpenAPIViewer from './OpenAPIViewer'

// ---------------------------------------------------------------------------
// Tree types + builder
// ---------------------------------------------------------------------------

interface TreeDir  { type: 'dir';  name: string; path: string; children: TreeNode[] }
interface TreeFile { type: 'file'; name: string; spec: ProjectSpec }
type TreeNode = TreeDir | TreeFile

function buildTree(specs: ProjectSpec[]): TreeNode[] {
  // intermediate map structure
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const root: Record<string, any> = {}

  for (const spec of specs) {
    const parts = spec.path.split('/')
    let cur = root
    for (let i = 0; i < parts.length - 1; i++) {
      if (!cur[parts[i]]) cur[parts[i]] = { _dir: true, _ch: {} }
      cur = cur[parts[i]]._ch
    }
    cur[parts[parts.length - 1]] = { _dir: false, _spec: spec }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function toNodes(obj: Record<string, any>, parentPath = ''): TreeNode[] {
    return Object.entries(obj)
      .sort(([an, av], [bn, bv]) => {
        if (av._dir && !bv._dir) return -1
        if (!av._dir && bv._dir) return 1
        return an.localeCompare(bn)
      })
      .map(([name, val]) => {
        const path = parentPath ? `${parentPath}/${name}` : name
        return val._dir
          ? { type: 'dir' as const, name, path, children: toNodes(val._ch, path) }
          : { type: 'file' as const, name, spec: val._spec as ProjectSpec }
      })
  }

  return toNodes(root)
}

function collectDirPaths(nodes: TreeNode[]): string[] {
  const paths: string[] = []
  for (const n of nodes) {
    if (n.type === 'dir') { paths.push(n.path); paths.push(...collectDirPaths(n.children)) }
  }
  return paths
}

// ---------------------------------------------------------------------------
// Tree row (recursive)
// ---------------------------------------------------------------------------

function TreeRow({ node, depth, selectedId, onSelect, expanded, onToggle }: {
  node: TreeNode
  depth: number
  selectedId: number | null
  onSelect: (id: number) => void
  expanded: Set<string>
  onToggle: (path: string) => void
}) {
  const baseIndent = depth * 12

  if (node.type === 'dir') {
    const isOpen = expanded.has(node.path)
    return (
      <>
        <button
          onClick={() => onToggle(node.path)}
          className="w-full text-left flex items-center gap-1 py-0.5 pr-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700 transition-colors cursor-pointer"
          style={{ paddingLeft: baseIndent + 4 }}
        >
          {isOpen
            ? <ChevronDown className="h-3 w-3 flex-shrink-0 text-gray-400" />
            : <ChevronRight className="h-3 w-3 flex-shrink-0 text-gray-400" />}
          {isOpen
            ? <FolderOpen className="h-3 w-3 flex-shrink-0 text-blue-400" />
            : <Folder className="h-3 w-3 flex-shrink-0 text-blue-400" />}
          <span className="text-[10px] font-mono font-semibold truncate text-gray-600">{node.name}</span>
        </button>
        {isOpen && node.children.map((child, i) => (
          <TreeRow key={i} node={child} depth={depth + 1} selectedId={selectedId} onSelect={onSelect} expanded={expanded} onToggle={onToggle} />
        ))}
      </>
    )
  }

  const isSelected = selectedId === node.spec.id
  return (
    <button
      onClick={() => onSelect(node.spec.id)}
      title={node.spec.path}
      className={`w-full text-left flex items-center gap-1.5 py-0.5 pr-2 transition-colors cursor-pointer ${
        isSelected
          ? 'bg-primary/10 text-primary font-medium border-l-2 border-l-primary'
          : 'text-gray-600 hover:bg-gray-100 hover:text-gray-700'
      }`}
      style={{ paddingLeft: baseIndent + 16 }}
    >
      <FileText className={`h-3 w-3 flex-shrink-0 ${isSelected ? 'text-primary' : 'text-gray-400'}`} />
      <span className="text-[10px] font-mono truncate">{node.name}</span>
    </button>
  )
}

// ---------------------------------------------------------------------------
// Content renderers
// ---------------------------------------------------------------------------

function MermaidRenderer({ content }: { content: string }) {
  const [showSource, setShowSource] = useState(false)
  const [MermaidDiagram, setMermaidDiagram] = useState<React.ComponentType<{ content: string; className?: string }> | null>(null)

  useEffect(() => {
    import('./MermaidDiagram').then((mod) => setMermaidDiagram(() => mod.default))
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

function SpecContentRenderer({ spec }: { spec: ProjectSpec }) {
  if (spec.format === 'mermaid') return <MermaidRenderer content={spec.content} />
  if (spec.format === 'openapi') return <OpenAPIViewer content={spec.content} />
  if (spec.format === 'markdown') return <MarkdownSpecRenderer content={spec.content} />
  return <pre className="text-sm text-gray-800 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg overflow-x-auto">{spec.content}</pre>
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

interface SpecFileBrowserProps {
  specs: ProjectSpec[]
  emptyMessage?: string
}

export default function SpecFileBrowser({ specs, emptyMessage = 'No specs generated yet for this module.' }: SpecFileBrowserProps) {
  const [selectedId, setSelectedId] = useState<number | null>(null)
  const [sidebarWidth, setSidebarWidth] = useState(208) // default w-52 = 208px
  const [isResizing, setIsResizing] = useState(false)

  // Drag-to-resize sidebar
  useEffect(() => {
    if (!isResizing) return
    const onMouseMove = (e: MouseEvent) => {
      const container = document.getElementById('spec-file-browser')
      if (!container) return
      const rect = container.getBoundingClientRect()
      const newWidth = Math.max(120, Math.min(e.clientX - rect.left, 600))
      setSidebarWidth(newWidth)
    }
    const onMouseUp = () => setIsResizing(false)
    document.addEventListener('mousemove', onMouseMove)
    document.addEventListener('mouseup', onMouseUp)
    return () => {
      document.removeEventListener('mousemove', onMouseMove)
      document.removeEventListener('mouseup', onMouseUp)
    }
  }, [isResizing])

  const tree = useMemo(() => buildTree(specs), [specs])

  // All dirs expanded by default; expand any new dirs that appear
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(collectDirPaths(tree)))
  useEffect(() => {
    const newPaths = collectDirPaths(tree)
    setExpanded((prev) => {
      const next = new Set(prev)
      newPaths.forEach((p) => next.add(p))
      return next
    })
  }, [tree])

  // Auto-select first spec
  useEffect(() => {
    if (specs.length > 0 && (selectedId === null || !specs.find((s) => s.id === selectedId))) {
      setSelectedId(specs[0].id)
    }
    if (specs.length === 0) setSelectedId(null)
  }, [specs]) // eslint-disable-line react-hooks/exhaustive-deps

  const toggleDir = (path: string) =>
    setExpanded((prev) => {
      const next = new Set(prev)
      next.has(path) ? next.delete(path) : next.add(path)
      return next
    })

  if (specs.length === 0) {
    return <p className="text-xs text-gray-400">{emptyMessage}</p>
  }

  const selectedSpec = specs.find((s) => s.id === selectedId)

  return (
    <div id="spec-file-browser" className="flex border rounded-lg overflow-hidden min-h-[300px]" style={{ userSelect: isResizing ? 'none' : undefined }}>
      {/* Left: file tree (resizable) */}
      <div className="overflow-y-auto border-r bg-gray-50 flex-shrink-0 py-1" style={{ width: sidebarWidth }}>
        {tree.map((node, i) => (
          <TreeRow
            key={i}
            node={node}
            depth={0}
            selectedId={selectedId}
            onSelect={setSelectedId}
            expanded={expanded}
            onToggle={toggleDir}
          />
        ))}
      </div>

      {/* Resize handle */}
      <div
        className="w-1 hover:bg-primary/30 cursor-col-resize flex-shrink-0 transition-colors"
        style={{ background: isResizing ? 'var(--color-primary, #3b82f6)' : undefined, opacity: isResizing ? 0.3 : undefined }}
        onMouseDown={() => setIsResizing(true)}
      />

      {/* Right: content panel */}
      <div className="flex-1 overflow-auto p-4 min-w-0">
        {selectedSpec ? (
          <SpecContentRenderer spec={selectedSpec} />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-400">
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
            <span className="text-sm">Loading...</span>
          </div>
        )}
      </div>
    </div>
  )
}


// CODEGEN-END
