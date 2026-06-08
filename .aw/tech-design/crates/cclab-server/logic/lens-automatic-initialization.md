---
id: lens-init-spec
type: spec
title: "Lens Automatic Initialization"
version: 1
spec_type: algorithm
created_at: 2026-01-27T15:55:23.225112+00:00
updated_at: 2026-01-27T15:55:23.225112+00:00
main_spec_ref: "crates/cclab-server/src/http_server.rs"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, logic, changes]
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Initialization Algorithm"
history:
  - timestamp: 2026-01-27T15:55:23.225112+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-01-27T15:56:56.702512+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_spec"
    action: "created"
    duration_secs: 266.40
  - timestamp: 2026-01-27T15:57:16.001932+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 19.30
  - timestamp: 2026-01-27T16:00:01.481318+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_spec"
    action: "revised"
    duration_secs: 165.48
  - timestamp: 2026-01-27T16:00:24.858738+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 23.38
---

# Lens Automatic Initialization

## Overview
<!-- type: overview lang: markdown -->

Lens handlers for registered projects are initialized automatically at server
startup. This improves responsiveness by pre-indexing registered projects and
avoids the first-request delay caused by lazy initialization. The registry also
survives server restarts so previously registered projects can be reloaded.

The old active file lived at
`.aw/tech-design/crates/cclab-server/lens-init-spec.md`. The canonical TD is
now `.aw/tech-design/crates/cclab-server/logic/lens-automatic-initialization.md`.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: lens-automatic-initialization-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: Registered projects persist across server stops
        risk: medium
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Server startup triggers indexing for registered projects
        risk: high
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Project indexing runs without blocking server availability
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: CLI start merges existing registry projects
        risk: medium
        verifymethod: test
    }
```

### R1: Registry Persistence

Registered projects must be preserved in `~/.cclab/registry.json` even when the
server process is stopped.

### R2: Background Initialization

The server must automatically trigger indexing for all registered projects on
startup.

### R3: Non-blocking Startup

Project indexing must run in background tasks so the server can become
available immediately.

### R4: CLI Integration

The CLI must correctly handle existing projects when starting a new server
instance after a shutdown or crash.

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: A server with three registered projects is shut down
    when: The server is restarted
    then: The three projects are still listed in the registry
  - id: S2
    requirement: R2
    given: A server starts with registered projects in the registry
    when: Startup completes
    then: Background indexing begins for every registered project
  - id: S3
    requirement: R3
    given: A large project is registered
    when: The server starts on port 3456
    then: The server becomes available immediately while indexing continues in the background
  - id: S4
    requirement: R2
    given: A server starts with existing projects in the registry
    when: Background tasks complete
    then: LensHandlerPool contains initialized handlers for all registered projects
  - id: S5
    requirement: R4
    given: A server is started after a system crash
    when: cclab server start --port 3456 runs
    then: The new server instance merges registered projects instead of overwriting them
```

## Initialization Algorithm
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lens-automatic-initialization-algorithm
entry: start_node
---
flowchart TB
    start_node[Start server] --> load_node[Load registry]
    load_node --> spawn_node[Spawn background indexing task]
    spawn_node --> iter_node{Next project path?}
    iter_node -- yes --> index_node[Index project]
    index_node --> iter_node
    iter_node -- no --> end_node[End background task]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/cclab-server/logic/lens-automatic-initialization.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move Lens automatic initialization TD under logic and normalize sections.
  - path: crates/cclab-server/src/http_server.rs
    action: MODIFY
    impl_mode: hand-written
    desc: Load registered projects at server startup and initialize Lens handlers in background tasks.
```
