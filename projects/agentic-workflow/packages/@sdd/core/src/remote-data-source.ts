// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/remote-data-source.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
