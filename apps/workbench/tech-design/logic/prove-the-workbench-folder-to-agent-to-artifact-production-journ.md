---
id: '2201'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-production-journey-applicability
entry: folder
nodes:
  folder: { kind: start, label: "select registered canonical folder" }
  agent: { kind: process, label: "select and launch native agent through real PTY" }
  output: { kind: process, label: "stream transcript and explicit OSC 7 cwd" }
  context: { kind: process, label: "render Markdown, Git, and optional AW context" }
  provenance: { kind: process, label: "show canonical source navigation and labels" }
  recover: { kind: decision, label: "agent or artifact error?" }
  evidence: { kind: process, label: "retain viewport, interaction, accessibility, transcript, and context evidence" }
  done: { kind: terminal, label: "repeatable production gate" }
edges:
  - { from: folder, to: agent }
  - { from: agent, to: output }
  - { from: output, to: context }
  - { from: context, to: provenance }
  - { from: provenance, to: recover }
  - { from: recover, to: evidence, label: "recovered or none" }
  - { from: evidence, to: done }
---
flowchart LR
    folder([Folder]) --> agent[Native PTY]
    agent --> output[Transcript and OSC7 cwd]
    output --> context[Markdown Git AW context]
    context --> provenance[Source navigation]
    provenance --> recover{Recovery state?}
    recover --> evidence[Retained evidence]
    evidence --> done([Production gate])
```

Assemble the existing folder registry, provider-neutral real PTY, OSC 7 active cwd, renderer registry, and canonical provenance into one desktop session boundary. Tauri commands start exactly one selected Claude Code, Codex, or AGY process in the selected canonical folder, stream bounded output, accept input/resize/terminate, expose recoverable unavailable-agent errors, and render Markdown/Git/optional AW context from the explicit active cwd. A deterministic local shell exercises the same session type in tests; installed vendor CLIs remain optional.

Upgrade the three-column primary state without moving folder ownership into the runtime: the center pane gains accessible agent choice, start/retry, terminal transcript/input, status and active-cwd disclosure; the right pane gains keyboard-operable Markdown/Git/AW choices, safe preview, provenance label, and source navigation. Preserve the established dark developer-tool system, visible focus, minimum 44px targets, stable 150-300ms color/opacity transitions, minimum 16px body readability, constrained-width fit, and reduced-motion behavior.

`production_journey` runs the real-PTY functional journey plus Jet at 1440x900 and 860x720. Jet writes a deterministic v1 manifest and retained screenshots; Rust adds transcript and context summaries. The manifest maps every assertion to its artifact and the external contract plus capability gate invoke the exact same Cargo test command. Optional graph/page providers remain outside the production prerequisite.
