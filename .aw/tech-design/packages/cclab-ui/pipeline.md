```yaml
module: "@cclab/ui/pipeline"
components: [SpecPipelineHistory, RunEntry, ArtifactViewer]
```

```yaml
# Component: SpecPipelineHistory
id: SpecPipelineHistory
props:
  liveNodes: DAGNode[]
  completedRuns: Run[]
  isActive: boolean
  projectId: string
  onGenerate: "() => void"
states:
  no_history: "empty message only"
  active_or_history: "generate_button + two-panel layout"
interactions:
  "click generate button": "onGenerate()"
  "click node (has artifact)": "set selectedNode"
  "click same node": "set selectedNode to null"
direction: vertical
children:
  # RunNodes groups nodes by module_key into fixed rows and collapsible Features section
  - id: generate_button
    height: auto
  - id: pipeline_two_panel
    direction: horizontal
    min_height: 300
    children:
      - id: run_history
        flex: 1
        scroll: vertical
        children:
          - id: live_run_entry
          - id: completed_run_entries
        # run_history internal structure:
        # Each RunEntry contains RunNodes which groups nodes by module_key:
        # - fixed_rows: nodes without ':feature:' in node_id (list_tree, grouping, shared, architecture)
        # - feature_group: collapsible section for nodes with ':feature:' in node_id
        #   states: { collapsed: "header showing 'Features (completed/total)'", expanded: "header + feature node rows" }
        #   interactions: { "click header": "toggle collapsed/expanded" }
      - id: artifact_viewer
        flex: 2
        scroll: vertical
        # ArtifactViewer fetches GET /api/projects/{projectId}/specs/{specId} for selected node
```

```yaml
# Component: RunEntry
id: RunEntry
props:
  label: string
  is_live: boolean
  nodes: DAGNode[]
states:
  collapsed: "header only"
  expanded: "header + node_rows"
interactions:
  "click header": "toggle between collapsed and expanded"
direction: vertical
children:
  - id: run_entry_header
    direction: horizontal
    height: auto
    children:
      - id: run_label
        flex: 1
      - id: live_indicator
  - id: node_rows
    scroll: vertical
```

```yaml
# Component: ArtifactViewer
id: ArtifactViewer
props:
  projectId: string
  specId: "number | null"
states:
  no_selection: "centered prompt text 'Select a node to view its artifact'"
  loading: "animate-pulse skeleton"
  error: "red error text"
  rendered: "format-aware content (mermaid | openapi | markdown | json | raw)"
interactions: {}
direction: vertical
data_source: "GET /api/projects/{projectId}/specs/{specId}"
```
