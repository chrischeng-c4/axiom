---
id: projects-sdd-packages-sdd-pipeline-src-index-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/pipeline/src/index.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/pipeline/src/index.ts` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: ts -->

```ts
export { default as PipelineDAG } from './PipelineDAG'
export { default as PipelineNodeCard } from './PipelineNode'
export { default as NodeDetail } from './NodeDetail'
export type {
  PipelineNode,
  PipelineJob,
  NodeStatus,
  PipelineDAGProps,
  NodeDetailProps,
} from './types'

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/pipeline/src/index.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
