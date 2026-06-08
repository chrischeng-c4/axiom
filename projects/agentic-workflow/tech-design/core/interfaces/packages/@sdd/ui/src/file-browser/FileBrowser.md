---
id: projects-sdd-packages-sdd-ui-src-file-browser-filebrowser-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileBrowserProps` | projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx | interface | pub | 8 |  |
| `breadcrumbs` | projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx | constant | pub | 36 |  |
| `navigateToBreadcrumb` | projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx | function | pub | 42 |  |
| `navigateToPath` | projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx | function | pub | 38 |  |
| `sortedFiles` | projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx | constant | pub | 51 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import FileTreeItem from './FileTreeItem'
import FileViewer from './FileViewer'
import { X, FolderOpen, Loader2, ChevronRight, AlertCircle } from 'lucide-react'
import type { FileEntry, FileContent } from '../types'

export interface FileBrowserProps {
  files: FileEntry[]
  isLoading: boolean
  error?: string | null
  currentPath: string
  onNavigate: (path: string) => void
  fileContent: FileContent | null
  contentLoading: boolean
  contentError?: string | null
  onSelectFile: (path: string) => void
  selectedFile: string | null
  onClose?: () => void
  defaultBranch?: string
}

export default function FileBrowser({
  files,
  isLoading,
  error,
  currentPath,
  onNavigate,
  fileContent,
  contentLoading,
  contentError,
  onSelectFile,
  selectedFile,
  onClose,
}: FileBrowserProps) {
  const breadcrumbs = currentPath ? currentPath.split('/') : []

  const navigateToPath = (path: string) => {
    onNavigate(path)
  }

  const navigateToBreadcrumb = (index: number) => {
    if (index < 0) {
      navigateToPath('')
    } else {
      navigateToPath(breadcrumbs.slice(0, index + 1).join('/'))
    }
  }

  // Sort: folders first, then files alphabetically
  const sortedFiles = [...(files || [])].sort((a, b) => {
    if (a.type !== b.type) return a.type === 'tree' ? -1 : 1
    return a.name.localeCompare(b.name)
  })

  return (
    <div className="border rounded-lg bg-white" data-testid="file-browser">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b bg-gray-50">
        <div className="flex items-center gap-2 text-sm">
          <FolderOpen className="h-4 w-4 text-gray-500" />
          {/* Breadcrumbs */}
          <nav className="flex items-center gap-1" aria-label="File path">
            <button
              onClick={() => navigateToBreadcrumb(-1)}
              className="text-blue-600 hover:text-blue-800 hover:underline cursor-pointer font-medium"
            >
              root
            </button>
            {breadcrumbs.map((segment, i) => (
              <span key={i} className="flex items-center gap-1">
                <ChevronRight className="h-3 w-3 text-gray-400" />
                <button
                  onClick={() => navigateToBreadcrumb(i)}
                  className={`cursor-pointer ${
                    i === breadcrumbs.length - 1
                      ? 'text-gray-900 font-medium'
                      : 'text-blue-600 hover:text-blue-800 hover:underline'
                  }`}
                >
                  {segment}
                </button>
              </span>
            ))}
          </nav>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="p-1 text-gray-400 hover:text-gray-600 rounded cursor-pointer"
            aria-label="Close file browser"
          >
            <X className="h-5 w-5" />
          </button>
        )}
      </div>

      {/* Content */}
      <div className="flex min-h-[400px]">
        {/* File list panel */}
        <div className="w-72 border-r flex-shrink-0 overflow-auto max-h-[600px]">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-5 w-5 animate-spin text-gray-400" />
            </div>
          ) : error ? (
            <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
              <AlertCircle className="h-5 w-5 text-red-400 mb-2" />
              <p className="text-sm text-red-600">Failed to load files</p>
            </div>
          ) : sortedFiles.length === 0 ? (
            <div className="flex items-center justify-center py-12">
              <p className="text-sm text-gray-400">Empty directory</p>
            </div>
          ) : (
            <div className="divide-y">
              {sortedFiles.map((entry) => (
                <FileTreeItem
                  key={entry.path}
                  entry={entry}
                  onNavigate={navigateToPath}
                  onSelect={onSelectFile}
                  isSelected={selectedFile === entry.path}
                />
              ))}
            </div>
          )}
        </div>

        {/* File viewer panel */}
        <div className="flex-1 overflow-auto p-4 max-h-[600px]">
          {!selectedFile ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-400">
              <FolderOpen className="h-12 w-12 mb-3 text-gray-200" />
              <p className="text-sm">Select a file to view its contents</p>
            </div>
          ) : contentLoading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
            </div>
          ) : contentError ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-400">
              <AlertCircle className="h-8 w-8 mb-2 text-red-300" />
              <p className="text-sm text-red-500">Failed to load file content</p>
            </div>
          ) : fileContent ? (
            <FileViewer file={fileContent} />
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-gray-400">
              <AlertCircle className="h-8 w-8 mb-2 text-red-300" />
              <p className="text-sm text-red-500">Failed to load file content</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/file-browser/FileBrowser.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
