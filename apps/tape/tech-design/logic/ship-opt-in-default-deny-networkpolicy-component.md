---
id: "1593"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-network-policy-contract
entry: ingress
nodes:
  ingress: { kind: start, label: "Ingress reaches Tape server selector" }
  client_label: { kind: decision, label: "Source namespace labeled tape.cclab.dev/client-access=true?" }
  scrape_label: { kind: decision, label: "Source has monitoring access labels?" }
  allow: { kind: terminal, label: "Allow TCP 7137 only" }
  deny: { kind: terminal, label: "Deny ingress" }
edges:
  - { from: ingress, to: client_label }
  - { from: client_label, to: allow, label: "yes" }
  - { from: client_label, to: scrape_label, label: "no" }
  - { from: scrape_label, to: allow, label: "yes" }
  - { from: scrape_label, to: deny, label: "no" }
---
flowchart TD
  ingress[Ingress reaches Tape server selector] --> client{Client namespace label?}
  client -->|yes| allow([Allow TCP 7137 only])
  client -->|no| scrape{Monitoring access labels?}
  scrape -->|yes| allow
  scrape -->|no| deny([Deny ingress])
```
