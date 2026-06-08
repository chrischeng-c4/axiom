---
id: packages/cclab-ui/api
---

## Component Props

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "@cclab/ui component props",
  "definitions": {
    "ProjectSpec": {
      "type": "object",
      "required": ["id", "name", "content"],
      "properties": {
        "id": { "type": "number" },
        "name": { "type": "string" },
        "content": { "type": "string" },
        "path": { "type": "string" }
      }
    },
    "DAGNode": {
      "type": "object",
      "required": ["node_id", "status"],
      "properties": {
        "node_id": { "type": "string" },
        "module_key": { "type": "string" },
        "status": { "type": "string" },
        "artifact": { "type": "string" },
        "spec_id": { "type": ["number", "null"] }
      }
    },
    "Run": {
      "type": "object",
      "required": ["id", "label", "nodes"],
      "properties": {
        "id": { "type": "string" },
        "label": { "type": "string" },
        "nodes": { "type": "array", "items": { "$ref": "#/definitions/DAGNode" } },
        "created_at": { "type": "string", "format": "date-time" }
      }
    },
    "Change": {
      "type": "object",
      "required": ["id", "title", "status"],
      "properties": {
        "id": { "type": "string" },
        "title": { "type": "string" },
        "status": { "type": "string", "enum": ["draft", "in_progress", "review", "merged", "closed"] },
        "issue_ids": { "type": "array", "items": { "type": "string" } },
        "spec_ids": { "type": "array", "items": { "type": "number" } },
        "branch_name": { "type": ["string", "null"] },
        "external_mr_url": { "type": ["string", "null"] },
        "created_at": { "type": "string", "format": "date-time" }
      }
    },
    "FileEntry": {
      "type": "object",
      "required": ["name", "path", "type"],
      "properties": {
        "name": { "type": "string" },
        "path": { "type": "string" },
        "type": { "type": "string", "enum": ["file", "directory"] },
        "size": { "type": ["number", "null"] }
      }
    },
    "FileContent": {
      "type": "object",
      "required": ["name", "content"],
      "properties": {
        "name": { "type": "string" },
        "content": { "type": "string" },
        "encoding": { "type": "string", "enum": ["utf-8", "base64"] },
        "size": { "type": "number" },
        "mime_type": { "type": "string" }
      }
    },
    "SelectOption": {
      "type": "object",
      "required": ["value", "label"],
      "properties": {
        "value": { "type": "string" },
        "label": { "type": "string" }
      }
    },
    "SpecFileBrowserProps": {
      "type": "object",
      "required": ["specs"],
      "properties": {
        "specs": {
          "type": "array",
          "items": { "$ref": "#/definitions/ProjectSpec" }
        },
        "emptyMessage": { "type": "string" }
      }
    },
    "MarkdownSpecRendererProps": {
      "type": "object",
      "required": ["content"],
      "properties": {
        "content": { "type": "string" }
      }
    },
    "MermaidDiagramProps": {
      "type": "object",
      "required": ["content"],
      "properties": {
        "content": { "type": "string" },
        "className": { "type": "string" }
      }
    },
    "OpenAPIViewerProps": {
      "type": "object",
      "required": ["content"],
      "properties": {
        "content": {
          "type": "string",
          "description": "Raw YAML string"
        }
      }
    },
    "SpecPipelineHistoryProps": {
      "type": "object",
      "required": ["liveNodes", "completedRuns", "isActive", "projectId", "onGenerate"],
      "properties": {
        "liveNodes": {
          "type": "array",
          "items": { "$ref": "#/definitions/DAGNode" }
        },
        "completedRuns": {
          "type": "array",
          "items": { "$ref": "#/definitions/Run" }
        },
        "isActive": { "type": "boolean" },
        "projectId": { "type": "string" },
        "onGenerate": {
          "type": "string",
          "const": "callback",
          "description": "() => void"
        }
      }
    },
    "RunEntryProps": {
      "type": "object",
      "required": ["label", "is_live", "nodes"],
      "properties": {
        "label": { "type": "string" },
        "is_live": { "type": "boolean" },
        "nodes": {
          "type": "array",
          "items": { "$ref": "#/definitions/DAGNode" }
        }
      }
    },
    "ArtifactViewerProps": {
      "type": "object",
      "required": ["projectId", "specId"],
      "properties": {
        "projectId": { "type": "string" },
        "specId": { "type": ["number", "null"] }
      }
    },
    "ChangeListProps": {
      "type": "object",
      "required": ["changes", "onSelect"],
      "properties": {
        "changes": {
          "type": "array",
          "items": { "$ref": "#/definitions/Change" }
        },
        "onSelect": {
          "type": "string",
          "const": "callback",
          "description": "(change: Change) => void"
        },
        "selectedId": { "type": "string" }
      }
    },
    "FileBrowserProps": {
      "type": "object",
      "required": ["projectId"],
      "properties": {
        "projectId": { "type": "string" },
        "defaultBranch": { "type": "string" },
        "onClose": {
          "type": "string",
          "const": "callback",
          "description": "() => void"
        }
      }
    },
    "FileTreeItemProps": {
      "type": "object",
      "required": ["entry", "onNavigate", "onSelect"],
      "properties": {
        "entry": { "$ref": "#/definitions/FileEntry" },
        "onNavigate": {
          "type": "string",
          "const": "callback",
          "description": "(path: string) => void"
        },
        "onSelect": {
          "type": "string",
          "const": "callback",
          "description": "(path: string) => void"
        },
        "isSelected": { "type": "boolean" }
      }
    },
    "FileViewerProps": {
      "type": "object",
      "required": ["file"],
      "properties": {
        "file": { "$ref": "#/definitions/FileContent" }
      }
    },
    "InlineEditTextProps": {
      "type": "object",
      "required": ["value", "onSave"],
      "properties": {
        "value": { "type": "string" },
        "onSave": {
          "type": "string",
          "const": "callback",
          "description": "(value: string) => void"
        },
        "className": { "type": "string" }
      }
    },
    "InlineEditSelectProps": {
      "type": "object",
      "required": ["value", "options", "onSave", "displayValue"],
      "properties": {
        "value": { "type": "string" },
        "options": {
          "type": "array",
          "items": { "$ref": "#/definitions/SelectOption" }
        },
        "onSave": {
          "type": "string",
          "const": "callback",
          "description": "(value: string) => void"
        },
        "displayValue": { "type": "string" }
      }
    },
    "InlineEditLabelsProps": {
      "type": "object",
      "required": ["labels", "onSave"],
      "properties": {
        "labels": {
          "type": "array",
          "items": { "type": "string" }
        },
        "onSave": {
          "type": "string",
          "const": "callback",
          "description": "(labels: string[]) => void"
        }
      }
    },
    "InlineEditDescriptionProps": {
      "type": "object",
      "required": ["value", "onSave"],
      "properties": {
        "value": { "type": "string" },
        "onSave": {
          "type": "string",
          "const": "callback",
          "description": "(value: string) => void"
        }
      }
    },
    "SyncStatusBadgeProps": {
      "type": "object",
      "required": ["status"],
      "properties": {
        "status": {
          "type": "string",
          "description": "Known values: synced, pending, failed. Unknown values render as gray fallback."
        },
        "className": { "type": "string" }
      }
    },
    "ConfirmDialogProps": {
      "type": "object",
      "required": ["open", "title", "description", "onConfirm", "onCancel"],
      "properties": {
        "open": { "type": "boolean" },
        "title": { "type": "string" },
        "description": { "type": "string" },
        "confirmLabel": { "type": "string" },
        "onConfirm": {
          "type": "string",
          "const": "callback",
          "description": "() => void"
        },
        "onCancel": {
          "type": "string",
          "const": "callback",
          "description": "() => void"
        }
      }
    },
    "CommentsSectionProps": {
      "type": "object",
      "required": ["projectId", "issueNumber"],
      "properties": {
        "projectId": { "type": "string" },
        "issueNumber": { "type": "integer" }
      }
    },
    "ConnectRepoFormProps": {
      "type": "object",
      "required": ["projectId", "onClose"],
      "properties": {
        "projectId": { "type": "string" },
        "onClose": { "type": "string", "const": "callback", "description": "() => void" }
      }
    },
    "HeaderProps": {
      "type": "object",
      "required": ["onMenuClick"],
      "properties": {
        "onMenuClick": {
          "type": "string",
          "const": "callback",
          "description": "() => void"
        }
      }
    }
  }
}
```

## Exports Map

```yaml
exports:
  "@cclab/ui/spec-viewer":
    - SpecFileBrowser
    - MarkdownSpecRenderer
    - MermaidDiagram
    - OpenAPIViewer
  "@cclab/ui/pipeline":
    - SpecPipelineHistory
    - RunEntry
    - ArtifactViewer
  "@cclab/ui/changes":
    - ChangeList
    - CommentsSection
  "@cclab/ui/file-browser":
    - FileBrowser
    - FileTreeItem
    - FileViewer
  "@cclab/ui/editing":
    - InlineEditText
    - InlineEditSelect
    - InlineEditLabels
    - InlineEditDescription
  "@cclab/ui/feedback":
    - SyncStatusBadge
    - ConfirmDialog
    - ConnectRepoForm
  "@cclab/ui/layout":
    - Header
```
