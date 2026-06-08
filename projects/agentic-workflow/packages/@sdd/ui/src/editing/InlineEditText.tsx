// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/editing/InlineEditText.md#source
// CODEGEN-BEGIN
import { useState, useRef, useEffect } from 'react'
import { Pencil, X, Check } from 'lucide-react'

export interface InlineEditTextProps {
  value: string
  onSave: (value: string) => void
  className?: string
}

export default function InlineEditText({
  value,
  onSave,
  className = '',
}: InlineEditTextProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(value)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editing) inputRef.current?.focus()
  }, [editing])

  const save = () => {
    const trimmed = draft.trim()
    if (trimmed && trimmed !== value) onSave(trimmed)
    setEditing(false)
  }

  const cancel = () => {
    setDraft(value)
    setEditing(false)
  }

  if (editing) {
    return (
      <div className="flex items-center gap-2 flex-1 min-w-0">
        <input
          ref={inputRef}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') save(); if (e.key === 'Escape') cancel() }}
          className="flex-1 px-2 py-1 border border-gray-300 rounded text-lg font-bold focus:outline-none focus:ring-2 focus:ring-primary"
        />
        <button onClick={save} className="p-1 text-green-600 hover:bg-green-50 rounded cursor-pointer">
          <Check className="h-4 w-4" />
        </button>
        <button onClick={cancel} className="p-1 text-gray-400 hover:bg-gray-100 rounded cursor-pointer">
          <X className="h-4 w-4" />
        </button>
      </div>
    )
  }

  return (
    <button
      onClick={() => { setDraft(value); setEditing(true) }}
      className={`${className} truncate text-left hover:bg-gray-50 px-1 -mx-1 rounded transition-colors cursor-pointer group inline-flex items-center gap-1`}
    >
      {value}
      <Pencil className="h-3.5 w-3.5 text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
    </button>
  )
}


// CODEGEN-END
