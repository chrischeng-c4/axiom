// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/spec-viewer/MarkdownSpecRenderer.md#source
// CODEGEN-BEGIN
/**
 * MarkdownSpecRenderer - renders spec markdown with embedded code blocks.
 * Handles mermaid diagrams, OpenAPI specs, and plain code blocks.
 *
 * Layout: two-column when outline exists -- content left, outline TOC right (sticky).
 * Anchor strategy: every heading gets an id prefixed with a per-instance React useId()
 * scope, so multiple MarkdownSpecRenderers on the same page never collide.
 * Outline navigation uses scrollIntoView -- never touches window.location.hash.
 */
import { useId, useMemo, useState, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import OpenAPIViewer from './OpenAPIViewer'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

interface Heading { level: number; text: string }

function parseHeadings(content: string): Heading[] {
  return content
    .split('\n')
    .map((line) => {
      const m = line.match(/^(#{1,4})\s+(.+)$/)
      return m ? { level: m[1].length, text: m[2].trim() } : null
    })
    .filter((h): h is Heading => h !== null)
}

function makeId(scope: string, text: string): string {
  return `${scope}-${text.toLowerCase().replace(/[^\w]+/g, '-').replace(/^-|-$/g, '')}`
}

// ---------------------------------------------------------------------------
// Lazy Mermaid renderer
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
          {showSource ? 'Show diagram' : 'Show source'}
        </button>
      </div>
      {showSource ? (
        <pre className="text-sm text-gray-800 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg overflow-x-auto">{content}</pre>
      ) : MermaidDiagram ? (
        <MermaidDiagram content={content} className="bg-gray-50 rounded-lg p-4" />
      ) : (
        <div className="animate-pulse bg-gray-100 rounded-lg h-32" />
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export default function MarkdownSpecRenderer({ content }: { content: string }) {
  const rawId = useId()
  // useId() returns ":r0:" style -- strip non-alphanumeric for valid HTML id prefix
  const scope = rawId.replace(/[^a-z0-9]/gi, 'x')

  const headings = useMemo(() => parseHeadings(content), [content])
  const hasOutline = headings.length >= 2

  const indentClass: Record<number, string> = { 1: '', 2: 'pl-3', 3: 'pl-6', 4: 'pl-9' }

  const scrollTo = (text: string) => {
    document.getElementById(makeId(scope, text))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }

  const makeHeading = (level: 1 | 2 | 3 | 4) => {
    const Tag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4'
    const sizeClass: Record<number, string> = {
      1: 'text-lg font-bold mt-6 mb-2 text-gray-900',
      2: 'text-base font-semibold mt-5 mb-2 text-gray-800',
      3: 'text-sm font-semibold mt-4 mb-1 text-gray-800',
      4: 'text-xs font-semibold mt-3 mb-1 text-gray-700 uppercase tracking-wide',
    }
    return ({ children }: { children?: React.ReactNode }) => {
      const text = String(children ?? '')
      return <Tag id={makeId(scope, text)} className={sizeClass[level]}>{children}</Tag>
    }
  }

  return (
    <div className="flex gap-5 min-h-0">
      {/* Main markdown content */}
      <div className="flex-1 min-w-0">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={{
            h1: makeHeading(1),
            h2: makeHeading(2),
            h3: makeHeading(3),
            h4: makeHeading(4),
            p: ({ children }) => <p className="text-sm text-gray-700 mb-3 leading-relaxed">{children}</p>,
            ul: ({ children }) => <ul className="text-sm text-gray-700 list-disc pl-5 mb-3 space-y-1">{children}</ul>,
            ol: ({ children }) => <ol className="text-sm text-gray-700 list-decimal pl-5 mb-3 space-y-1">{children}</ol>,
            li: ({ children }) => <li className="leading-relaxed">{children}</li>,
            a: ({ href, children }) => <a href={href} className="text-blue-600 hover:underline" target="_blank" rel="noopener noreferrer">{children}</a>,
            strong: ({ children }) => <strong className="font-semibold text-gray-900">{children}</strong>,
            blockquote: ({ children }) => (
              <blockquote className="border-l-4 border-gray-300 pl-4 text-sm text-gray-500 italic my-3">{children}</blockquote>
            ),
            table: ({ children }) => (
              <div className="overflow-x-auto mb-3">
                <table className="text-xs border border-gray-200 rounded w-full">{children}</table>
              </div>
            ),
            thead: ({ children }) => <thead className="bg-gray-100">{children}</thead>,
            th: ({ children }) => <th className="text-left px-3 py-1.5 font-medium text-gray-600 border-b border-gray-200">{children}</th>,
            td: ({ children }) => <td className="px-3 py-1.5 border-b border-gray-100 text-gray-700">{children}</td>,
            hr: () => <hr className="my-4 border-gray-200" />,
            code: ({ className, children, ...props }) => {
              const inline = !className && typeof children === 'string' && !String(children).includes('\n')
              if (inline) {
                return <code className="text-xs font-mono bg-gray-100 px-1 py-0.5 rounded text-gray-800" {...props}>{children}</code>
              }
              const lang = /language-(\w+)/.exec(className ?? '')?.[1]?.toLowerCase() ?? ''
              const src = String(children).trim()
              if (lang === 'mermaid') return <MermaidRenderer content={src} />
              if (lang === 'openapi' || (lang === 'yaml' && src.includes('openapi:'))) return <OpenAPIViewer content={src} />
              return (
                <pre className="text-sm text-gray-800 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg overflow-x-auto mb-3">
                  <code>{src}</code>
                </pre>
              )
            },
            pre: ({ children }) => <>{children}</>,
          }}
        >
          {content}
        </ReactMarkdown>
      </div>

      {/* Outline column -- sticky TOC */}
      {hasOutline && (
        <div className="w-40 flex-shrink-0 hidden md:block">
          <div className="sticky top-0 pt-1">
            <p className="text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">Outline</p>
            <nav className="space-y-0.5">
              {headings.map((h, i) => (
                <button
                  key={i}
                  onClick={() => scrollTo(h.text)}
                  className={`w-full text-left text-[11px] text-gray-500 hover:text-gray-900 hover:bg-gray-100 rounded px-1 py-0.5 transition-colors cursor-pointer truncate leading-snug ${indentClass[h.level] ?? 'pl-9'}`}
                >
                  {h.text}
                </button>
              ))}
            </nav>
          </div>
        </div>
      )}
    </div>
  )
}


// CODEGEN-END
