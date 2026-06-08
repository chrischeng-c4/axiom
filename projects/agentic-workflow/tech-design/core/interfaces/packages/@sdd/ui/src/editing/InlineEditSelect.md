---
id: projects-sdd-packages-sdd-ui-src-editing-inlineeditselect-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditSelect.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditSelect.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `InlineEditSelect` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditSelect.tsx | function | pub | 13 | InlineEditSelect({   value,   options,   onSave,   displayValue, }: InlineEditSelectProps) |
| `InlineEditSelectProps` | projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditSelect.tsx | interface | pub | 6 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { useState } from 'react'
import { Pencil } from 'lucide-react'

export interface InlineEditSelectProps {
  value: string
  options: { value: string; label: string }[]
  onSave: (value: string) => void
  displayValue: string
}

export default function InlineEditSelect({
  value,
  options,
  onSave,
  displayValue,
}: InlineEditSelectProps) {
  const [editing, setEditing] = useState(false)

  if (editing) {
    return (
      <select
        autoFocus
        value={value}
        onChange={(e) => { onSave(e.target.value); setEditing(false) }}
        onBlur={() => setEditing(false)}
        className="text-sm px-2 py-1 border border-gray-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-primary cursor-pointer"
      >
        {options.map(o => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    )
  }

  return (
    <button
      onClick={() => setEditing(true)}
      className="text-sm font-medium text-gray-900 hover:bg-gray-50 px-1 -mx-1 rounded transition-colors cursor-pointer group inline-flex items-center gap-1"
    >
      {displayValue}
      <Pencil className="h-3 w-3 text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity" />
    </button>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/editing/InlineEditSelect.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
