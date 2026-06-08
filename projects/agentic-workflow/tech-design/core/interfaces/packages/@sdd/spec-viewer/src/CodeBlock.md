---
id: projects-sdd-packages-sdd-spec-viewer-src-codeblock-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/spec-viewer/src/CodeBlock.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/spec-viewer/src/CodeBlock.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodeBlock` | projects/agentic-workflow/packages/@sdd/spec-viewer/src/CodeBlock.tsx | function | pub | 10 | CodeBlock({ code, language, className = '' }: CodeBlockProps) |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useCallback } from 'react'
import type { CodeBlockProps } from './types'

/**
 * Syntax-highlighted code block with copy-to-clipboard button (REQ-SV-05).
 * Uses CSS language class for optional external highlighting integration.
 */
export default function CodeBlock({ code, language, className = '' }: CodeBlockProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      // Fallback for insecure context
      const textarea = document.createElement('textarea')
      textarea.value = code
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }, [code])

  return (
    <div className={`spec-viewer-codeblock ${className}`} style={{ position: 'relative', marginBottom: 12 }}>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '4px 12px',
        background: '#e5e7eb',
        borderRadius: '8px 8px 0 0',
        fontSize: 11,
        color: '#6b7280',
      }}>
        <span>{language || 'text'}</span>
        <button
          onClick={handleCopy}
          style={{
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            fontSize: 11,
            color: copied ? '#16a34a' : '#6b7280',
            padding: '2px 8px',
            borderRadius: 4,
            transition: 'color 0.2s',
          }}
          aria-label={copied ? 'Copied' : 'Copy code'}
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      <pre style={{
        margin: 0,
        padding: 16,
        background: '#f9fafb',
        borderRadius: '0 0 8px 8px',
        overflowX: 'auto',
        fontSize: 13,
        lineHeight: 1.6,
        fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',
        color: '#1f2937',
      }}>
        <code className={language ? `language-${language}` : undefined}>
          {code}
        </code>
      </pre>
    </div>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/spec-viewer/src/CodeBlock.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
