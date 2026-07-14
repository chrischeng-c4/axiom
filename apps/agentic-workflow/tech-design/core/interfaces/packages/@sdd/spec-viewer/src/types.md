---
id: projects-sdd-packages-sdd-spec-viewer-src-types-ts
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "The @sdd package interfaces are legacy implementation evidence and do not define a separate AW product surface."
---

# Standardized apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodeBlockProps` | apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts | interface | pub | 19 |  |
| `MermaidDiagramProps` | apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts | interface | pub | 12 |  |
| `SpecViewerProps` | apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts | interface | pub | 3 |  |
## Source
<!-- type: source lang: ts -->

```ts
export interface SpecViewerProps {
  /** Markdown content to render */
  content: string
  /** Optional CSS class name */
  className?: string
  /** Enable dark mode (auto-detected from prefers-color-scheme by default) */
  darkMode?: boolean
}

export interface MermaidDiagramProps {
  /** Mermaid diagram source code */
  content: string
  /** Optional CSS class name */
  className?: string
}

export interface CodeBlockProps {
  /** Code content */
  code: string
  /** Language identifier for syntax class */
  language?: string
  /** Optional CSS class name */
  className?: string
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/packages/@sdd/spec-viewer/src/types.ts
    action: create
    section: source
    impl_mode: codegen
    description: |
      Generate the complete source file from the generic Source template. This
      is a cross-language raw source template used for regenerable adoption
      before higher-level semantic generators exist.
```
