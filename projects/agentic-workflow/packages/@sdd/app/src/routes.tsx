// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/packages/@sdd/app/src/routes.md#source
// CODEGEN-BEGIN
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


// CODEGEN-END
