// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/ui/src/file-browser/FileTreeItem.md#source
// CODEGEN-BEGIN
import { Folder, File, ChevronRight } from 'lucide-react'
import type { FileEntry } from '../types'

interface FileTreeItemProps {
  entry: FileEntry
  onNavigate: (path: string) => void
  onSelect: (path: string) => void
  isSelected?: boolean
}

export default function FileTreeItem({ entry, onNavigate, onSelect, isSelected }: FileTreeItemProps) {
  const isFolder = entry.type === 'tree'

  const handleClick = () => {
    if (isFolder) {
      onNavigate(entry.path)
    } else {
      onSelect(entry.path)
    }
  }

  return (
    <button
      onClick={handleClick}
      className={`w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-50 transition-colors cursor-pointer ${
        isSelected ? 'bg-blue-50 text-blue-700' : 'text-gray-700'
      }`}
    >
      {isFolder ? (
        <Folder className="h-4 w-4 text-blue-500 flex-shrink-0" />
      ) : (
        <File className="h-4 w-4 text-gray-400 flex-shrink-0" />
      )}
      <span className="truncate flex-1">{entry.name}</span>
      {isFolder && <ChevronRight className="h-4 w-4 text-gray-300 flex-shrink-0" />}
      {!isFolder && entry.size != null && (
        <span className="text-xs text-gray-400 flex-shrink-0">
          {formatSize(entry.size)}
        </span>
      )}
    </button>
  )
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}


// CODEGEN-END
