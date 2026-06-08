---
id: projects-sdd-packages-sdd-core-src-context-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/core/src/context.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/core/src/context.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DataSourceContext` | projects/agentic-workflow/packages/@sdd/core/src/context.ts | constant | pub | 10 |  |
| `DataSourceProvider` | projects/agentic-workflow/packages/@sdd/core/src/context.ts | constant | pub | 12 |  |
| `useDataSource` | projects/agentic-workflow/packages/@sdd/core/src/context.ts | function | pub | 18 | useDataSource() |
## Source
<!-- type: source lang: ts -->

```ts
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

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/core/src/context.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
