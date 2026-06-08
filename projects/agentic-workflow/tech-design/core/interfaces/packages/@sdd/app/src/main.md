---
id: projects-sdd-packages-sdd-app-src-main-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/main.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/main.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `dataSource` | projects/agentic-workflow/packages/@sdd/app/src/main.tsx | constant | pub | 14 |  |
| `port` | projects/agentic-workflow/packages/@sdd/app/src/main.tsx | constant | pub | 13 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { DataSourceProvider, LocalDataSource } from '@sdd/core'
import { ScoreRoutes } from './routes'

/**
 * Standalone entry point for `score view`.
 * Reads the port from the current window location (axum serves the app on that port).
 */
const port = parseInt(window.location.port, 10) || 3000
const dataSource = new LocalDataSource(port)

function App() {
  return (
    <BrowserRouter>
      <DataSourceProvider value={dataSource}>
        <div className="min-h-screen bg-gray-50">
          <nav className="bg-white border-b px-6 py-3">
            <div className="flex items-center gap-6">
              <span className="text-lg font-bold text-gray-900">Score</span>
              <a href="/tech-designs" className="text-sm text-gray-600 hover:text-gray-900">Tech Designs</a>
              <a href="/changes" className="text-sm text-gray-600 hover:text-gray-900">Changes</a>
              <a href="/issues" className="text-sm text-gray-600 hover:text-gray-900">Issues</a>
              <a href="/lineage" className="text-sm text-gray-600 hover:text-gray-900">Lineage</a>
            </div>
          </nav>
          <main className="py-6 px-4 sm:px-6 lg:px-8">
            <ScoreRoutes />
          </main>
        </div>
      </DataSourceProvider>
    </BrowserRouter>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/app/src/main.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
