---
id: projects-sdd-packages-sdd-ui-src-primitives-badge-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/primitives/badge.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/primitives/badge.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Badge` | projects/agentic-workflow/packages/@sdd/ui/src/primitives/badge.tsx | constant | pub | 14 |  |
| `BadgeProps` | projects/agentic-workflow/packages/@sdd/ui/src/primitives/badge.tsx | interface | pub | 10 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
/**
 * Badge component
 */

import React from 'react'
import { cn } from '../lib/utils'

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'success' | 'warning' | 'destructive'
}

const Badge = React.forwardRef<HTMLDivElement, BadgeProps>(
  ({ className, variant = 'default', ...props }, ref) => {
    const variants = {
      default: 'bg-primary text-primary-foreground',
      secondary: 'bg-secondary text-secondary-foreground',
      success: 'bg-green-500 text-white',
      warning: 'bg-yellow-500 text-white',
      destructive: 'bg-destructive text-destructive-foreground',
    }

    return (
      <div
        ref={ref}
        className={cn(
          'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors',
          variants[variant],
          className
        )}
        {...props}
      />
    )
  }
)
Badge.displayName = 'Badge'

export { Badge }

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/primitives/badge.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
