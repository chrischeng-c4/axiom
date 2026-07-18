---
id: '1970'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mesh-cli-shell-contract
entry: start
nodes:
  start:       { kind: start,    label: "mesh <subcommand>" }
  parse:       { kind: process,  label: "clap parses top-level command surface from apps/mesh/src/main.rs" }
  llm:         { kind: terminal, label: "llm [--topic outline|boundaries] [--format md|json] -> cli_std::llm::render(mesh topics)" }
  upgrade:     { kind: terminal, label: "upgrade [--version TAG] [--check] -> cli_std::upgrade with ToolInfo{name=mesh, release_prefix=mesh@}" }
  issue:       { kind: terminal, label: "issue <search|view|create> -> cli_std::issue scoped to app:mesh" }
  serve:       { kind: terminal, label: "serve placeholder exits (code 3) with tracked 'not implemented yet: HTTP service shell' diagnostic" }
  collections: { kind: terminal, label: "collections placeholder exits (code 3) with tracked 'not implemented yet: collection lifecycle' diagnostic" }
  nodes:       { kind: terminal, label: "nodes placeholder exits (code 3) with tracked 'not implemented yet: node write/read' diagnostic" }
  edges:       { kind: terminal, label: "edges placeholder exits (code 3) with tracked 'not implemented yet: edge write/read' diagnostic" }
  query:       { kind: terminal, label: "query placeholder exits (code 3) with tracked 'not implemented yet: traversal/path query' diagnostic" }
  dockerfile:  { kind: terminal, label: "dockerfile placeholder exits (code 3) with tracked 'not implemented yet: dockerfile render' diagnostic" }
  k8s:         { kind: terminal, label: "k8s placeholder exits (code 3) with tracked 'not implemented yet: k8s render/operator' diagnostic" }
  boundary:    { kind: process,  label: "llm topics state: Mesh owns typed node/edge property-graph storage, traversal/path query, and a log-driven derived index (never system of record); it does NOT own vector ANN (Beam), lexical/semantic/perceptual search (Lumen), or OLAP aggregation (Cube)" }
  noheavydeps: { kind: process,  label: "crate dependencies stay CLI-shell-only in this slice; no GPU/raft/http-server runtime deps" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: llm,         label: "llm" }
  - { from: parse, to: upgrade,     label: "upgrade" }
  - { from: parse, to: issue,       label: "issue" }
  - { from: parse, to: serve,       label: "serve" }
  - { from: parse, to: collections, label: "collections" }
  - { from: parse, to: nodes,       label: "nodes" }
  - { from: parse, to: edges,       label: "edges" }
  - { from: parse, to: query,       label: "query" }
  - { from: parse, to: dockerfile,  label: "dockerfile" }
  - { from: parse, to: k8s,         label: "k8s" }
  - { from: llm,   to: boundary }
  - { from: parse, to: noheavydeps }
---
flowchart TD
    start([mesh CLI]) --> parse[Parse top-level subcommand]
    parse -->|llm| llm[Render offline agent topics via cli_std::llm]
    parse -->|upgrade| upgrade[Delegate self-update/check to cli_std::upgrade]
    parse -->|issue| issue[Delegate issue search/view/create to cli_std::issue app:mesh]
    parse -->|serve| serve[Placeholder: HTTP service shell not implemented, exit 3]
    parse -->|collections| collections[Placeholder: collection lifecycle not implemented, exit 3]
    parse -->|nodes| nodes[Placeholder: node write/read not implemented, exit 3]
    parse -->|edges| edges[Placeholder: edge write/read not implemented, exit 3]
    parse -->|query| query[Placeholder: traversal/path query not implemented, exit 3]
    parse -->|dockerfile| dockerfile[Placeholder: dockerfile render not implemented, exit 3]
    parse -->|k8s| k8s[Placeholder: k8s render/operator not implemented, exit 3]
    llm --> boundary[Topics name Mesh/Beam/Lumen/Cube boundary explicitly]
    parse --> noheavydeps[No GPU/raft/http-server runtime dependency in this slice]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
  - path: "apps/mesh/Cargo.toml"
    action: create
    section: logic
    impl_mode: hand-written
  - path: "apps/mesh/build.rs"
    action: create
    section: logic
    impl_mode: hand-written
  - path: "apps/mesh/src/lib.rs"
    action: create
    section: logic
    impl_mode: hand-written
  - path: "apps/mesh/src/main.rs"
    action: create
    section: logic
    impl_mode: hand-written
  - path: "apps/mesh/tests/cli_contract.rs"
    action: create
    section: unit-test
    impl_mode: hand-written
```
