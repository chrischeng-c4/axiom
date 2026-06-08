---
id: projects-sdd-packages-sdd-ui-src-spec-viewer-mermaiddiagram-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MermaidDiagram` | projects/agentic-workflow/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.tsx | function | pub | 10 | MermaidDiagram({ content, className = '' }: MermaidDiagramProps) |
| `MermaidDiagramProps` | projects/agentic-workflow/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.tsx | interface | pub | 5 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
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

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/spec-viewer/MermaidDiagram.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
