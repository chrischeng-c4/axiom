---
id: beam-cli-shell
summary: >
  Add Beam's first Rust service crate and binary shell so the project has an
  agent-drivable `beam` CLI. The slice wires the required standard
  `llm`/`upgrade`/`issue` verbs through shared `cli-std`, exposes placeholder
  service verbs for future collection/index/query/deploy work, and keeps the
  crate CPU/GPU-neutral with no CUDA, Metal, or vector runtime dependency.
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "beam-cli-convention-and-vector-verbs"
    coverage: partial
    rationale: >
      This TD turns Beam's CLI Interface capability root from a placeholder epic
      into the first independently verifiable non-epic implementation slice.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-cli-shell-contract
entry: start
nodes:
  start:       { kind: start,    label: "beam <subcommand>" }
  parse:       { kind: process,  label: "clap parses top-level command surface from projects/beam/src/main.rs" }
  llm:         { kind: terminal, label: "llm [--topic outline|boundaries|operations] [--format md|json] -> cli_std::llm::render(beam topics)" }
  upgrade:     { kind: terminal, label: "upgrade [--version TAG] [--check] -> cli_std::upgrade with ToolInfo{name=beam, release_prefix=beam@}" }
  issue:       { kind: terminal, label: "issue <search|view|create> -> cli_std::issue scoped to project:beam" }
  serve:       { kind: terminal, label: "serve placeholder exits with tracked 'not implemented yet: HTTP service shell' diagnostic" }
  collections: { kind: terminal, label: "collections placeholder exits with tracked 'not implemented yet: collection lifecycle' diagnostic" }
  index:       { kind: terminal, label: "index placeholder exits with tracked 'not implemented yet: index lifecycle' diagnostic" }
  query:       { kind: terminal, label: "query placeholder exits with tracked 'not implemented yet: vector query' diagnostic" }
  dockerfile:  { kind: terminal, label: "dockerfile placeholder exits with tracked 'not implemented yet: dockerfile render' diagnostic" }
  k8s:         { kind: terminal, label: "k8s placeholder exits with tracked 'not implemented yet: k8s render/operator' diagnostic" }
  boundary:    { kind: process,  label: "llm topics state: Beam owns GPU vector DB/index lifecycle; Lumen owns mixed search/ranking/dedup" }
  nogpu:       { kind: process,  label: "crate dependencies remain CPU/GPU-neutral in this slice; no CUDA/Metal/runtime driver deps" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: llm,         label: "llm" }
  - { from: parse, to: upgrade,     label: "upgrade" }
  - { from: parse, to: issue,       label: "issue" }
  - { from: parse, to: serve,       label: "serve" }
  - { from: parse, to: collections, label: "collections" }
  - { from: parse, to: index,       label: "index" }
  - { from: parse, to: query,       label: "query" }
  - { from: parse, to: dockerfile,  label: "dockerfile" }
  - { from: parse, to: k8s,         label: "k8s" }
  - { from: llm,   to: boundary }
  - { from: parse, to: nogpu }
---
flowchart TD
    start([beam CLI]) --> parse[Parse top-level subcommand]
    parse -->|llm| llm[Render offline agent topics via cli_std::llm]
    parse -->|upgrade| upgrade[Delegate self-update/check to cli_std::upgrade]
    parse -->|issue| issue[Delegate issue search/view/create to cli_std::issue project:beam]
    parse -->|serve| serve[Placeholder: service shell not implemented]
    parse -->|collections| collections[Placeholder: collection lifecycle not implemented]
    parse -->|index| index[Placeholder: index lifecycle not implemented]
    parse -->|query| query[Placeholder: vector query not implemented]
    parse -->|dockerfile| dockerfile[Placeholder: dockerfile render not implemented]
    parse -->|k8s| k8s[Placeholder: k8s render/operator not implemented]
    llm --> boundary[Topics name Beam/Lumen boundary explicitly]
    parse --> nogpu[No GPU runtime dependency in this slice]
```
