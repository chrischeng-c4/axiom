```yaml
module: "@cclab/ui/changes"
components: [ChangeList, CommentsSection]
```

```yaml
# Component: ChangeList
id: ChangeList
props:
  changes: "Change[]"
  onSelect: "(change: Change) => void"
  selectedId?: string
states:
  empty: "FileText icon + 'No changes yet' + helper text, centered"
  populated: "divide-y list of change_row entries; selected row has bg-blue-50 + border-l-2 border-blue-500"
interactions:
  "click row": "onSelect(change)"
  "click external_mr_link": "open in new tab (stopPropagation)"
direction: vertical
children:
  - id: change_row
    direction: horizontal
    height: auto
    children:
      - id: change_title
        flex: 1
        # truncated text
      - id: status_badge
        # color-coded: draft | in_progress | review | merged | closed
      - id: issue_ids_count
      - id: spec_ids_count
        # conditional: renders only when spec_ids.length > 0
      - id: branch_name
        # optional: renders only when branch_name present; GitBranch icon
      - id: external_mr_link
        # optional: renders only when external_mr_url present; ExternalLink icon
      - id: created_at
        # formatted date via toLocaleDateString()
```

```yaml
# Component: CommentsSection
id: CommentsSection
props:
  projectId: string
  issueNumber: number
states:
  loading: "animate-pulse skeleton blocks"
  empty: "renders nothing (returns null)"
  populated: "vertical list of comment cards with author + timestamp + markdown body"
interactions: {}
direction: vertical
data_source: "GET /api/projects/{projectId}/issues/{issueNumber}/comments"
```
