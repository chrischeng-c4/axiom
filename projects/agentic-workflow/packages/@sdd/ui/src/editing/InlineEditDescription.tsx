// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/editing/InlineEditDescription.md#source
// CODEGEN-BEGIN
import { useState, useRef, useEffect } from 'react'
import { Pencil } from 'lucide-react'
import { Card, CardContent } from '../primitives/card'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

export interface InlineEditDescriptionProps {
  value: string
  onSave: (value: string) => void
}

export default function InlineEditDescription({
  value,
  onSave,
}: InlineEditDescriptionProps) {
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState(value)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (editing && textareaRef.current) {
      textareaRef.current.focus()
      textareaRef.current.setSelectionRange(textareaRef.current.value.length, textareaRef.current.value.length)
    }
  }, [editing])

  const save = () => {
    if (draft !== value) onSave(draft)
    setEditing(false)
  }

  if (editing) {
    return (
      <Card>
        <CardContent className="pt-6 space-y-3">
          <textarea
            ref={textareaRef}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary text-sm font-mono"
            rows={16}
          />
          <div className="flex justify-end gap-2">
            <button
              onClick={() => { setDraft(value); setEditing(false) }}
              className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm text-gray-700 hover:bg-gray-50 transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              onClick={save}
              className="px-3 py-1.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors cursor-pointer"
            >
              Save
            </button>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card
      className="group cursor-pointer hover:shadow transition-shadow"
      onClick={() => { setDraft(value); setEditing(true) }}
    >
      <CardContent className="pt-6 relative">
        <Pencil className="h-4 w-4 text-gray-300 absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity" />
        {value ? (
          <div className="prose prose-sm max-w-none prose-headings:text-gray-900 prose-p:text-gray-700 prose-a:text-primary prose-code:text-sm prose-code:before:content-none prose-code:after:content-none prose-pre:bg-gray-50 prose-pre:border prose-pre:border-gray-200 prose-pre:text-gray-800 prose-pre:[&_code]:bg-transparent prose-pre:[&_code]:p-0 prose-table:text-sm prose-th:bg-gray-50 prose-img:rounded-lg">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {value}
            </ReactMarkdown>
          </div>
        ) : (
          <p className="text-sm text-gray-400 italic">Click to add a description...</p>
        )}
      </CardContent>
    </Card>
  )
}


// CODEGEN-END
