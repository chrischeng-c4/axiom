---
change: codegen-td-to-code
group: codegen-td-to-code-main
date: 2026-04-10
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| spec-diff-codegen | crates/sdd/logic | high | R1, R2, R3, R4, R5, R6, R7, R8, R9, R10 |
| code-generator-contract | crates/sdd/generate | high | CodeGenerator trait, SpecIRGenerator trait |
| codegen-system | crates/sdd/generate | high | Generator registry, SpecIR dispatch |
| mermaid-plus-format | crates/sdd/generate | high | Frontmatter format, YAML schema per diagram type |
| requirement-plus-enhancement | crates/sdd/generate | high | SysML v1.6 requirementDiagram, impl_at field |
| spec-ir-schema | crates/sdd/generate | high | SpecIR schema for parsing, Diagram content types |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| sdd-codegen-graph-envelope | create | crates/sdd/logic/codegen-graph-envelope.md | overview, schema, changes |
| sdd-codegen-type-system | create | crates/sdd/logic/codegen-types.md | overview, schema, changes |
| sdd-codegen-marker-system | create | crates/sdd/logic/codegen-markers.md | overview, requirements, changes |
| sdd-codegen-structural-generators | create | crates/sdd/logic/codegen-structural.md | overview, requirements, changes |
| sdd-codegen-behavioral-generators | create | crates/sdd/logic/codegen-behavioral.md | overview, requirements, scenarios, changes |
| sdd-codegen-documentation-generators | create | crates/sdd/logic/codegen-documentation.md | overview, requirements, changes |
| sdd-codegen-validation-harness | create | crates/sdd/logic/codegen-validation.md | overview, cli, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: codegen-td-to-code

**Verdict**: APPROVED

### Summary

Reference context covers all affected areas. Spec plan is comprehensive with 7 new specs covering the full codegen pipeline: graph envelope, type system, marker system, structural/behavioral/documentation generators, and validation harness. All main_spec_ref paths have subfolders under logic/. Specs are scoped correctly — each covers one logical unit.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
- ✅ Relevance scores are reasonable
- ✅ Key requirements listed per spec are accurate
- ✅ No irrelevant specs included
- ✅ spec_plan: every entry has main_spec_ref set
- ✅ spec_plan: sections are reasonable for the requirements
- ✅ spec_plan: modify entries have valid source paths
  - All entries are create, not modify
- ✅ spec_plan: main_spec_ref paths include a subfolder
- ✅ spec_plan: each spec file covers exactly one logical unit
- ✅ spec_plan: no duplicate section types within a file
- ✅ spec_plan: spec paths mirror source structure

### Issues

No issues found.
