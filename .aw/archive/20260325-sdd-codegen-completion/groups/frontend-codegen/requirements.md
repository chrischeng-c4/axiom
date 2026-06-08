---
change: sdd-codegen-completion
group: frontend-codegen
date: 2026-03-20
---

# Requirements

Implement validators and code generators for the three frontend section types: wireframe (YAML DSL), component (CEM JSON), and design-token (DTCG JSON). Deliverables: (1) DTCG design tokens → CSS custom properties and/or Tailwind config output; (2) CEM component definition → TypeScript prop interface + component skeleton; (3) wireframe YAML → React/Vue/Svelte component tree scaffold (one framework target required). Cross-section composition rule: wireframe + component + design-token → complete UI component with typed props and applied tokens. Cross-ref to rest-api section enables data fetching hook generation in the component skeleton. Validators for all three section types are required before any generator is wired.
