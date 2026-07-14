---
id: "1669"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-profile-observability-contract
entry: upload
nodes:
  upload: { kind: start, label: "OTel profile payload and metadata" }
  blob: { kind: process, label: "validate bounds then durably store content-addressed blob" }
  event: { kind: process, label: "write bounded profile event with hash size encoding and correlations" }
  commit: { kind: process, label: "Raft commit and durable raw cursor" }
  projection: { kind: process, label: "materialize samples functions locations mappings labels and time range" }
  analysis: { kind: terminal, label: "flamegraph top functions diff and trace correlation" }
  rebuild: { kind: start, label: "raw replay" }
  verify: { kind: decision, label: "referenced durable blob exists and digest matches" }
  reject: { kind: terminal, label: "typed missing or corrupt blob error" }
edges:
  - { from: upload, to: blob }
  - { from: blob, to: event }
  - { from: event, to: commit }
  - { from: commit, to: projection }
  - { from: projection, to: analysis }
  - { from: rebuild, to: verify }
  - { from: verify, to: projection, label: "yes" }
  - { from: verify, to: reject, label: "no" }
---
flowchart LR
    upload([profile]) --> blob[durable blob] --> event[bounded event] --> commit[commit] --> projection[profile view] --> analysis([analysis])
    rebuild([raw replay]) --> verify{blob valid}
    verify -->|yes| projection
    verify -->|no| reject([typed error])
```
