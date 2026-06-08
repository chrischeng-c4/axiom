```yaml
module: "@cclab/ui/file-browser"
components: [FileBrowser, FileTreeItem, FileViewer]
```

```yaml
# Component: FileBrowser
# Flat directory navigator with breadcrumb (NOT expandable tree)
id: FileBrowser
props:
  projectId: string
  defaultBranch?: string
  onClose?: "() => void"
states:
  loading: "Loader2 spinner centered in file list panel"
  empty: "centered 'Empty directory' text"
  populated: "two-panel layout with file list and content viewer"
  error: "AlertCircle + 'Failed to load files' centered"
  content_loading: "Loader2 spinner in file_content_panel"
  no_file_selected: "'Select a file to view' prompt in file_content_panel"
interactions:
  "click directory": "navigate into directory (replace file list with directory contents)"
  "click file": "set selectedFile, show content in file_content_panel"
  "click breadcrumb": "navigate to ancestor directory"
direction: horizontal
children:
  - id: header_bar
    direction: horizontal
    height: 40
    children:
      - id: folder_icon
        # FolderOpen icon
      - id: breadcrumb_nav
        flex: 1
      - id: close_button
        # conditional: X icon, calls onClose
  - id: file_list
    width: 288
    scroll: vertical
    # sorted: folders first, then files alphabetically; flat FileTreeItem rows
  - id: file_content_panel
    flex: 1
    scroll: both
    # conditional: shows 'Select a file' prompt when no selectedFile; Loader2 when loading; FileViewer when loaded
```

```yaml
# Component: FileTreeItem
# Flat row, NOT recursive
id: FileTreeItem
props:
  entry: "FileEntry"
  onNavigate: "(path: string) => void"
  onSelect: "(path: string) => void"
  isSelected?: boolean
states: {}
  # stateless row component -- no internal state
interactions:
  "click directory": "onNavigate(entry.path)"
  "click file": "onSelect(entry.path)"
direction: horizontal
height: auto
children:
  - id: entry_icon
    width: 16
    # Folder icon (blue) for dirs, File icon (gray) for files
  - id: entry_name
    flex: 1
    # truncated text
  - id: chevron_icon
    width: 16
    # conditional: ChevronRight for directories only
  - id: file_size
    # conditional: formatted size for files only (when entry.size present)
```

```yaml
# Component: FileViewer
# Receives already-fetched content from parent
id: FileViewer
props:
  file: "FileContent"
states:
  image: "centered img tag with base64 data URI (png, jpg, gif, svg, webp, ico)"
  video: "centered video element with controls (mp4, webm, mov)"
  pdf: "iframe with base64 data URI"
  csv_tsv: "parsed table (papaparse) with sticky header"
  excel: "parsed table (xlsx) with sticky header"
  text_content: "SyntaxHighlighter with language detection and line numbers"
  fallback: "FileText icon + filename + size + download link"
interactions: {}
direction: vertical
# no data_source -- parent (FileBrowser) fetches content and passes FileContent prop
```
