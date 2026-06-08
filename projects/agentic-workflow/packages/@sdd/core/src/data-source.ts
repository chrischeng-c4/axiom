// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/data-source.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
