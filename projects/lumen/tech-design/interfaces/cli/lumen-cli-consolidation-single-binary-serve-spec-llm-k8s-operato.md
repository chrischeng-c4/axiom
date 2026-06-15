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
---
flowchart TD
    parse{{parse lumen subcommand}} -->|serve| serve[serve: serving node data-plane]
    parse -->|spec| spec[spec: offline OpenAPI / JSON-schema / shapes / fields]
    parse -->|llm| llm[llm topic: offline agent narrative]
    parse -->|k8s| k8s{k8s subcommand}
    parse -->|-h / --help| help[help: long_about points to lumen llm outline]
    k8s -->|operator| operator[operator: reconcile controller cfg feature operator]
    k8s -->|gen-crd| gencrd[gen-crd: print Lumen CRD YAML]
    operator -. feature off .-> nofeat([clear error: built without operator support])
    serve --> done([command complete])
    spec --> done
    llm --> done
    gencrd --> done
    help --> done
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
cli:
  name: lumen
  about: "Single agent-first CLI for the lumen search engine. Agents start here: lumen llm outline."
  commands:
    - name: serve
      about: "Run a serving node (HTTP API + background apply loop)."
      args:
        - {name: "--host", env: "LUMEN_HOST", default: "127.0.0.1"}
        - {name: "--port", env: "LUMEN_PORT", default: "7373"}
        - {name: "--wal", env: "LUMEN_WAL", default: "embedded", choices: ["embedded", "nats"]}
        - {name: "--persistence", env: "LUMEN_PERSISTENCE", default: "cbor", choices: ["cbor", "segment"]}
    - name: spec
      about: "Print the machine-readable integration contract (offline, no server)."
      args:
        - {name: "--format", default: "openapi", choices: ["openapi", "openapi-yaml", "json-schema"]}
        - {name: "--shapes", kind: "flag"}
        - {name: "--fields", kind: "flag"}
    - name: llm
      about: "Print agent-facing topics (offline). outline is the entry point."
      args:
        - {name: "topic", kind: "positional", default: "outline", choices: ["outline", "workflow", "integration", "quickstart", "recipes"]}
        - {name: "--format", default: "md", choices: ["md", "json"]}
    - name: k8s
      about: "Kubernetes operator and CRD generation (manifest/render only; lumen does not deploy)."
      commands:
        - name: operator
          about: "Run the Lumen CRD reconcile controller (container CMD; requires build feature operator)."
        - name: gen-crd
          about: "Print the Lumen CustomResourceDefinition YAML."
```

## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: kube, spec: "0.98", features: [runtime, derive, client], optional: true }
  - { name: k8s-openapi, spec: "0.24", features: [v1_32], optional: true }
  - { name: schemars, spec: "0.8", optional: true }
```
