---
id: cue-react-on-jet-frontend-shell
summary: React-on-Jet frontend shell contract for Cue Artifact Studio and Admin while keeping Jet as the owning frontend substrate.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# React On Jet Frontend Shell

## Frontend Shell Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/react-on-jet-shell/v0"
title: Cue React On Jet Shell v0
type: object
additionalProperties: false
required: [sites, substrate, validation]
properties:
  sites:
    type: array
    items:
      type: object
      required: [name, path, audience]
      properties:
        name: { enum: [artifact_studio, admin] }
        path: { enum: [projects/cue/artifact-studio, projects/cue/admin] }
        audience: { enum: [project_owner, platform_operator] }
  substrate:
    type: object
    required: [owner, ui_runtime, package_manager]
    properties:
      owner: { const: jet }
      ui_runtime: { enum: [react_tsx] }
      package_manager: { enum: [jet] }
  validation:
    type: object
    required: [jet_check, jet_build, e2e]
    properties:
      jet_check: { type: boolean }
      jet_build: { type: boolean }
      e2e: { type: boolean }
```

## Substrate Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-react-on-jet-substrate-logic
entry: BuildFrontendSlice
nodes:
  BuildFrontendSlice: { kind: start, label: frontend slice starts }
  UseJet: { kind: process, label: use Jet build/dev/test path }
  JetFails: { kind: decision, label: Jet-specific failure? }
  ReproduceInVite: { kind: process, label: reproduce minimal TSX in Vite }
  ViteWorks: { kind: decision, label: Vite works? }
  FileJetIssue: { kind: process, label: file project:jet issue }
  NarrowWorkaround: { kind: process, label: add removable Cue workaround }
  DebugCueCode: { kind: process, label: debug Cue app code }
  SliceReady: { kind: terminal, label: frontend slice ready }
edges:
  - { from: BuildFrontendSlice, to: UseJet, label: begin }
  - { from: UseJet, to: JetFails, label: validate }
  - { from: JetFails, to: SliceReady, label: no }
  - { from: JetFails, to: ReproduceInVite, label: yes }
  - { from: ReproduceInVite, to: ViteWorks, label: compare }
  - { from: ViteWorks, to: FileJetIssue, label: yes }
  - { from: FileJetIssue, to: NarrowWorkaround, label: issue linked }
  - { from: NarrowWorkaround, to: SliceReady, label: workaround tested }
  - { from: ViteWorks, to: DebugCueCode, label: no }
  - { from: DebugCueCode, to: SliceReady, label: fixed }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: artifact_studio_runs_on_jet
    given: [Artifact Studio TSX source exists]
    when: [frontend validation runs]
    then: [Jet owns check, build, dev, and e2e path]
  - id: admin_is_separate_site
    given: [Admin shell is added]
    when: [routing is inspected]
    then: [Admin is separate from Artifact Studio and targets platform operators]
  - id: jet_failure_reproduced
    given: [Jet fails on valid TSX]
    when: [Vite repro works]
    then: [project:jet issue is linked and workaround is narrow]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/projects/cue/react-on-jet-frontend-shell.md
    action: create
    impl_mode: hand-written
    description: Define React TSX on Jet shell contract for Cue frontend sites.
  - path: projects/cue/artifact-studio/
    action: modify
    impl_mode: hand-written
    description: Keep Artifact Studio as owner-facing Jet site backed by Cue backend APIs.
  - path: projects/cue/admin/
    action: create
    impl_mode: hand-written
    description: Add separate platform-operator Admin site when #1991 implementation starts.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  artifact_studio_jet_build:
    kind: frontend
    verifies: [Jet builds Artifact Studio]
  artifact_studio_backend_e2e:
    kind: browser
    verifies: [Artifact Studio renders backend state]
  admin_site_separation:
    kind: frontend
    verifies: [Admin route or site is distinct from Artifact Studio]
  jet_substrate_repro_policy:
    kind: review
    verifies: [Jet blockers have Vite comparison and linked project:jet issue]
```
