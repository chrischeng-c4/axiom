---
number: 1057
title: "sdd: add Backend/MLE/Agent section types — grpc, graphql, model, prompt"
state: open
labels: [type:enhancement, priority:p3, crate:sdd]
group: "new-section-types"
---

# #1057 — sdd: add Backend/MLE/Agent section types — grpc, graphql, model, prompt

Parent: #1051

## Remaining section types (deferred — add when encountered)

### Backend

| type | lang | note |
|------|------|------|
| `grpc` | `protobuf` (proto3 IDL) | `rpc-api` is OpenRPC (JSON-RPC only) |
| `graphql` | `graphql` (SDL) | Neither OpenAPI nor OpenRPC |

### MLE

| type | lang | note |
|------|------|------|
| `model` | `yaml` or `json` | Model architecture, layer definition, I/O tensor shapes, metrics |
| `pipeline` | shared with SRE issue #1056 | Data/ML pipeline DAG |

### Agent

| type | lang | note |
|------|------|------|
| `prompt` | `markdown` | Template + variables + system instructions. Currently `logic` covers it, but semantics differ from flowchart |
