---
id: projects-sdd-packages-sdd-ui-src-editing-inlineeditlabels-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `InlineEditLabelsProps` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | interface | pub | 7 |  |
| `[draft, setDraft]` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | constant | pub | 17 |  |
| `[editing, setEditing]` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | constant | pub | 16 |  |
| `inputRef` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | constant | pub | 18 |  |
| `save` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | function | pub | 29 |  |
| `startEdit` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx | function | pub | 24 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState, useRef, useEffect } from 'react'
import { Pencil, X, Check } from 'lucide-react'
import { Badge } from '../primitives/badge'

export interface InlineEditLabelsProps {
  labels: string[]
  onSave: (labels: string[]) => void
}

export default function InlineEditLabels({
  labels,
  onSave,
}: InlineEditLabelsProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editing) inputRef.current?.focus()
  }, [editing])

  const startEdit = () => {
    setDraft(labels.join(', '))
    setEditing(true)
  }

  const save = () => {
    const newLabels = draft.split(',').map(l => l.trim()).filter(Boolean)
    onSave(newLabels)
    setEditing(false)
  }

  if (editing) {
    return (
      <div className="space-y-2">
        <input
          ref={inputRef}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') save(); if (e.key === 'Escape') setEditing(false) }}
          placeholder="label1, label2, ..."
          className="w-full px-2 py-1 border border-gray-300 rounded text-sm focus:outline-none focus:ring-2 focus:ring-primary"
        />
        <div className="flex gap-1 justify-end">
          <button onClick={save} className="p-1 text-green-600 hover:bg-green-50 rounded cursor-pointer">
            <Check className="h-3.5 w-3.5" />
          </button>
          <button onClick={() => setEditing(false)} className="p-1 text-gray-400 hover:bg-gray-100 rounded cursor-pointer">
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    )
  }

  return (
    <button
      onClick={startEdit}
      className="w-full text-left cursor-pointer group"
    >
      {labels.length > 0 ? (
        <div className="flex flex-wrap gap-1">
          {labels.map((label) => (
            <Badge key={label} variant="secondary">{label}</Badge>
          ))}
          <Pencil className="h-3 w-3 text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity self-center" />
        </div>
      ) : (
        <span className="text-sm text-gray-400 italic group-hover:text-gray-500 inline-flex items-center gap-1">
          Add labels
          <Pencil className="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
        </span>
      )}
    </button>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditLabels.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
