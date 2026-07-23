---
id: '2445'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-delivery-channels
entry: build
nodes:
  build: { kind: start, label: selected-channel-build }
  stable: { kind: process, label: stable-product }
  beta: { kind: process, label: beta-product }
  roots: { kind: process, label: isolated-state-roots }
  registry: { kind: process, label: profile-runtime-registry }
  verify: { kind: decision, label: independent-runtime-proof }
  done: { kind: terminal, label: safe-daily-and-beta-use }
edges:
  - { from: build, to: stable, label: stable }
  - { from: build, to: beta, label: beta }
  - { from: stable, to: roots }
  - { from: beta, to: roots }
  - { from: roots, to: registry }
  - { from: registry, to: verify }
  - { from: verify, to: done, label: yes }
---
flowchart LR
  build([Selected build skill]) -->|Stable| stable[Axiom Workbench]
  build -->|Beta| beta[Axiom Workbench Beta]
  stable --> roots[Separate state roots]
  beta --> roots
  roots --> registry[Profile runtime registry]
  registry --> verify{Independent?}
  verify -->|Yes| done([Safe daily use])
```

Stable is `Axiom Workbench`, bundle id `com.axiom.workbench`, profile `stable`, and state root `~/.axiom-workbench`. Beta is `Axiom Workbench Beta`, bundle id `com.axiom.workbench.beta`, profile `beta`, and state root `~/.axiom-workbench-beta`. Each has its own runtime registry, lock, logs, and project metadata. Stable uses the approved cobalt/amber icon; Beta uses the ultraviolet/mint icon.

Build scripts select an explicit Xcode scheme/configuration and only terminate the matching bundle executable. `workbench-build-beta` may never touch Stable. `workbench-build-stable` builds/opens Stable only when invoked. The CLI accepts `--profile stable|beta`, defaults to stable, and derives all local paths from that profile; a cross-profile snapshot cannot discover another runtime.

The legacy cclab bundle is not deleted. A new product starts independently; its one-runtime lease is scoped to its own state root. Tests inspect bundle identities, app names, state-root derivation, icon assets, and two simultaneous profile registries without Computer Use.
