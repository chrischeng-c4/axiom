```yaml
module: "@cclab/ui/spec-viewer"
components: [SpecFileBrowser, MarkdownSpecRenderer, MermaidDiagram, OpenAPIViewer]
```

```yaml
# Component: SpecFileBrowser
id: SpecFileBrowser
props:
  specs: ProjectSpec[]
  emptyMessage?: string
states:
  empty: "show emptyMessage centered"
  populated: "two-panel layout"
  dir_expanded: "directory node shows children; ChevronDown + FolderOpen icon"
  dir_collapsed: "directory node hides children; ChevronRight + Folder icon"
interactions:
  "click file": "set selectedId to spec.id (number)"
  "click directory": "toggle directory expand/collapse"
  "mount": "auto-select first file if specs non-empty"
direction: horizontal
min_height: 300
children:
  - id: file_list
    flex: 1
    scroll: vertical
  - id: content_viewer
    flex: 2
    scroll: vertical
    # content_viewer dispatches to: MermaidDiagram | OpenAPIViewer | MarkdownSpecRenderer | pre (raw)
```

```yaml
# Component: MarkdownSpecRenderer
# Used in content_viewer inside spec_file_browser
id: MarkdownSpecRenderer
props:
  content: string
states:
  no_outline: "<=1 heading -> single-column layout (outline_nav hidden)"
  with_outline: ">=2 headings -> two-column layout with sticky outline_nav (md:block)"
interactions:
  "click outline item": "scrollIntoView heading (smooth, block:start); no URL hash mutation"
direction: horizontal
# heading IDs scoped per-instance via useId() to prevent collisions
# code block dispatch:
#   mermaid lang -> MermaidDiagram (dynamic import, toggle source/diagram)
#   openapi lang OR yaml block containing "openapi:" -> OpenAPIViewer
#   others -> preformatted block
children:
  - id: md_content
    flex: 1
    scroll: vertical
    # renders markdown via ReactMarkdown + remark-gfm
  - id: outline_nav
    width: 160
    scroll: none
    # sticky top-0; hidden on mobile (md:block)
    # items: one per heading (h1-h4)
```

```yaml
# Component: MermaidDiagram
# Loaded lazily (dynamic import) inside MarkdownSpecRenderer for mermaid code blocks
id: MermaidDiagram
props:
  content: string
  className?: string
states:
  loading: "animate-pulse h-48 skeleton block (initial; shown until mermaid.render() resolves)"
  error: "red box with error message + collapsible raw source section"
  rendered: "SVG output via dangerouslySetInnerHTML"
direction: block
# init config (dynamic import): {startOnLoad: false, theme: "default", securityLevel: "strict"}
```

```yaml
# Component: OpenAPIViewer
# Used inside MarkdownSpecRenderer for openapi/yaml code blocks; also standalone
id: OpenAPIViewer
props:
  content: string
  # raw YAML string
states:
  populated: "expandable endpoint list (docs view; initial when yaml valid and paths non-empty)"
  no_endpoints: "gray empty message (docs view; yaml valid but no paths)"
  parse_error: "red error message (docs view; yaml parse failure)"
  show_source: "raw preformatted YAML (toggled on by toggle button)"
interactions:
  "click toggle button": "switch between show_source and docs view"
  "click endpoint_summary_row": "toggle endpoint_detail expanded/collapsed"
direction: vertical
children:
  - id: toggle_button
    height: auto
  - id: endpoint_list
    flex: 1
    scroll: vertical
    children:
      - id: endpoint_row
        direction: vertical
        height: auto
        children:
          - id: endpoint_summary_row
            direction: horizontal
            height: auto
            children:
              - id: method_badge
                # color-coded: GET=blue | POST=green | PUT=amber | PATCH=orange | DELETE=red | HEAD=purple | OPTIONS=gray
              - id: endpoint_path
                flex: 1
              - id: endpoint_summary
          - id: endpoint_detail
            # rendered when row is expanded
            direction: vertical
            height: auto
            children:
              - id: parameters_table
                columns: [name, in, required, type, description]
              - id: request_body_block
                # JSON display
              - id: responses_list
                # status + description + content per response
```
