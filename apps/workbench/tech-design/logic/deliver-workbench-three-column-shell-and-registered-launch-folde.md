---
id: '2192'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-three-column-folder-shell
entry: load
nodes:
  load: { kind: start, label: "load persisted registered-folder identity and selected id" }
  render: { kind: process, label: "render launch folders, terminal preparation region, and read-only context region" }
  choose: { kind: decision, label: "register, select, navigate, or collapse?" }
  register: { kind: process, label: "open native directory picker, canonicalize the directory, and persist identity" }
  select: { kind: process, label: "persist selected folder id and publish its canonical path to the future launch boundary" }
  navigate: { kind: process, label: "move folder focus with ArrowUp or ArrowDown and activate with Enter or Space" }
  collapse: { kind: process, label: "toggle compact launch-folder rail without persisting layout state" }
  recover: { kind: process, label: "show actionable empty, cancelled, invalid-path, or persistence error state" }
  ready: { kind: terminal, label: "three-column shell remains ready without launching a child process" }
edges:
  - { from: load, to: render }
  - { from: render, to: choose }
  - { from: choose, to: register, label: "add folder" }
  - { from: choose, to: select, label: "select folder" }
  - { from: choose, to: navigate, label: "keyboard navigation" }
  - { from: choose, to: collapse, label: "collapse or expand" }
  - { from: choose, to: recover, label: "empty, cancel, or error" }
  - { from: register, to: render }
  - { from: select, to: ready }
  - { from: navigate, to: render }
  - { from: collapse, to: render }
  - { from: recover, to: render }
---
flowchart LR
    load([Load folder registry]) --> render[Render three-column shell]
    render --> choose{User action}
    choose -->|Add| register[Native directory picker and persist identity]
    choose -->|Select| select[Persist selected id and expose launch path]
    choose -->|Keyboard| navigate[Roving folder focus]
    choose -->|Collapse| collapse[Compact folder rail]
    choose -->|Empty or error| recover[Actionable state]
    register --> render
    navigate --> render
    collapse --> render
    recover --> render
    select --> ready([Ready; no child process])
```

The shell keeps two state boundaries deliberately separate. Rust owns a small
`LaunchFolderRegistry` containing canonical directory identity plus the
selected folder id, persisted below the Tauri application configuration
directory. The navigation collapsed state is transient UI preference and is
not written into that registry. Terminal cwd remains absent until child WI
#2194.

The native host exposes commands to load the registry, choose/register a folder
through the Tauri dialog plugin, select a registered folder, and resolve the
selected canonical path for the future agent-launch boundary. Registration
rejects non-directories, de-duplicates canonical paths, and preserves the prior
selection when a picker is cancelled or persistence fails. No command in this
slice starts a process, allocates a PTY, or mutates AW.

The local WebView renders accessible `nav`, `main`, and `aside` landmarks.
The folder list uses ordinary buttons plus ArrowUp/ArrowDown and Enter/Space
navigation, the collapse control remains keyboard reachable, and a live status
region reports cancellation and errors. Desktop and constrained-desktop
layouts keep all three regions functional: the constrained layout uses a
compact folder rail and a bounded context pane rather than hiding primary
actions.

A browser-only test bridge supplies deterministic command results to the same
UI module for Jet E2E execution. Production always selects the Tauri invoke
bridge. The journey records rendered desktop and constrained-width PNG
evidence, interaction results, focus order, landmark/readability assertions,
and the future launch path without spawning any agent process.
