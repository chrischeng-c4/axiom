---
id: projects-sdd-packages-sdd-app-src-routes-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/app/src/routes.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/app/src/routes.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: tsx -->

```tsx
import { Routes, Route, Navigate } from 'react-router-dom'
import type { RouteObject } from 'react-router-dom'
import TechDesignList from './pages/TechDesignList'
import TechDesignDetail from './pages/TechDesignDetail'
import ChangeList from './pages/ChangeList'
import ChangeDetail from './pages/ChangeDetail'
import IssueList from './pages/IssueList'
import IssueDetail from './pages/IssueDetail'
import Lineage from './pages/Lineage'

/**
 * Route definitions as RouteObject[] for programmatic use (e.g., Conductor embedding).
 */
export const scoreRoutes: RouteObject[] = [
  { index: true, element: <Navigate to="tech-designs" replace /> },
  { path: 'tech-designs', element: <TechDesignList /> },
  { path: 'tech-designs/:id', element: <TechDesignDetail /> },
  { path: 'changes', element: <ChangeList /> },
  { path: 'changes/:id', element: <ChangeDetail /> },
  { path: 'issues', element: <IssueList /> },
  { path: 'issues/:id', element: <IssueDetail /> },
  { path: 'lineage', element: <Lineage /> },
]

/**
 * ScoreRoutes -- renders the SDD route tree as a <Routes> block.
 * Use this in standalone mode (score view) or embed inside another Router.
 */
export function ScoreRoutes() {
  return (
    <Routes>
      <Route index element={<Navigate to="tech-designs" replace />} />
      <Route path="tech-designs" element={<TechDesignList />} />
      <Route path="tech-designs/:id" element={<TechDesignDetail />} />
      <Route path="changes" element={<ChangeList />} />
      <Route path="changes/:id" element={<ChangeDetail />} />
      <Route path="issues" element={<IssueList />} />
      <Route path="issues/:id" element={<IssueDetail />} />
      <Route path="lineage" element={<Lineage />} />
    </Routes>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/app/src/routes.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
