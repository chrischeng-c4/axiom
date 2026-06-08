---
id: projects-sdd-packages-sdd-spec-viewer-src-specviewer-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Token` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | interface | pub | 32 |  |
| `[MermaidDiagram, setMermaidDiagram]` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 197 |  |
| `[page, setPage]` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 265 |  |
| `[showSource, setShowSource]` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 198 |  |
| `bg` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 285 |  |
| `bgCode` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 288 |  |
| `borderColor` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 289 |  |
| `escapeHtml` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | function | pub | 20 | escapeHtml(text: string) |
| `fg` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 286 |  |
| `fgMuted` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 287 |  |
| `hasOutline` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 269 |  |
| `headings` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 268 |  |
| `isDark` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 272 |  |
| `isPaginated` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 275 |  |
| `pageTokens` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 276 |  |
| `parseMarkdown` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | function | pub | 45 | parseMarkdown(src: string) |
| `parsePipeLine` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | function | pub | 162 | parsePipeLine(line: string) |
| `renderInline` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | function | pub | 175 | renderInline(text: string) |
| `sanitizeHtml` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | function | pub | 11 | sanitizeHtml(text: string) |
| `scrollTo` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 281 |  |
| `tokens` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 267 |  |
| `totalPages` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx | constant | pub | 274 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useMemo, useState, useEffect, useCallback } from 'react'
import type { SpecViewerProps } from './types'
import CodeBlock from './CodeBlock'

// ---------------------------------------------------------------------------
// XSS sanitization (REQ-SV-07)
// ---------------------------------------------------------------------------

function sanitizeHtml(text: string): string {
  return text
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<\/script>/gi, '')
    .replace(/<script[^>]*>/gi, '')
    .replace(/on\w+\s*=\s*"[^"]*"/gi, '')
    .replace(/on\w+\s*=\s*'[^']*'/gi, '')
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

// ---------------------------------------------------------------------------
// Markdown token types
// ---------------------------------------------------------------------------

interface Token {
  type: 'heading' | 'paragraph' | 'code' | 'table' | 'ul' | 'ol' | 'blockquote' | 'hr' | 'blank'
  content: string
  level?: number      // heading level 1-6
  language?: string   // code block language
  rows?: string[][]   // table rows (first row is header)
  items?: string[]    // list items
}

// ---------------------------------------------------------------------------
// Markdown parser
// ---------------------------------------------------------------------------

function parseMarkdown(src: string): Token[] {
  const sanitized = sanitizeHtml(src)
  const lines = sanitized.split('\n')
  const tokens: Token[] = []
  let i = 0

  while (i < lines.length) {
    const line = lines[i]

    // Fenced code block
    const fenceMatch = line.match(/^(`{3,}|~{3,})(\w*)\s*$/)
    if (fenceMatch) {
      const fence = fenceMatch[1]
      const lang = fenceMatch[2] || ''
      const codeLines: string[] = []
      i++
      while (i < lines.length && !lines[i].startsWith(fence)) {
        codeLines.push(lines[i])
        i++
      }
      i++ // skip closing fence
      tokens.push({ type: 'code', content: codeLines.join('\n'), language: lang })
      continue
    }

    // Heading
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/)
    if (headingMatch) {
      tokens.push({ type: 'heading', content: headingMatch[2], level: headingMatch[1].length })
      i++
      continue
    }

    // Horizontal rule
    if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line)) {
      tokens.push({ type: 'hr', content: '' })
      i++
      continue
    }

    // Table (line with pipes)
    if (line.includes('|') && i + 1 < lines.length && /^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/.test(lines[i + 1])) {
      const rows: string[][] = []
      // Header row
      rows.push(parsePipeLine(line))
      i++ // skip header
      i++ // skip separator
      while (i < lines.length && lines[i].includes('|') && lines[i].trim() !== '') {
        rows.push(parsePipeLine(lines[i]))
        i++
      }
      tokens.push({ type: 'table', content: '', rows })
      continue
    }

    // Blockquote
    if (line.startsWith('>')) {
      const quoteLines: string[] = []
      while (i < lines.length && (lines[i].startsWith('>') || (lines[i].trim() !== '' && quoteLines.length > 0 && !lines[i].startsWith('#')))) {
        quoteLines.push(lines[i].replace(/^>\s?/, ''))
        i++
      }
      tokens.push({ type: 'blockquote', content: quoteLines.join('\n') })
      continue
    }

    // Unordered list
    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = []
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*[-*+]\s+/, ''))
        i++
      }
      tokens.push({ type: 'ul', content: '', items })
      continue
    }

    // Ordered list
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const items: string[] = []
      while (i < lines.length && /^\s*\d+[.)]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*\d+[.)]\s+/, ''))
        i++
      }
      tokens.push({ type: 'ol', content: '', items })
      continue
    }

    // Blank line
    if (line.trim() === '') {
      i++
      continue
    }

    // Paragraph (collect consecutive non-blank non-special lines)
    const paraLines: string[] = []
    while (
      i < lines.length &&
      lines[i].trim() !== '' &&
      !lines[i].match(/^#{1,6}\s/) &&
      !lines[i].match(/^(`{3,}|~{3,})/) &&
      !lines[i].match(/^(-{3,}|\*{3,}|_{3,})\s*$/) &&
      !/^\s*[-*+]\s+/.test(lines[i]) &&
      !/^\s*\d+[.)]\s+/.test(lines[i]) &&
      !lines[i].startsWith('>')
    ) {
      paraLines.push(lines[i])
      i++
    }
    if (paraLines.length > 0) {
      tokens.push({ type: 'paragraph', content: paraLines.join(' ') })
    }
  }

  return tokens
}

function parsePipeLine(line: string): string[] {
  return line.split('|').map(c => c.trim()).filter((_, idx, arr) => {
    // Remove empty leading/trailing cells from pipe-delimited lines
    if (idx === 0 && arr[0] === '') return false
    if (idx === arr.length - 1 && arr[arr.length - 1] === '') return false
    return true
  })
}

// ---------------------------------------------------------------------------
// Inline formatting
// ---------------------------------------------------------------------------

function renderInline(text: string): string {
  let result = escapeHtml(text)
  // Bold
  result = result.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  result = result.replace(/__(.+?)__/g, '<strong>$1</strong>')
  // Italic
  result = result.replace(/\*(.+?)\*/g, '<em>$1</em>')
  result = result.replace(/_(.+?)_/g, '<em>$1</em>')
  // Inline code
  result = result.replace(/`([^`]+)`/g, '<code style="font-family:monospace;background:#f3f4f6;padding:1px 4px;border-radius:3px;font-size:0.85em">$1</code>')
  // Links
  result = result.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer" style="color:#2563eb;text-decoration:none">$1</a>')
  // Strikethrough
  result = result.replace(/~~(.+?)~~/g, '<del>$1</del>')
  return result
}

// ---------------------------------------------------------------------------
// Lazy Mermaid wrapper
// ---------------------------------------------------------------------------

function LazyMermaid({ content }: { content: string }) {
  const [MermaidDiagram, setMermaidDiagram] = useState<React.ComponentType<{ content: string; className?: string }> | null>(null)
  const [showSource, setShowSource] = useState(false)

  useEffect(() => {
    import('./MermaidDiagram').then((mod) => setMermaidDiagram(() => mod.default))
  }, [])

  return (
    <div style={{ marginBottom: 12 }}>
      <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: 4 }}>
        <button
          onClick={() => setShowSource(!showSource)}
          style={{ background: 'none', border: 'none', cursor: 'pointer', fontSize: 12, color: '#6b7280' }}
        >
          {showSource ? 'Show Diagram' : 'Show Source'}
        </button>
      </div>
      {showSource ? (
        <CodeBlock code={content} language="mermaid" />
      ) : MermaidDiagram ? (
        <MermaidDiagram content={content} />
      ) : (
        <div style={{ background: '#f3f4f6', borderRadius: 8, height: 192 }} />
      )}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Heading outline
// ---------------------------------------------------------------------------

interface HeadingInfo {
  level: number
  text: string
  id: string
}

function extractHeadings(tokens: Token[]): HeadingInfo[] {
  return tokens
    .filter((t): t is Token & { level: number } => t.type === 'heading' && t.level !== undefined)
    .map((t) => ({
      level: t.level!,
      text: t.content,
      id: `sv-${t.content.toLowerCase().replace(/[^\w]+/g, '-').replace(/^-|-$/g, '')}`,
    }))
}

// ---------------------------------------------------------------------------
// Pagination for large specs (REQ-SV-08)
// ---------------------------------------------------------------------------

const TOKENS_PER_PAGE = 200

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

/**
 * SpecViewer renders Markdown content with:
 * - Mermaid diagrams as SVG (REQ-SV-03)
 * - Syntax-highlighted code blocks with copy button (REQ-SV-05)
 * - Tables, lists, headings, bold/italic, links (REQ-SV-02)
 * - XSS prevention (REQ-SV-07)
 * - Dark mode support (REQ-SV-06)
 * - Pagination for large specs (REQ-SV-08)
 */
export default function SpecViewer({ content, className = '', darkMode }: SpecViewerProps) {
  const [page, setPage] = useState(0)

  const tokens = useMemo(() => parseMarkdown(content), [content])
  const headings = useMemo(() => extractHeadings(tokens), [tokens])
  const hasOutline = headings.length >= 2

  // Detect dark mode preference
  const isDark = darkMode ?? (typeof window !== 'undefined' && window.matchMedia?.('(prefers-color-scheme: dark)').matches)

  const totalPages = Math.ceil(tokens.length / TOKENS_PER_PAGE)
  const isPaginated = totalPages > 1
  const pageTokens = isPaginated ? tokens.slice(page * TOKENS_PER_PAGE, (page + 1) * TOKENS_PER_PAGE) : tokens

  // Reset page when content changes
  useEffect(() => { setPage(0) }, [content])

  const scrollTo = useCallback((id: string) => {
    document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }, [])

  const bg = isDark ? '#1f2937' : '#ffffff'
  const fg = isDark ? '#e5e7eb' : '#1f2937'
  const fgMuted = isDark ? '#9ca3af' : '#6b7280'
  const bgCode = isDark ? '#111827' : '#f9fafb'
  const borderColor = isDark ? '#374151' : '#e5e7eb'

  return (
    <div
      className={`spec-viewer ${isDark ? 'spec-viewer-dark' : ''} ${className}`}
      style={{
        display: 'flex',
        gap: 20,
        minHeight: 0,
        color: fg,
        background: bg,
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        fontSize: 14,
        lineHeight: 1.6,
      }}
    >
      {/* Main content */}
      <div style={{ flex: 1, minWidth: 0 }}>
        {pageTokens.map((token, idx) => (
          <TokenRenderer key={idx} token={token} isDark={isDark} bgCode={bgCode} fgMuted={fgMuted} borderColor={borderColor} />
        ))}

        {/* Pagination controls */}
        {isPaginated && (
          <div style={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            gap: 12,
            padding: '16px 0',
            borderTop: `1px solid ${borderColor}`,
            marginTop: 16,
          }}>
            <button
              onClick={() => setPage(p => Math.max(0, p - 1))}
              disabled={page === 0}
              style={{
                padding: '4px 12px', borderRadius: 4, border: `1px solid ${borderColor}`,
                background: page === 0 ? borderColor : 'transparent',
                cursor: page === 0 ? 'default' : 'pointer', fontSize: 13, color: fg,
              }}
            >
              Previous
            </button>
            <span style={{ fontSize: 13, color: fgMuted }}>
              Page {page + 1} of {totalPages}
            </span>
            <button
              onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              style={{
                padding: '4px 12px', borderRadius: 4, border: `1px solid ${borderColor}`,
                background: page >= totalPages - 1 ? borderColor : 'transparent',
                cursor: page >= totalPages - 1 ? 'default' : 'pointer', fontSize: 13, color: fg,
              }}
            >
              Next
            </button>
          </div>
        )}
      </div>

      {/* Outline column */}
      {hasOutline && (
        <div style={{ width: 160, flexShrink: 0 }}>
          <div style={{ position: 'sticky', top: 0, paddingTop: 4 }}>
            <p style={{ fontSize: 10, fontWeight: 600, color: fgMuted, textTransform: 'uppercase', letterSpacing: 1, marginBottom: 8, paddingLeft: 4 }}>
              Outline
            </p>
            <nav>
              {headings.map((h, i) => (
                <button
                  key={i}
                  onClick={() => scrollTo(h.id)}
                  style={{
                    display: 'block',
                    width: '100%',
                    textAlign: 'left',
                    fontSize: 11,
                    color: fgMuted,
                    background: 'none',
                    border: 'none',
                    cursor: 'pointer',
                    padding: '2px 4px',
                    paddingLeft: (h.level - 1) * 12 + 4,
                    borderRadius: 3,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    lineHeight: 1.4,
                  }}
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

// ---------------------------------------------------------------------------
// Token renderer
// ---------------------------------------------------------------------------

function TokenRenderer({ token, isDark, bgCode, fgMuted, borderColor }: {
  token: Token
  isDark: boolean
  bgCode: string
  fgMuted: string
  borderColor: string
}) {
  switch (token.type) {
    case 'heading': {
      const sizes: Record<number, { fontSize: number; fontWeight: number; margin: string }> = {
        1: { fontSize: 22, fontWeight: 700, margin: '24px 0 8px' },
        2: { fontSize: 18, fontWeight: 600, margin: '20px 0 8px' },
        3: { fontSize: 15, fontWeight: 600, margin: '16px 0 4px' },
        4: { fontSize: 13, fontWeight: 600, margin: '12px 0 4px' },
        5: { fontSize: 12, fontWeight: 600, margin: '8px 0 4px' },
        6: { fontSize: 11, fontWeight: 600, margin: '8px 0 4px' },
      }
      const level = token.level ?? 1
      const style = sizes[level] ?? sizes[6]
      const Tag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6'
      const id = `sv-${token.content.toLowerCase().replace(/[^\w]+/g, '-').replace(/^-|-$/g, '')}`
      return <Tag id={id} style={{ ...style, margin: style.margin }}>{token.content}</Tag>
    }

    case 'paragraph':
      return <p style={{ marginBottom: 12 }} dangerouslySetInnerHTML={{ __html: renderInline(token.content) }} />

    case 'code': {
      if (token.language === 'mermaid') {
        return <LazyMermaid content={token.content} />
      }
      return <CodeBlock code={token.content} language={token.language} />
    }

    case 'table': {
      if (!token.rows || token.rows.length === 0) return null
      const [header, ...body] = token.rows
      return (
        <div style={{ overflowX: 'auto', marginBottom: 12 }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr>
                {header.map((cell, ci) => (
                  <th key={ci} style={{
                    textAlign: 'left', padding: '6px 12px', fontWeight: 600,
                    borderBottom: `2px solid ${borderColor}`, background: bgCode,
                    color: fgMuted,
                  }} dangerouslySetInnerHTML={{ __html: renderInline(cell) }} />
                ))}
              </tr>
            </thead>
            <tbody>
              {body.map((row, ri) => (
                <tr key={ri}>
                  {row.map((cell, ci) => (
                    <td key={ci} style={{
                      padding: '6px 12px', borderBottom: `1px solid ${borderColor}`,
                    }} dangerouslySetInnerHTML={{ __html: renderInline(cell) }} />
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )
    }

    case 'ul':
      return (
        <ul style={{ paddingLeft: 20, marginBottom: 12 }}>
          {token.items?.map((item, i) => (
            <li key={i} style={{ marginBottom: 2 }} dangerouslySetInnerHTML={{ __html: renderInline(item) }} />
          ))}
        </ul>
      )

    case 'ol':
      return (
        <ol style={{ paddingLeft: 20, marginBottom: 12 }}>
          {token.items?.map((item, i) => (
            <li key={i} style={{ marginBottom: 2 }} dangerouslySetInnerHTML={{ __html: renderInline(item) }} />
          ))}
        </ol>
      )

    case 'blockquote':
      return (
        <blockquote style={{
          borderLeft: `4px solid ${isDark ? '#4b5563' : '#d1d5db'}`,
          paddingLeft: 16, margin: '12px 0',
          color: fgMuted, fontStyle: 'italic',
        }} dangerouslySetInnerHTML={{ __html: renderInline(token.content) }} />
      )

    case 'hr':
      return <hr style={{ border: 'none', borderTop: `1px solid ${borderColor}`, margin: '16px 0' }} />

    default:
      return null
  }
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/spec-viewer/src/SpecViewer.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
