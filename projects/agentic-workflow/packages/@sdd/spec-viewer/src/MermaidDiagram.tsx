// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/spec-viewer/src/MermaidDiagram.md#source
// CODEGEN-BEGIN
import { useEffect, useRef, useState } from 'react'
import type { MermaidDiagramProps } from './types'

/**
 * Renders a Mermaid diagram as SVG.
 * Lazy-loads the mermaid library on first render.
 * On invalid syntax, shows the error message + source code (REQ-SV-04).
 */
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
      <div className={`spec-viewer-mermaid-error ${className}`} style={{
        background: '#fef2f2', border: '1px solid #fecaca', borderRadius: 8, padding: 16,
      }}>
        <p style={{ fontSize: 14, color: '#dc2626', marginBottom: 8 }}>Diagram render error</p>
        <pre style={{ fontSize: 12, color: '#ef4444', whiteSpace: 'pre-wrap', marginBottom: 8 }}>{error}</pre>
        <details>
          <summary style={{ fontSize: 12, color: '#6b7280', cursor: 'pointer' }}>Source</summary>
          <pre style={{ fontSize: 12, color: '#4b5563', whiteSpace: 'pre-wrap', marginTop: 4 }}>{content}</pre>
        </details>
      </div>
    )
  }

  if (!svg) {
    return (
      <div className={className} style={{
        background: '#f3f4f6', borderRadius: 8, height: 192, animation: 'pulse 2s infinite',
      }} />
    )
  }

  return (
    <div
      ref={containerRef}
      className={`spec-viewer-mermaid ${className}`}
      style={{ overflowX: 'auto' }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  )
}


// CODEGEN-END
