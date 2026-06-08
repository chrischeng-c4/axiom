---
change: section-type-coverage
group: new-section-types
date: 2026-03-24
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| change-spec | section-types | high | Section Type → Spec Lang Mapping table: add 16 new types (e2e-scenario, test-fixture, perf-test, threat-model, auth-matrix, security-test, container, deploy, cloud-resource, pipeline, observability, grpc, graphql, model, prompt) with lang and code fence, section_rules: add keyword patterns for grpc, graphql, e2e-scenario, threat-model, auth-matrix, security-test, container, deploy, cloud-resource, pipeline, observability, model, prompt, test-fixture, perf-test, fill_order: assign priority positions — e2e-scenario after test-plan (p1), security types p2, all SRE/BE/MLE/Agent types p3, section_prompts: add CLI flag descriptions and guidance for all 16 new types, Cross-reference: new types use existing _sdd.id / _sdd.refs conventions (YAML DSL types) or language-specific id/ref (json for grpc/graphql, markdown for prompt) |
| reference-context | section-types | high | spec_plan.sections enum in artifact schema: extend allowed values with all 16 new section type names, Section rule engine keyword-to-type matching updated to recognize new types |
| codegen-system | generate | medium | SpecIR enum needs new variants for grpc, graphql, prompt, model, e2e-scenario, threat-model, auth-matrix, security-test, container, deploy, cloud-resource, pipeline, observability, test-fixture, perf-test, Router data flow: route new section types through SpecIR parser path (JSON Schema envelope for grpc/graphql, markdown for prompt) |
| spec-ir-contract | generate | medium | SpecIR enum type: new variants for the new section types follow the same pattern as existing Deploy, Wireframe, Component, DesignToken, SpecBundle multi-spec context supports new types for generators |
| artifact-tools | interfaces | medium | sdd_artifact_create_change_spec payload schema sections enum must include all new type names for validation to pass |
| spec-ir-schema | generate | medium | SpecIR YAML manifest kind enum: grpc, graphql, model, prompt may need new Kind values |
| spec-model | generate | low | Background on spec type semantics — justifies why new types are distinct rather than overloading existing types (e.g. grpc is not rest-api, prompt is not logic) |
| code-generator-contract | generate | low | Generator output patterns — establishes how new section types will eventually map to generated output files (.proto, .graphql SDL, Dockerfile, etc.) |
| generator-react | generate | low | SpecIRGenerator trait pattern — reference for how new section types will plug into generator routing (can_generate + generate_from_ir) |
| cli-commands | interfaces | low | cclab sdd artifact command routing — context for how per-section-type CLI flags (section_prompts) are structured and invoked |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| change-spec-types | modify | crates/cclab-sdd/logic/change-spec.md | overview, changes |
| reference-context-types | modify | crates/cclab-sdd/logic/reference-context.md | overview, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: section-type-coverage

**Verdict**: APPROVED

### Summary

Comprehensive reference context covering change-spec (section types table, rules, fill order), reference-context (spec_plan sections enum), codegen system (SpecIR variants), spec-ir-contract, artifact-tools, and supporting specs. spec_plan correctly targets change-spec.md and reference-context.md as modify actions.

### Issues

No issues found.
