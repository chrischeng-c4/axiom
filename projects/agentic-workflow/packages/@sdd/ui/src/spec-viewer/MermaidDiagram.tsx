// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.md#source
// CODEGEN-BEGIN
import { useEffect, useRef, useState } from 'react'

interface MermaidDiagramProps {
  content: string
  className?: string
}

export default function MermaidDiagram({ content, className = '' }: MermaidDiagramProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [error, setError] = useState<string | null>(null)
  const [svg, setSvg] = useState<string>('')

  useEffect(() => {
    let cancelled = false

    async function render() {
      try {
        const mermaid = (await import('mermaid')).default
        mermaid.initialize({
          startOnLoad: false,
          theme: 'default',
          securityLevel: 'strict',
        })

        const id = `mermaid-${Math.random().toString(36).slice(2, 9)}`
        const { svg: rendered } = await mermaid.render(id, content)
        if (!cancelled) {
          setSvg(rendered)
          setError(null)
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : 'Failed to render diagram')
          setSvg('')
        }
      }
    }

    render()
    return () => { cancelled = true }
  }, [content])

  if (error) {
    return (
      <div className={`bg-red-50 border border-red-200 rounded-lg p-4 ${className}`}>
        <p className="text-sm text-red-600 mb-2">Diagram render error</p>
        <pre className="text-xs text-red-500 whitespace-pre-wrap">{error}</pre>
        <details className="mt-2">
          <summary className="text-xs text-gray-500 cursor-pointer">Source</summary>
          <pre className="text-xs text-gray-600 mt-1 whitespace-pre-wrap">{content}</pre>
        </details>
      </div>
    )
  }

  if (!svg) {
    return (
      <div className={`animate-pulse bg-gray-100 rounded-lg h-48 ${className}`} />
    )
  }

  return (
    <div
      ref={containerRef}
      className={`overflow-x-auto ${className}`}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  )
}


// CODEGEN-END
