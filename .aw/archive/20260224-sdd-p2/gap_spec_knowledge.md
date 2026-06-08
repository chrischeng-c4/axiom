---
change_id: sdd-p2
type: gap_spec_knowledge
created_at: 2026-02-23T16:37:54.850997+00:00
updated_at: 2026-02-23T16:37:54.850997+00:00
---

# Gap Analysis: Spec vs Knowledge

## 1. Semantic Logic Exclusion
- **Severity**: High
- **Type**: boundary_misalignment
- **Gap**: 'spec-ir-evaluation.md' explicitly excludes 'Context cascade' and 'Complexity routing' from specs, labeling them as 'Semantic Logic' that lives only in code. However, 'spec-model.md' positions Flowchart+ and Sequence+ as the definitive source for handler logic. This creates a boundary ambiguity for what should be specified.
- **Action Needed**: reconcile_coverage_philosophy
- **Repair Action**: Update 'spec-model.md' to explicitly define the boundary between 'Specifiable Logic' (Flowchart+) and 'Orchestration Logic' (Code-only), or update specs to include cascade logic via Flowchart+ nodes.

## 2. State Machine Phase Inconsistency
- **Severity**: Medium
- **Type**: spec_contradiction
- **Gap**: 'config.md' defines agent chains for 'post_clarification' phases, and 'spec-ir-evaluation.md' identifies them as 'planned but missing in code'. However, Knowledge artifacts (like 'spec-model.md') do not include these phases in the canonical SDD workflow overview, leading to inconsistent mental models of the state machine.
- **Action Needed**: standardize_workflow_phases
- **Repair Action**: Add the 'post_clarification' loop to the canonical SDD workflow diagram in 'spec-model.md' or remove it from the 'config.md' spec if it is deprecated.

## 3. Tooling/Artifact Mismatch (Task Generation)
- **Severity**: Medium
- **Type**: knowledge_not_in_spec
- **Gap**: 'spec_context' identifies that 'sdd_generate_tasks' is 'defined in specs' but implementation uses 'sdd_write_artifact'. Knowledge documents ('spec-model.md', 'code-generator-contract.md') do not mention 'sdd_generate_tasks' as a core or supported generator tool.
- **Action Needed**: align_tooling_knowledge
- **Repair Action**: Add 'sdd_generate_tasks' to the 'API Spec — System Contract' section of 'spec-model.md' as a supported generation tool to align with the technical design.

## 4. Validation Rule Knowledge Gap
- **Severity**: Low
- **Type**: knowledge_not_in_spec
- **Gap**: 'config.md' contains extensive validation configuration (regex patterns, severity maps). Knowledge artifacts describe the structure of specs but fail to document how validation rules are managed or how the system enforces 'Quality Gates' via config.
- **Action Needed**: document_validation_strategy
- **Repair Action**: Add a 'Spec Validation Contract' section to 'spec-model.md' defining how regexes map to 'Requirement Plus' nodes and how severity maps impact the workflow.

## 5. Executor Field Contract
- **Severity**: Medium
- **Type**: spec_contradiction
- **Gap**: 'spec-ir-evaluation.md' claims the 'executor' field is '100% derivable', but implementation fails to emit it in normal responses. 'code-generator-contract.md' is silent on whether 'metadata' like executors should be part of the generated contract or remains purely internal.
- **Action Needed**: clarify_metadata_emission
- **Repair Action**: Update 'code-generator-contract.md' to specify whether 'executor' information is a required response field for SDD actions.
