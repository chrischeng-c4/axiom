---
id: sdd-interfaces-services-spec-service-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Spec Service Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/spec_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ApiSpecData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 38 |  |
| `CreateSpecInput` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 48 |  |
| `DiagramData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 94 |  |
| `RequirementData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 110 |  |
| `ScenarioData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 124 |  |
| `SpecChangeData` | projects/agentic-workflow/src/services/spec_service.rs | struct | pub | 138 |  |
| `create_spec` | projects/agentic-workflow/src/services/spec_service.rs | function | pub | 435 | create_spec(input: CreateSpecInput, project_root: &Path) -> Result<String> |
| `resolve_section_rules` | projects/agentic-workflow/src/services/spec_service.rs | function | pub | 921 | resolve_section_rules(     requirements_text: &str,     design_system: Option<&DesignSystem>, ) -> Vec<SectionEntry> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap spec-service-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/spec_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:spec-service-runtime>"
    description: "Source template owns spec service runtime behavior and tests."
```
