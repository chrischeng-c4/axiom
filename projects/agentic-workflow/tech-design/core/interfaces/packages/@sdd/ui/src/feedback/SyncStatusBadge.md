---
id: projects-sdd-packages-sdd-ui-src-feedback-syncstatusbadge-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SyncStatusBadge` | projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx | function | pub | 17 | SyncStatusBadge({ status, className }: SyncStatusBadgeProps) |
| `SyncStatusBadgeProps` | projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx | interface | pub | 12 |  |
| `statusConfig` | projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx | constant | pub | 6 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
import { cn } from '../lib/utils'
import { CheckCircle, Clock, AlertCircle, RefreshCw } from 'lucide-react'

const statusConfig: Record<string, { label: string; className: string; icon: typeof CheckCircle }> = {
  synced: { label: 'Synced', className: 'bg-green-100 text-green-800', icon: CheckCircle },
  pending: { label: 'Pending', className: 'bg-yellow-100 text-yellow-800', icon: Clock },
  failed: { label: 'Failed', className: 'bg-red-100 text-red-800', icon: AlertCircle },
}

interface SyncStatusBadgeProps {
  status: string
  className?: string
}

export default function SyncStatusBadge({ status, className }: SyncStatusBadgeProps) {
  const config = statusConfig[status] || { label: status, className: 'bg-gray-100 text-gray-700', icon: RefreshCw }
  const Icon = config.icon

  return (
    <span className={cn(
      'inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold',
      config.className,
      className,
    )}>
      <Icon className="h-3 w-3" />
      {config.label}
    </span>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/feedback/SyncStatusBadge.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
