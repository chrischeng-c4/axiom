---
id: aw-ec-tool-binding-config
summary: Per-project EC tool-binding config (`ec.<category>` map on the Project model) plus verify-ec dispatch — bound categories run an external tool (arena/rig/meter) command; unbound categories fall back to the EC manifest command. Exit-code gated; report-JSON folding deferred.
fill_sections: [logic, schema, unit-test]
capability_refs:
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-tool-binding-dispatch
    claim: ec-tool-binding-dispatch
    coverage: full
    rationale: "Lets verify-ec route an EC category to an external measurement tool declared in config, instead of only the fixed manifest command."
---

# TD: aw EC tool-binding config + verify-ec dispatch

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: verify-ec-dispatch
entry: start
nodes:
  start:        { kind: start,    label: "run_project_ec_command(case, project)" }
  has_ec:       { kind: decision, label: "project.ec has case.category?" }
  lookup:       { kind: process,  label: "binding = project.ec[case.category]" }
  build:        { kind: process,  label: "cmd = binding.command() (arena/rig/meter)" }
  build_ok:     { kind: decision, label: "command built ok?" }
  bad_tool:     { kind: terminal, label: "Failed: unknown tool in binding" }
  use_manifest: { kind: process,  label: "cmd = case.command (manifest / cargo-test fallback)" }
  spawn:        { kind: process,  label: "sh -c cmd in project_root" }
  exit0:        { kind: decision, label: "exit code == 0?" }
  passed:       { kind: terminal, label: "ProjectEcCommandReport: Passed" }
  failed:       { kind: terminal, label: "ProjectEcCommandReport: Failed (exit, tails)" }
edges:
  - { from: start,        to: has_ec }
  - { from: has_ec,       to: lookup,       label: "yes" }
  - { from: has_ec,       to: use_manifest, label: "no" }
  - { from: lookup,       to: build }
  - { from: build,        to: build_ok }
  - { from: build_ok,     to: spawn,        label: "ok" }
  - { from: build_ok,     to: bad_tool,     label: "err" }
  - { from: use_manifest, to: spawn }
  - { from: spawn,        to: exit0 }
  - { from: exit0,        to: passed,       label: "yes" }
  - { from: exit0,        to: failed,       label: "no" }
---
flowchart TD
    start([run_project_ec_command]) --> has_ec{project.ec has case.category?}
    has_ec -->|yes| lookup[binding = project.ec category]
    has_ec -->|no| use_manifest[cmd = case.command fallback]
    lookup --> build[cmd = binding.command]
    build --> build_ok{command built ok?}
    build_ok -->|ok| spawn[sh -c cmd in project_root]
    build_ok -->|err| bad_tool([Failed: unknown tool])
    use_manifest --> spawn
    spawn --> exit0{exit code == 0?}
    exit0 -->|yes| passed([Passed])
    exit0 -->|no| failed([Failed])
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "aw-ec-tool-binding"
title: "EC tool-binding additions to the Project model"
description: "Project gains an optional `ec` map (category -> tool binding) verified by aw health --verify-ec. Project-scoped (like TD), declared before `workspaces`."
type: object
properties:
  ec:
    type: object
    description: "Optional. Map of EC category (free string: correctness | benchmark | security | stability | ...) to a tool binding. A category absent from this map falls back to the EC manifest command (the cargo-test default)."
    additionalProperties:
      $ref: "#/$defs/EcBinding"
$defs:
  EcBinding:
    type: object
    additionalProperties: false
    required: [tool]
    properties:
      tool:
        type: string
        enum: ["arena", "rig", "meter"]
        description: "Which external measurement tool verifies this EC category."
      spec:
        type: string
        description: "arena: comparison spec path -> `arena run --spec <spec>`."
      dir:
        type: string
        description: "rig: scenario directory -> `rig run --dir <dir>`."
      meter:
        type: string
        description: "meter: meter.toml path the meter invocation honors for [gate] ceilings."
```
