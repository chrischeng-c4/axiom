// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/index.md#source
// CODEGEN-BEGIN
// @sdd/core -- barrel export

// Types
export type {
  TechDesign,
  TechDesignSummary,
  Change,
  ChangeSummary,
  ChangePhase,
  Issue,
  IssueSummary,
  IssueStatus,
  IssuePriority,
  LineageGraph,
  LineageNode,
  LineageEdge,
  LineageNodeKind,
  ProjectInfo,
} from './types'

// Data source interface
export type { SddDataSource } from './data-source'

// Data source implementations
export { LocalDataSource } from './local-data-source'
export { RemoteDataSource } from './remote-data-source'

// React context
export { DataSourceContext, DataSourceProvider, useDataSource } from './context'


// CODEGEN-END
