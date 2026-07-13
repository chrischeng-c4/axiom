---
id: "1593"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-network-policy-ingress-boundary
entry: component
nodes:
  component: { kind: start, label: "Opt-in NetworkPolicy component" }
  select: { kind: process, label: "Select only Tape server pods" }
  client: { kind: process, label: "Allow labeled client namespaces on TCP 7137" }
  scrape: { kind: process, label: "Allow labeled Prometheus namespaces on TCP 7137" }
  deny: { kind: terminal, label: "All other ingress denied by policy" }
edges:
  - { from: component, to: select }
  - { from: select, to: client }
  - { from: select, to: scrape }
  - { from: client, to: deny }
  - { from: scrape, to: deny }
---
flowchart TD
  component[Opt-in NetworkPolicy component] --> select[Select only Tape server pods]
  select --> client[Allow labeled client namespaces on TCP 7137]
  select --> scrape[Allow labeled Prometheus namespaces on TCP 7137]
  client --> deny([All other ingress denied])
  scrape --> deny
```
