---
id: '1578'
capability_refs:
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "dedicated-statefulset-operator-topology"
    claim: "dedicated-statefulset-operator-topology"
    coverage: partial
    rationale: "The fixture parity gate protects the existing Dockerfile render contract without changing the StatefulSet/operator implementation."
summary: >
  WI #1578 restores byte parity between the Tape release Dockerfile renderer
  and its committed fixture after the shared release version advanced to 0.4.5.
  It changes no runtime, deployment topology, or image publishing behavior.
fill_sections: [logic, changes]
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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/Dockerfile.release
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Synchronize the committed release fixture's image-build comment and TAPE_VERSION default from tape@0.4.4 to the renderer's current tape@0.4.5 output. No Dockerfile instructions or runtime behavior change. generator gap: missing-generator:fixture:release-dockerfile-parity (#1578)."
```
