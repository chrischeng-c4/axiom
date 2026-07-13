---
id: '1578'
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-release-dockerfile-fixture-parity-applicability
entry: version
nodes:
  version:
    kind: start
    label: "Tape release renderer derives the current package release version"
  render:
    kind: process
    label: "tape dockerfile render --variant release produces the canonical artifact"
  fixture:
    kind: process
    label: "Committed apps/tape/Dockerfile.release must contain the same versioned artifact"
  gate:
    kind: terminal
    label: "deploy_cli byte-parity test passes; no runtime or image-publish behavior changes"
edges:
  - { from: version, to: render }
  - { from: render, to: fixture }
  - { from: fixture, to: gate }
---
flowchart TD
    version[Tape release renderer derives the current package release version] --> render[tape dockerfile render --variant release produces the canonical artifact]
    render --> fixture[Committed apps/tape/Dockerfile.release must contain the same versioned artifact]
    fixture --> gate([deploy_cli byte-parity test passes; no runtime or image-publish behavior changes])
```
