---
change_id: cclab-taipan
type: gap_spec_knowledge
created_at: 2026-02-12T07:39:32.938540+00:00
updated_at: 2026-02-12T07:39:32.938540+00:00
---

# Gap Analysis: Spec vs Knowledge

## Summary
There are several misalignments between the documented architectural patterns and the current specifications, particularly regarding performance engineering and the overlap between different analysis pipeline patterns.

## Responsibility Boundary Misalignments
- **Analysis Pipeline Ownership**: Both `aurora-codegen-system` and `prism-class-diagram` specify multi-stage analysis pipelines. There is no clear boundary defined in either spec or knowledge base on whether the Taipan compiler should strictly follow the Aurora codegen pattern or the Prism analysis pattern, or how they should be unified. (Severity: MEDIUM)
- **Agent Skill Integration**: `30-claude/skills.md` defines how to extend capabilities via skills, but the `cli-architecture` and `aurora-codegen-system` specs do not define how the compiler's analysis or generation capabilities should be exposed or consumed as Agent Skills. (Severity: LOW)

## Unreflected Knowledge Patterns
- **Profiling and OS-Level Optimization**: `orbit/performance-tuning.md` provides a comprehensive guide for OS-level optimizations and profiling. However, the `02-architecture-principles` (Performance First) and `00-architecture` (Performance Targets) specs lack specific requirements for profiling hooks or OS-level tuning parameters for the Taipan execution environment. (Severity: MEDIUM)
- **Data Mapper for Complex Logic**: The `05-titan/architecture-guide.md` pattern of using Data Mappers to decouple domain from persistence is not reflected or adapted in the `aurora-codegen-system` spec, which could lead to tight coupling between the Taipan IR and its target code generation backends. (Severity: MEDIUM)

## Contradictions
- **DSL Pattern Consistency**: `grid/formula-syntax.md` documents a specific DSL syntax pattern, but the `spec_context` gaps indicate that Taipan lacks a syntax spec entirely. This creates a risk that Taipan's grammar will contradict established DSL conventions used elsewhere in the project without a spec to reconcile them. (Severity: MEDIUM)
