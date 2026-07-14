---
id: "1658"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-otlp-gcp-batch-ingest
entry: client
nodes:
  client: { kind: start, label: "OTLP or GCP client" }
  auth: { kind: process, label: "project scoped authentication" }
  body: { kind: process, label: "bounded body and gzip decode" }
  decode: { kind: decision, label: "endpoint and content type?" }
  normalize: { kind: process, label: "normalize JSON or protobuf into OperationalEventV2" }
  admit: { kind: process, label: "project quota and concurrency admission" }
  govern: { kind: process, label: "validate schema and apply governance" }
  append: { kind: process, label: "append accepted items through RaftStateMachine" }
  batch: { kind: terminal, label: "ordered batch outcomes" }
  partial: { kind: terminal, label: "OTLP partial success" }
edges:
  - { from: client, to: auth }
  - { from: auth, to: body }
  - { from: body, to: decode }
  - { from: decode, to: normalize, label: "events, OTLP, or GCP" }
  - { from: normalize, to: admit }
  - { from: admit, to: govern }
  - { from: govern, to: append }
  - { from: append, to: batch, label: "events write" }
  - { from: append, to: partial, label: "OTLP" }
---
flowchart TD
    client([OTLP or GCP client]) --> auth[project scoped authentication]
    auth --> body[bounded body and gzip decode]
    body --> decode{endpoint and content type?}
    decode --> normalize[normalize JSON or protobuf into OperationalEventV2]
    normalize --> admit[project quota and concurrency admission]
    admit --> govern[validate schema and apply governance]
    govern --> append[append accepted items through RaftStateMachine]
    append -->|events write| batch([ordered batch outcomes])
    append -->|OTLP| partial([OTLP partial success])
```

The transport boundary is bounded and synchronous: authentication, decompression, decoding, normalization, admission, validation, governance, and durable append complete before an item is accepted. Invalid siblings do not block valid items. Every accepted signal becomes `OperationalEventV2` before the shared append path, so transport cannot bypass Raft, privacy policy, idempotency, or raw-journal durability.
