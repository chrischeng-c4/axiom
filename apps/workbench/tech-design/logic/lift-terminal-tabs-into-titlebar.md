---
id: '2463'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-titlebar-terminal-tabs
entry: native-window
nodes:
  native-window: { kind: start, label: "native hidden-titlebar window" }
  sidebar: { kind: process, label: "preserve sidebar safe area" }
  terminal: { kind: process, label: "extend terminal workspace through top safe area" }
  tabs: { kind: terminal, label: "top-aligned terminal tab strip" }
edges:
  - { from: native-window, to: sidebar }
  - { from: native-window, to: terminal }
  - { from: terminal, to: tabs }
---
flowchart LR
    window([Native window]) --> sidebar[Projects safe area]
    window --> terminal[Terminal top safe-area expansion]
    terminal --> tabs([Top-aligned terminal tabs])
```

This change applies only to the central terminal workspace in the native macOS `NavigationSplitView` detail region. The terminal tab strip draws through the window's top content safe area so it occupies the titlebar-adjacent space, while the sidebar and auxiliary column retain their normal safe-area layout.

`WorkbenchView` remains the split-layout owner. It applies top-edge safe-area expansion only to `terminalWorkspace`; it does not alter `folderSidebar` or `auxiliaryColumn`. Existing tab controls, identifiers, lifecycle composition, and renderer identity remain unchanged. The tab strip stays the first visual and keyboard-focusable element in the terminal workspace.

The Projects sidebar titlebar controls remain outside this expansion. No terminal process, tab identity, renderer, project selection, or PTY cwd changes as part of this layout update.
