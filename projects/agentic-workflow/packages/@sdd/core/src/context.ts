// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/core/src/context.md#source
// CODEGEN-BEGIN
import { createContext, useContext } from 'react'
import type { SddDataSource } from './data-source'

/**
 * React Context for providing an SddDataSource to the component tree.
 * The app root (or embedding host) must wrap the tree with DataSourceProvider.
 */
export const DataSourceContext = createContext<SddDataSource | null>(null)

export const DataSourceProvider = DataSourceContext.Provider

/**
 * Hook to consume the current SddDataSource.
 * Throws if used outside a DataSourceProvider.
 */
export function useDataSource(): SddDataSource {
  const ds = useContext(DataSourceContext)
  if (!ds) {
    throw new Error(
      'useDataSource() must be used within a <DataSourceProvider>. ' +
      'Wrap your component tree with DataSourceProvider and pass an SddDataSource implementation.'
    )
  }
  return ds
}


// CODEGEN-END
