```yaml
module: "@cclab/ui/layout"
components: [Header]
```

```yaml
# Component: Header
# Top bar of the app shell
id: Header
props:
  onMenuClick: "() => void"
states: {}
interactions:
  "click menu button": "onMenuClick() -- toggles mobile sidebar"
  "click capture button": "html2canvas screenshot -> clipboard"
direction: horizontal
height: 56
children:
  - id: menu_button
    width: 40
    # Menu icon; visible on mobile only (lg:hidden)
  - id: spacer
    flex: 1
  - id: capture_button
    width: 40
    # Camera icon; screenshots page to clipboard
```
