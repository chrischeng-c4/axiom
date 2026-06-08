// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/local-data-source.md#source
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
 * LocalDataSource -- fetches SDD data from the axum REST API served by `score view`.
 * Base URL is `http://localhost:{port}`.
 */
export class LocalDataSource implements SddDataSource {
  private baseUrl: string

  constructor(port: number) {
    this.baseUrl = `http://localhost:${port}`
  }

  private async fetch<T>(path: string): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`)
    if (!res.ok) {
      throw new Error(`LocalDataSource: ${res.status} ${res.statusText} for ${path}`)
    }
    return res.json() as Promise<T>
  }

  async listTechDesigns(): Promise<TechDesignSummary[]> {
    return this.fetch('/api/tech-designs')
  }

  async getTechDesign(id: string): Promise<TechDesign> {
    return this.fetch(`/api/tech-designs/${encodeURIComponent(id)}`)
  }

  async listChanges(): Promise<ChangeSummary[]> {
    return this.fetch('/api/changes')
  }

  async getChange(id: string): Promise<Change> {
    return this.fetch(`/api/changes/${encodeURIComponent(id)}`)
  }

  async listIssues(): Promise<IssueSummary[]> {
    return this.fetch('/api/issues')
  }

  async getIssue(id: string): Promise<Issue> {
    return this.fetch(`/api/issues/${encodeURIComponent(id)}`)
  }

  async getLineage(artifactId: string): Promise<LineageGraph> {
    return this.fetch(`/api/lineage/${encodeURIComponent(artifactId)}`)
  }

  async getInfo(): Promise<ProjectInfo> {
    return this.fetch('/api/info')
  }
}


// CODEGEN-END
