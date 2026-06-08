```yaml
module: "@cclab/ui/feedback"
components: [SyncStatusBadge, ConfirmDialog, ConnectRepoForm]
```

```yaml
# Component: SyncStatusBadge
id: SyncStatusBadge
props:
  status: string
  className?: string
states:
  synced: "green-100 bg + green-800 text + CheckCircle icon + 'Synced'"
  pending: "yellow-100 bg + yellow-800 text + Clock icon + 'Pending'"
  failed: "red-100 bg + red-800 text + AlertCircle icon + 'Failed'"
  unknown: "gray-100 bg + gray-700 text + RefreshCw icon + status string as label"
interactions: {}
direction: horizontal
height: auto
```

```yaml
# Component: ConfirmDialog
id: ConfirmDialog
props:
  open: boolean
  title: string
  description: string
  confirmLabel?: string
  onConfirm: "() => void"
  onCancel: "() => void"
states:
  closed: "not rendered"
  open: "modal overlay with title, description, Cancel + Confirm buttons"
interactions:
  "click Confirm": "onConfirm()"
  "click Cancel or backdrop": "onCancel()"
direction: vertical
```

```yaml
# Component: ConnectRepoForm
# Used in: tab_settings_content > settings_repo_section
id: ConnectRepoForm
props:
  projectId: string
  onClose: "() => void"
states:
  idle: "form fields editable, connect button enabled"
  connecting: "form fields disabled, connect button shows 'Connecting...' (isPending)"
  error: "red AlertCircle + error message above form, form re-enabled"
interactions:
  "click connect": "POST /api/projects/{id}/connect-repo -> onClose() on success"
  "click cancel or X": "onClose()"
direction: vertical
width: 600
children:
  - id: close_button
    # X icon in card header top-right
  - id: error_alert
    # conditional: shown when mutation isError; red background + error message
  - id: field_gitlab_url
    # input: GitLab URL (e.g., https://gitlab.com)
  - id: field_gitlab_project_id
    # input: GitLab Project ID (numeric)
  - id: field_gitlab_access_token
    # input: Access Token (password field)
  - id: field_path
    # input: Repository path (optional)
  - id: action_buttons
    direction: horizontal
    # Cancel button + Connect Repo button
    action: "POST /api/projects/{id}/connect-repo {gitlab_url, gitlab_project_id, gitlab_access_token, path}"
```
