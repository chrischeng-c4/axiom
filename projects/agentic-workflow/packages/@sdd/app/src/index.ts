// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/index.md#source
// CODEGEN-BEGIN
// @sdd/app -- library entry point for embedding in Conductor FE

export { ScoreRoutes, scoreRoutes } from './routes'

// Re-export core types and context for convenience
export { DataSourceProvider, useDataSource } from '@sdd/core'
export type { SddDataSource } from '@sdd/core'


// CODEGEN-END
