---
change_id: genesis-186-28
type: spec_context
created_at: 2026-02-14T03:18:38.770309+00:00
updated_at: 2026-02-14T03:18:38.770309+00:00
iteration: 2
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-genesis
  - cclab-grid
  - cclab-grid-db
  - cclab-ion
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-taipan
  - cclab-titan
  - nebula
---

# Spec Context

## Relevant Specs

- **create-spec** (group: cclab-genesis)
  - relevance: high
  - reason: Core target for LLM enrichment and diagram generation improvements.
  - key sections: Compositional Tag System, Requirement Diagram Mapping, Prompt Template
- **review-spec** (group: cclab-genesis)
  - relevance: high
  - reason: Core target for review cycle improvements and unified verdicts.
  - key sections: Verdict -> Phase Mapping, Threshold Escalation, Review Checklist
- **mermaid-plus-format** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the target output format for enriched diagrams.
  - key sections: Format Specification, YAML Frontmatter Schema
- **mermaid-plus-conversion** (group: cclab-aurora)
  - relevance: medium
  - reason: Relevant for understanding how specs are transformed into visual diagrams.
  - key sections: Conversion Algorithm
- **spec-validator** (group: cclab-aurora)
  - relevance: medium
  - reason: Integration point for validating enriched specs during creation and review.
  - key sections: Type Validation, Completeness Check
- **generate-tasks** (group: cclab-genesis)
  - relevance: medium
  - reason: Planning flow context for task derivation from specs.
  - key sections: Task Generation Algorithm, File Path Extraction
- **verdict-unification** (group: cclab-genesis)
  - relevance: medium
  - reason: Ensures consistency in review results across the workflow.
  - key sections: R1 - Unify spec verdict names, R3 - Unify Rust verdict enums
- **delegate-agent** (group: cclab-genesis)
  - relevance: medium
  - reason: Defines agent invocation and verification mechanisms for enrichment.
  - key sections: Sequence Diagram, Behavior Flowchart

## Dependencies

- cclab-genesis/create-spec depends on cclab/changes/{change_id}/proposal.md#spec_plan
- cclab-genesis/review-spec depends on cclab-genesis/create-spec (spec_created phase)
- cclab-genesis/generate-tasks depends on cclab-genesis/review-spec (all_specs_approved phase)
- cclab-genesis/review-spec uses cclab-aurora/spec-validator for automated completeness check
- cclab-aurora/mermaid-plus-format and cclab-aurora/mermaid-plus-conversion inform cclab-genesis/create-spec enrichment logic

## Gaps

- Absence of --quick flag in create_spec or run_change to bypass LLM enrichment.
- Generic LLM enrichment logic in create_spec lacks deep integration with codebase_context and gap artifacts.
- Review cycle thresholds in review-spec.md lack structured enforcement and alignment with #186 requirements.
- Manual selection requirement for Aurora diagram generation instead of automated detection from change definitions.
