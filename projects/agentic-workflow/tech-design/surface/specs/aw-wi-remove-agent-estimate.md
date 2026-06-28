---
id: aw-wi-remove-agent-estimate
summary: Work-item readiness no longer requires Agent Estimate fields.
fill_sections: [scenarios, logic, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: capability-to-epic-planning
    claim: capability-to-epic-planning
    coverage: full
    rationale: "Readiness without Agent Estimate fields is part of the work-item validation and planning surface."
---
<!-- HANDWRITE-BEGIN gap="missing-generator:schema:c3920953" tracker="pending-tracker" reason="Canonical contract for readiness without estimate fields." -->

# WI Readiness Without Agent Estimate

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-wi-remove-agent-estimate-scenarios
scenarios:
  - id: S1
    title: bounded non-epic passes without estimates
    given:
      - "a non-epic work item has Capability Alignment, Scope, Acceptance Criteria, and Reference Context"
    when:
      - "readiness validation runs before TD"
    then:
      - "validation does not require an Agent Estimate section"
      - "validation does not read agent_minutes, confidence, risk, or human_attention as gates"
  - id: S2
    title: legacy estimate section is inert
    given:
      - "an older work item body still contains ## Agent Estimate"
    when:
      - "the body is parsed, merged, or validated"
    then:
      - "the old section remains ordinary body text"
      - "invalid legacy estimate values do not create readiness errors"
  - id: S3
    title: planning output is estimate-free
    given:
      - "aw wi atomize or aw wi prioritize renders local planning artifacts"
    then:
      - "artifact lines do not summarize agent_minutes or human_attention"
      - "roadmap-sized routing is based on issue type and size heuristics"
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-wi-remove-agent-estimate-logic
entry: start
nodes:
  start: { kind: start, label: "Start WI readiness" }
  epic: { kind: decision, label: "Issue is epic?" }
  required: { kind: process, label: "Check required bounded sections" }
  size: { kind: decision, label: "Looks roadmap-sized?" }
  ready: { kind: terminal, label: "Ready for TD" }
  atomize: { kind: terminal, label: "Needs atomize" }
  triage: { kind: terminal, label: "Needs triage" }
edges:
  - { from: start, to: epic }
  - { from: epic, to: atomize, label: "yes" }
  - { from: epic, to: required, label: "no" }
  - { from: required, to: triage, label: "missing alignment or testability" }
  - { from: required, to: size, label: "bounded sections present" }
  - { from: size, to: atomize, label: "roadmap-sized" }
  - { from: size, to: ready, label: "bounded" }
---
flowchart TD
  start([Start WI readiness]) --> epic{Issue is epic?}
  epic -- yes --> atomize([Needs atomize])
  epic -- no --> required[Check required bounded sections]
  required -- missing alignment or testability --> triage([Needs triage])
  required -- bounded sections present --> size{Looks roadmap-sized?}
  size -- roadmap-sized --> atomize
  size -- bounded --> ready([Ready for TD])
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: aw
    subcommands:
      - name: wi
        affected:
          - create
          - fill-section
          - validate
          - plan
          - atomize
          - prioritize
        removed_public_contract:
          - "## Agent Estimate"
          - "agent_minutes readiness buckets"
          - "confidence/risk/human_attention readiness gates"
        readiness_contract:
          required_sections:
            - Capability Alignment
            - Scope
            - Acceptance Criteria
            - Reference Context
          routing:
            - dependency blockers
            - roadmap-sized work
            - missing alignment or testability
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-wi-remove-agent-estimate-unit-test
coverage_kind: unit
evidence:
  command: "cargo test -p agentic-workflow wi_remove_agent_estimate -- --nocapture"
---
requirementDiagram
  requirement no_estimate_required {
    id: UT1
    text: "bounded non-epic work validates without Agent Estimate"
    risk: medium
    verifymethod: test
  }
  requirement legacy_section_inert {
    id: UT2
    text: "legacy Agent Estimate content does not create readiness errors"
    risk: medium
    verifymethod: test
  }
  requirement planning_text_clean {
    id: UT3
    text: "prioritize output omits estimate fields"
    risk: medium
    verifymethod: test
  }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: wi-remove-agent-estimate-build
    capability_id: work-item-planning
    claim_id: capability-to-epic-planning
    command: cargo build -p agentic-workflow --bin aw
    assertions:
      - "the aw binary builds after removing estimate helpers"
  - id: wi-remove-agent-estimate-spec-check
    capability_id: work-item-planning
    claim_id: capability-to-epic-planning
    command: ./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md
    assertions:
      - "the canonical contract remains parseable by td check"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/issues.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Remove Agent Estimate from WI templates, fill prompts, readiness validation, and planning output.
  - path: projects/agentic-workflow/tech-design/surface/interfaces/src/issues.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Refresh the issues.rs source snapshot after removing estimate helpers and validation.
  - path: AGENTS.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Replace estimate-based bounded-WI guidance with section-based readiness guidance.
  - path: CLAUDE.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Mirror the section-based bounded-WI guidance.
  - path: .agents/skills/aw-wi/SKILL.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Update human-facing aw:wi bounded gate instructions.
  - path: projects/agentic-workflow/templates/cli/mainthread/CLAUDE.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Update aw init CLAUDE template guidance.
  - path: projects/agentic-workflow/templates/cli/mainthread/skills/aw-wi/SKILL.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Update aw init aw-wi skill template guidance.
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Mark planning readiness as section-based instead of estimate-based.
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-wi-draft-valid-by-construction.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Remove Agent Estimate from the draft-valid-by-construction expected section list.
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md
    action: create
    section: schema
    impl_mode: hand-written
    description: Canonical contract for WI readiness without estimate fields.
  - action: annotate
    section: e2e-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the e2e-test section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
<!-- HANDWRITE-END -->
