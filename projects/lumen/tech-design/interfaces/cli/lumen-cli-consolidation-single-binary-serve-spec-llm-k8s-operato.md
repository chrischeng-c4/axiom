---
id: lumen-cli-consolidation
summary: Consolidate lumen into a single agent-first CLI — serve / spec / llm / k8s operator — removing the openapi-dump, bench, and consumer sibling binaries and folding the operator behind the `operator` feature gate so a non-operator build is kube-free.
fill_sections: [logic, cli, manifest, unit-test]
---

# TD: lumen CLI consolidation (single binary)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-cli-dispatch
entry: parse
nodes:
  parse: {kind: start, label: "parse lumen subcommand"}
  serve: {kind: process, label: "serve: run serving node (data-plane)"}
  spec: {kind: process, label: "spec: offline OpenAPI / JSON-schema / shapes / fields"}
  llm: {kind: process, label: "llm <topic>: offline agent narrative (outline=entry)"}
  k8s: {kind: decision, label: "k8s subcommand"}
  operator: {kind: process, label: "k8s operator: reconcile controller (cfg feature=operator)"}
  gencrd: {kind: process, label: "k8s gen-crd: print Lumen CRD YAML"}
  help: {kind: process, label: "--help: long_about points to 'lumen llm outline'"}
  nofeat: {kind: terminal, label: "clear error: built without operator support"}
  done: {kind: terminal, label: "command complete"}
edges:
  - {from: parse, to: serve, label: "serve"}
  - {from: parse, to: spec, label: "spec"}
  - {from: parse, to: llm, label: "llm"}
  - {from: parse, to: k8s, label: "k8s"}
  - {from: parse, to: help, label: "-h/--help"}
  - {from: k8s, to: operator, label: "operator"}
  - {from: k8s, to: gencrd, label: "gen-crd"}
  - {from: operator, to: nofeat, label: "feature off"}
  - {from: serve, to: done}
  - {from: spec, to: done}
  - {from: llm, to: done}
  - {from: gencrd, to: done}
  - {from: help, to: done}
```
