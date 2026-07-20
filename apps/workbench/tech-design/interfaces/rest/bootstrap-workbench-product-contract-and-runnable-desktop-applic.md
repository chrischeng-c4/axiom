---
id: '2191'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-desktop-bootstrap-flow
entry: invoke
nodes:
  invoke: { kind: start, label: "launch workbench desktop binary" }
  host: { kind: process, label: "initialize Tauri 2 host and one WebView window" }
  content: { kind: process, label: "load bounded local bootstrap document" }
  ready: { kind: process, label: "emit ready only after the window reaches host-ready state" }
  shutdown: { kind: decision, label: "normal close or smoke shutdown request?" }
  cleanup: { kind: process, label: "close window and drain host lifecycle resources" }
  done: { kind: terminal, label: "process exits cleanly" }
edges:
  - { from: invoke, to: host }
  - { from: host, to: content }
  - { from: content, to: ready }
  - { from: ready, to: shutdown }
  - { from: shutdown, to: cleanup, label: "either path" }
  - { from: cleanup, to: done }
---
flowchart TD
    invoke([launch workbench]) --> host[Tauri 2 host plus one WebView]
    host --> content[local bootstrap document]
    content --> ready[host-ready marker]
    ready --> shutdown{normal close or smoke shutdown?}
    shutdown --> cleanup[close window and drain resources]
    cleanup --> done([clean process exit])
```

The desktop boundary is Rust plus Tauri 2. The initial document is deliberately local and bounded; later renderer slices may compile Jet/WASM or another renderer into the same WebView contract without changing PTY ownership. The host owns native window and process lifecycle only. Native Claude Code, Codex, and AGY processes remain authoritative and are not launched in this slice. The smoke protocol is test-only host control, not a user-facing CLI session model.
