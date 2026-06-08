---
change_id: sdd-p1
type: gap_spec_knowledge
created_at: 2026-02-23T14:25:44.529067+00:00
updated_at: 2026-02-23T14:25:44.529067+00:00
---

# Gap Analysis: Spec vs Knowledge (sdd-p1)

## High Severity

- **[workflow-loop-tool-filtering-mismatch] Boundary Misalignment**
  - Description: Contradiction between knowledge base's rigid stage-specific tool counts (Plan: 22, Implement: 4) and spec's requirement for a dynamic run-change DAG loop. The knowledge base does not detail how the loop interacts with filtered tool sets.

## Medium Severity

- **[yaml-ir-relevance-discrepancy] Contradiction**
  - Description: Misalignment in architectural significance of YAML Spec IR. Knowledge base (genesis-372) treats it as a critical migration to reduce token overhead, while spec_context marks it as low relevance.

- **[missing-post-clarification-knowledge] Unmatched Pattern**
  - Description: Knowledge base lacks documentation for the post-clarification phases described in specs, focusing only on Plan/Implement/Review. This reinforces the implementation gap identified in spec_context.

## Low Severity

- **[artifact-naming-contract-gap] Unmatched Pattern**
  - Description: The "Unified Artifact Management" pattern in knowledge does not explicitly enforce or document the naming convention (create_{artifact}) required by specs, leading to the naming divergence (explore_spec) noted in code.
