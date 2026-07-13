---
id: '1578'
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-release-dockerfile-fixture-parity-contract
entry: render
nodes:
  render:
    kind: start
    label: "tape dockerfile render --variant release emits tape@0.4.5 from the current renderer"
  compare:
    kind: decision
    label: "Does committed Dockerfile.release exactly match the rendered release artifact?"
  stale:
    kind: process
    label: "Replace only the stale tape@0.4.4 fixture version and matching build comment"
  pass:
    kind: terminal
    label: "cargo test -p tape --test deploy_cli dockerfile_render_reproduces_committed_fixtures -- --exact passes"
edges:
  - { from: render, to: compare }
  - { from: compare, to: stale, label: "no" }
  - { from: stale, to: pass }
  - { from: compare, to: pass, label: "yes" }
---
flowchart TD
    render[tape dockerfile render --variant release emits tape@0.4.5 from the current renderer] --> compare{Does committed Dockerfile.release exactly match the rendered release artifact?}
    compare -->|no| stale[Replace only the stale tape@0.4.4 fixture version and matching build comment]
    stale --> pass([cargo test -p tape --test deploy_cli dockerfile_render_reproduces_committed_fixtures -- --exact passes])
    compare -->|yes| pass
```
