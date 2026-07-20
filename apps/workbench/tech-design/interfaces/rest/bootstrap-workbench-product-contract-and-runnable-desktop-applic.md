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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: Register apps/workbench as an explicit workspace member.
  - path: Cargo.lock
    action: modify
    section: logic
    impl_mode: hand-written
    description: Lock the selected Tauri 2 desktop-host dependency graph.
  - path: apps/workbench/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
    description: Define the workbench library and desktop binary with Tauri 2 host/build dependencies.
  - path: apps/workbench/build.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Run the Tauri build integration for the desktop host.
  - path: apps/workbench/tauri.conf.json
    action: create
    section: logic
    impl_mode: hand-written
    description: Configure one local WebView window and disable release bundling for the initial host slice.
  - path: apps/workbench/ui/index.html
    action: create
    section: logic
    impl_mode: hand-written
    description: Provide a bounded non-placeholder bootstrap document; no three-column product behavior lands here.
  - path: apps/workbench/src/lib.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Own the Tauri builder, window-ready marker, test-only shutdown handshake, and clean lifecycle exit.
  - path: apps/workbench/src/main.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: Launch the native desktop host without defining a second agent CLI or session surface.
  - path: apps/workbench/tests/desktop_launch_smoke.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Spawn the real desktop binary, wait for host-ready, request shutdown, and require a clean bounded exit.
  - path: apps/workbench/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Replace the skeleton Brief with the terminal-first product boundary and native-agent authority statement.
  - path: apps/workbench/CAPABILITIES.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Register Terminal-first Agent Workbench with epic 2171 and child work-root evidence.
  - path: apps/workbench/CONTRIBUTING.md
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: Record the narrow launch-smoke command and native-host ownership rules.
  - path: apps/workbench/aw.toml
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: Replace the bootstrap true gate with the desktop launch-smoke verification command.
```
