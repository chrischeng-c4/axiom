---
id: projects-sdd-packages-sdd-ui-src-layout-header-tsx
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "The @sdd package interfaces are client embedding surfaces over the AW Core workflow protocol."
---

# Standardized projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `HeaderProps` | projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx | interface | pub | 10 |  |
| `[captured, setCaptured]` | projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx | constant | pub | 16 |  |
| `[capturing, setCapturing]` | projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx | constant | pub | 15 |  |
| `handleCapture` | projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx | function | pub | 18 |  |
## Source
<!-- type: source lang: tsx -->

```tsx
/**
 * Header component with mobile menu button and capture screen
 */

import { useState } from 'react'
import { Menu, Camera, Loader2, Check } from 'lucide-react'

interface HeaderProps {
  onMenuClick?: () => void
}

export default function Header({ onMenuClick }: HeaderProps) {
  const [capturing, setCapturing] = useState(false)
  const [captured, setCaptured] = useState(false)

  const handleCapture = async () => {
    if (capturing) return
    setCapturing(true)
    try {
      const html2canvas = (await import('html2canvas')).default
      const canvas = await html2canvas(document.body)
      const blob = await new Promise<Blob>((resolve) =>
        canvas.toBlob((b) => resolve(b!), 'image/png')
      )
      await navigator.clipboard.write([
        new ClipboardItem({ 'image/png': blob }),
      ])
      setCaptured(true)
      setTimeout(() => setCaptured(false), 2000)
    } catch (err) {
      console.error('Screen capture failed:', err)
    } finally {
      setCapturing(false)
    }
  }

  return (
    <header className="bg-white shadow-sm border-b">
      <div className="px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={onMenuClick}
            aria-label="Open navigation menu"
            className="inline-flex items-center justify-center h-10 w-10 rounded-lg text-gray-500 hover:text-gray-900 hover:bg-gray-100 transition-colors cursor-pointer lg:hidden"
          >
            <Menu className="h-5 w-5" />
          </button>
          <h2 className="text-lg font-semibold text-gray-900">Conductor Dashboard</h2>
        </div>
        <div className="flex items-center space-x-4">
          <button
            onClick={handleCapture}
            disabled={capturing}
            aria-label="Capture screen"
            className="inline-flex items-center justify-center h-9 w-9 rounded-lg text-gray-500 hover:text-gray-900 hover:bg-gray-100 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {capturing ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : captured ? (
              <Check className="h-4 w-4 text-green-600" />
            ) : (
              <Camera className="h-4 w-4" />
            )}
          </button>
          <span className="text-sm text-gray-500 hidden sm:block">AI-Powered Development Automation</span>
        </div>
      </div>
    </header>
  )
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/packages/@sdd/ui/src/layout/Header.tsx
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
