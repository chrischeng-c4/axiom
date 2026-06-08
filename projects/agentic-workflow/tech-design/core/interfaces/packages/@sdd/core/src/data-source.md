---
id: projects-sdd-packages-sdd-core-src-data-source-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/core/src/data-source.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/core/src/data-source.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SddDataSource` | projects/agentic-workflow/packages/@sdd/core/src/data-source.ts | interface | pub | 21 |  |
## Source
<!-- type: source lang: ts -->

```ts
import type {
  TechDesignSummary,
  TechDesign,
  ChangeSummary,
  Change,
  IssueSummary,
  Issue,
  LineageGraph,
  ProjectInfo,
} from './types'

/**
 * Abstract data-access interface for the SDD frontend.
 *
 * Two implementations:
 * - LocalDataSource  -- calls the axum REST API at localhost (score view)
 * - RemoteDataSource -- calls the Conductor REST API (Conductor FE)
 */
export interface SddDataSource {
  listTechDesigns(): Promise<TechDesignSummary[]>
  getTechDesign(id: string): Promise<TechDesign>
  listChanges(): Promise<ChangeSummary[]>
  getChange(id: string): Promise<Change>
  listIssues(): Promise<IssueSummary[]>
  getIssue(id: string): Promise<Issue>
  getLineage(artifactId: string): Promise<LineageGraph>
  getInfo(): Promise<ProjectInfo>
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/core/src/data-source.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
