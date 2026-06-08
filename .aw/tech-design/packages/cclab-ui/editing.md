```yaml
module: "@cclab/ui/editing"
components: [InlineEditText, InlineEditSelect, InlineEditLabels, InlineEditDescription]
```

```yaml
# Component: InlineEditText
id: InlineEditText
props:
  value: string
  onSave: "(value: string) => void"
  className?: string
states:
  view: "button displaying value; Pencil icon on hover"
  editing: "input field with current value; Check + X icon buttons"
interactions:
  "click value": "switch to editing state"
  "Enter or click Check": "save if trimmed value changed -> onSave(newValue) -> view"
  "Escape or click X": "discard draft -> view"
direction: horizontal
height: auto
```

```yaml
# Component: InlineEditSelect
id: InlineEditSelect
props:
  value: string
  options: "{value: string, label: string}[]"
  onSave: "(value: string) => void"
  displayValue: string
states:
  view: "button displaying displayValue; Pencil icon on hover"
  editing: "select dropdown with options; autoFocus"
interactions:
  "click value": "switch to editing state"
  "change selection": "onSave(newValue) -> view (auto-save on change)"
  "blur": "switch to view"
direction: horizontal
height: auto
```

```yaml
# Component: InlineEditLabels
id: InlineEditLabels
props:
  labels: "string[]"
  onSave: "(labels: string[]) => void"
states:
  view: "badge list or 'Add labels' placeholder; Pencil icon on hover"
  editing: "input field (comma-separated); Check + X icon buttons"
interactions:
  "click labels area": "switch to editing state"
  "Enter or click Check": "parse comma-separated -> filter empty -> onSave(labels) -> view"
  "Escape or click X": "discard draft -> view"
direction: vertical
height: auto
```

```yaml
# Component: InlineEditDescription
id: InlineEditDescription
props:
  value: string
  onSave: "(value: string) => void"
states:
  view: "Card with markdown-rendered content (ReactMarkdown + remarkGfm); Pencil icon on hover"
  editing: "Card with 16-row textarea; Cancel + Save buttons in footer"
interactions:
  "click description card": "switch to editing state"
  "click Save": "onSave(newValue) -> view"
  "click Cancel": "discard draft -> view"
direction: vertical
```
