---
id: projects-jet-logic-nx-support-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Jet Nx Support

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/nx-support.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Jet Nx Support

### Overview

This spec owns Jet's Nx monorepo integration. Jet detects an Nx workspace by
finding and parsing `nx.json`, builds or retrieves the Nx project graph, orders
projects topologically, and routes build/install flows through the Nx-aware
workspace mode when appropriate.

| Area | Source | Responsibility |
|------|--------|----------------|
| Nx graph model | `crates/jet/src/pkg_manager/nx.rs` | Parse Nx graph JSON and direct `project.json` files |
| Workspace detection | `crates/jet/src/pkg_manager/workspace.rs` | Prefer Nx over Jet workspace and package workspace modes |
| CLI flags | `crates/jet/src/cli.rs` | `--nx` force mode and Nx project build selection |
| Tests | `crates/jet/tests/nx_support.rs`, `crates/jet/src/pkg_manager/nx_test.rs` | Detection, parsing, topological sort, direct file graph |

### Requirements

```mermaid
---
id: jet-nx-support-requirements
entry: N1
---
requirementDiagram
    requirement N1 {
        id: N1
        text: Workspace detection returns Nx mode when nx json is present and valid
        risk: high
        verifymethod: test
    }
    requirement N2 {
        id: N2
        text: Malformed nx json produces a parse error instead of silently falling back
        risk: medium
        verifymethod: test
    }
    requirement N3 {
        id: N3
        text: Nx takes priority over package json workspaces and Jet workspace mode
        risk: high
        verifymethod: test
    }
    requirement N4 {
        id: N4
        text: Nx project graph supports deterministic topological build ordering
        risk: high
        verifymethod: test
    }
    requirement N5 {
        id: N5
        text: Direct project json scanning can build a project graph without invoking Nx CLI
        risk: high
        verifymethod: test
    }
    requirement N6 {
        id: N6
        text: CLI build uses Nx graph when Nx mode is detected or forced
        risk: high
        verifymethod: test
    }
    requirement N7 {
        id: N7
        text: Forced Nx mode errors clearly when nx json is absent
        risk: medium
        verifymethod: test
    }
```

### Scenarios

```yaml
scenarios:
  - id: S1
    requirement: N1
    title: Repository with nx.json is detected as an Nx workspace
  - id: S2
    requirement: N2
    title: Invalid nx.json fails detection with a parse error
  - id: S3
    requirement: N3
    title: Repository with both nx.json and package workspaces chooses Nx
  - id: S4
    requirement: N4
    title: Graph dependencies produce dependency-first topological order
  - id: S5
    requirement: N5
    title: project.json files are scanned into graph nodes and dependency edges
  - id: S6
    requirement: N6
    title: jet build dispatches through Nx graph when in Nx workspace
  - id: S7
    requirement: N7
    title: jet build --nx outside Nx workspace exits with a clear error
```

### Logic

```mermaid
---
id: jet-nx-support-logic
entry: A
---
flowchart TD
    A[Jet command starts] --> B{--nx forced?}
    B -->|yes| C{nx.json present?}
    C -->|no| D[error --nx set but no nx.json]
    C -->|yes| E[parse nx.json]
    B -->|no| F[WorkspaceMode detect]
    F --> G{nx.json present?}
    G -->|yes| E
    G -->|no| H[fall back to Jet workspace package workspace or single project]
    E --> I[NxWorkspaceManager]
    I --> J{direct project json graph succeeds?}
    J -->|yes| K[NxProjectGraph from files]
    J -->|no| L[run nx graph --json]
    L --> K
    K --> M[topological sort]
    M --> N[build selected projects in order]
```

### Dependency Model

```mermaid
---
id: jet-nx-support-dependencies
entry: NxWorkspaceManager
---
classDiagram
    class NxWorkspaceManager {
        +PathBuf root
        +NxConfig config
        +discover(root)
        +get_project_graph()
        +build_project_graph_from_files()
    }
    class NxProjectGraph {
        +NxGraphData graph
        +topological_sort()
    }
    class NxGraphData {
        +nodes
        +dependencies
    }
    class NxProject {
        +name
        +data
    }
    class WorkspaceMode {
        <<enum>>
        Nx
        Jet
        Package
        Single
    }

    WorkspaceMode --> NxWorkspaceManager
    NxWorkspaceManager --> NxConfig
    NxWorkspaceManager --> NxProjectGraph
    NxProjectGraph --> NxGraphData
    NxGraphData --> NxProject
```

### Schema

```yaml
schemas:
  NxConfig:
    rust_type: NxConfig
    fields:
      affected:
        type: map
        value: serde_json::Value
      tasks_runner_options:
        type: map
        value: serde_json::Value
  NxProjectGraph:
    rust_type: NxProjectGraph
    fields:
      graph:
        type: NxGraphData
  NxGraphData:
    rust_type: NxGraphData
    fields:
      nodes:
        type: map
        value: NxProject
      dependencies:
        type: map
        value: array<NxDependency>
  NxProject:
    rust_type: NxProject
    fields:
      name:
        type: string
      data:
        type: Option<NxProjectData>
  NxDependency:
    rust_type: NxDependency
    fields:
      source:
        type: string
      target:
        type: string
      type:
        type: string
```

### Test Plan

```mermaid
---
id: jet-nx-support-test-plan
entry: T1
---
flowchart TD
    T1[cargo test -p jet nx_support] --> T2[workspace detection]
    T2 --> T3[malformed nx json error]
    T3 --> T4[Nx priority over package workspaces]
    T4 --> T5[topological sort]
    T5 --> T6[direct project json graph]
    T6 --> T7[pkg_manager nx unit tests]
    T7 --> T8[aw td check logic nx-support spec]
```

### Changes

```yaml
changes:
  - path: .aw/tech-design/crates/jet/logic/nx-support.md
    action: create
    purpose: Re-home and normalize the Nx support TD.
    impl_mode: hand-written
  - path: .aw/tech-design/crates/jet/nx-support.md
    action: delete
    purpose: Remove duplicated loose root spec.
    impl_mode: hand-written
  - path: crates/jet/src/pkg_manager/nx.rs
    action: none
    purpose: Existing Nx graph and workspace manager implementation described by this spec.
    impl_mode: hand-written
  - path: crates/jet/src/pkg_manager/workspace.rs
    action: none
    purpose: Existing workspace detection priority described by this spec.
    impl_mode: hand-written
  - path: crates/jet/src/cli.rs
    action: none
    purpose: Existing Nx CLI dispatch described by this spec.
    impl_mode: hand-written
```
