---
id: projects-sdd-packages-sdd-core-src-remote-data-source-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/core/src/remote-data-source.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/core/src/remote-data-source.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RemoteDataSource` | projects/agentic-workflow/packages/@sdd/core/src/remote-data-source.ts | class | pub | 20 |  |
## Source
<!-- type: source lang: ts -->

```ts
import type { SddDataSource } from './data-source'
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
 * RemoteDataSource -- fetches SDD data from the Conductor REST API.
 * Base URL is `/api/projects/{projectId}`.
 * Used when @sdd/app is mounted inside the Conductor FE.
 */
export class RemoteDataSource implements SddDataSource {
  private baseUrl: string

  constructor(projectId: string) {
    this.baseUrl = `/api/projects/${encodeURIComponent(projectId)}`
  }

  private async fetch<T>(path: string): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`)
    if (!res.ok) {
      throw new Error(`RemoteDataSource: ${res.status} ${res.statusText} for ${path}`)
    }
    return res.json() as Promise<T>
  }

  async listTechDesigns(): Promise<TechDesignSummary[]> {
    return this.fetch('/tech-designs')
  }

  async getTechDesign(id: string): Promise<TechDesign> {
    return this.fetch(`/tech-designs/${encodeURIComponent(id)}`)
  }

  async listChanges(): Promise<ChangeSummary[]> {
    return this.fetch('/changes')
  }

  async getChange(id: string): Promise<Change> {
    return this.fetch(`/changes/${encodeURIComponent(id)}`)
  }

  async listIssues(): Promise<IssueSummary[]> {
    return this.fetch('/issues')
  }

  async getIssue(id: string): Promise<Issue> {
    return this.fetch(`/issues/${encodeURIComponent(id)}`)
  }

  async getLineage(artifactId: string): Promise<LineageGraph> {
    return this.fetch(`/lineage/${encodeURIComponent(artifactId)}`)
  }

  async getInfo(): Promise<ProjectInfo> {
    return this.fetch('/info')
  }
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/core/src/remote-data-source.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
