// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/editing/InlineEditSelect.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
