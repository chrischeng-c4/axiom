---
id: "1669"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-profile-observability-applicability
entry: profile
nodes:
  profile: { kind: start, label: "profile signal with bounded metadata" }
  reference: { kind: decision, label: "durable content-addressed blob reference present" }
  durable: { kind: decision, label: "referenced blob is durable and hash/size match" }
  project: { kind: process, label: "project samples mappings functions locations and correlations" }
  query: { kind: terminal, label: "flamegraph top functions diff and trace evidence" }
  reject: { kind: terminal, label: "reject before raw journal acknowledgement" }
edges:
  - { from: profile, to: reference }
  - { from: reference, to: durable, label: "yes" }
  - { from: reference, to: reject, label: "no" }
  - { from: durable, to: project, label: "yes" }
  - { from: durable, to: reject, label: "no" }
  - { from: project, to: query }
---
flowchart LR
    profile([profile]) --> reference{blob reference}
    reference -->|yes| durable{blob durable}
    reference -->|no| reject([reject])
    durable -->|yes| project[project profile]
    durable -->|no| reject
    project --> query([analysis])
```
