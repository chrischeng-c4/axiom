---
number: 937
title: "feat(sdd): frontend codegen — wireframe + component + design-token → UI code"
state: open
labels: [enhancement, P2, crate:sdd]
group: "frontend-codegen"
---

# #937 — feat(sdd): frontend codegen — wireframe + component + design-token → UI code

## Summary

SDD has three frontend-related section types (`wireframe`, `component`, `design-token`) with defined schemas:
- **wireframe**: YAML DSL with framework-agnostic primitives
- **component**: Custom Elements Manifest (CEM) JSON
- **design-token**: W3C DTCG 2025.10 JSON

Plus #897 proposes a wireframe YAML DSL. However, none of these have codegen templates or cross-section composition rules for frontend frameworks.

## Current State

| Section Type | Schema | Validator | Generator (Mermaid+) | Codegen → UI Code |
|---|---|---|---|---|
| wireframe | YAML DSL defined | ❌ | N/A (not Mermaid) | ❌ |
| component | CEM JSON defined | ❌ | N/A (not Mermaid) | ❌ |
| design-token | DTCG JSON defined | ❌ | N/A (not Mermaid) | ❌ |

## Proposal

### Phase 1: Wireframe → component tree scaffold

```
wireframe YAML (page layout + sections + form fields)
    ↓
React/Vue/Svelte component tree scaffold
    ↓
- Page component with layout
- Form component with typed fields
- Nav component with routes
```

### Phase 2: Design-token → theme/style system

```
DTCG design tokens (colors, spacing, typography)
    ↓
CSS custom properties / Tailwind config / theme object
```

### Phase 3: Component → typed props + event interface

```
CEM (attributes, events, slots, CSS parts)
    ↓
TypeScript interface + component skeleton
```

### Cross-section composition

Frontend codegen needs:
- `wireframe` (structure) + `component` (contract) + `design-token` (style) → complete UI component
- Cross-ref to `rest-api` for data fetching hooks
- Cross-ref to `schema` for form validation types

## Acceptance Criteria

- [ ] At least one frontend target (React recommended) produces component skeleton from wireframe
- [ ] Design tokens generate CSS custom properties or Tailwind config
- [ ] Component CEM generates TypeScript prop interface
- [ ] Cross-ref to rest-api enables data fetching hook generation
