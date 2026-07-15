---
id: aw-dx-trait-derivation
summary: Promote the explicit agent_facing capability profile trait so it derives the Developer & Agent Experience root and links its normative CONTRIBUTING convention.
fill_sections: [schema, logic, unit-test, changes]
capability_refs:
  - id: capability-control-plane
    role: primary
    gap: agent-facing-dx-baseline-trait
    claim: agent-facing-dx-baseline-trait
    coverage: full
    rationale: "Trait-derived capability baselines must enforce settled ecosystem conventions and expose actionable readiness gaps."
---
<!-- HANDWRITE-BEGIN gap="missing-generator:logic:dx-trait-derivation" tracker="#1481" reason="The trait choice preserves existing project scope while making future agent-facing adoption explicit and enforceable." -->

# Developer & Agent Experience Trait Derivation

## Schema
<!-- type: schema lang: yaml -->

```yaml
trait:
  id: agent_facing
  derives: [developer-agent-experience]
  enforces: "## DX convention: every service and CLI ships a Developer & Agent Experience capability"
selection:
  existing_agent_facing_projects: []
  rationale: "Promote the existing explicit trait; no current project profile is newly obligated, and service/CLI profile adoption remains project-scoped."
missing_baseline:
  report: "capability profile requires missing baseline capabilities: developer-agent-experience"
  remediation: "aw capability draft --project <project>"
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-dx-trait-derivation-flow
entry: profile
nodes:
  profile: { kind: start, label: "Read capability profile traits" }
  selected: { kind: decision, label: "agent_facing declared?" }
  unchanged: { kind: terminal, label: "Preserve existing baseline set" }
  derive: { kind: process, label: "Require developer-agent-experience" }
  declared: { kind: decision, label: "README owns DX root?" }
  ready: { kind: terminal, label: "Continue capability readiness" }
  gap: { kind: terminal, label: "Report missing root and draft remediation" }
edges:
  - { from: profile, to: selected }
  - { from: selected, to: unchanged, label: "no" }
  - { from: selected, to: derive, label: "yes" }
  - { from: derive, to: declared }
  - { from: declared, to: ready, label: "yes" }
  - { from: declared, to: gap, label: "no" }
---
flowchart TD
    profile[Read capability.profile.traits] --> selected{agent_facing declared?}
    selected -->|no| unchanged[Preserve existing required baselines]
    selected -->|yes| derive[Require developer-agent-experience]
    derive --> declared{README owns root?}
    declared -->|yes| ready[Continue normal capability readiness]
    declared -->|no| gap[Report missing root and aw capability draft remediation]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-dx-trait-derivation-unit
coverage_kind: unit
evidence:
  command: cargo test -p agentic-workflow --lib agent_facing_trait_requires_developer_agent_experience_with_remediation -- --nocapture
---
requirementDiagram
  requirement derive_dx {
    id: UT1
    text: "agent_facing derives developer-agent-experience"
    risk: high
    verifymethod: test
  }
  requirement remediate_gap {
    id: UT2
    text: "a missing root reports the normal capability baseline remediation"
    risk: high
    verifymethod: test
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/doc_mirror.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: Promote agent_facing, its baseline capability, and its DX convention anchor in the single trait registry.
  - path: apps/agentic-workflow/src/cli/capability.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: Prove the selected trait derives and reports the missing Developer & Agent Experience baseline.
  - path: CONTRIBUTING.md
    action: modify
    section: schema
    impl_mode: codegen
    description: Regenerate the trait table through aw meta sync; do not hand-edit its marker-owned rows.
```

<!-- HANDWRITE-END -->
