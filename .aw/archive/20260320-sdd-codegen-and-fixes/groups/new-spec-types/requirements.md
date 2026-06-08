---
change: sdd-codegen-and-fixes
group: new-spec-types
date: 2026-03-20
---

# Requirements

Extend the SDD section type system with two new additions. (1) Deploy section type (#934): add a `deploy` section type for infra-as-code concerns currently absent from SDD (containers, k8s manifests, migrations, env vars, rollback strategy, resource requirements). Define the YAML DSL (services: name/image/replicas/env/ports/health_check, migrations: type/path/run_before, rollback: strategy/auto_rollback_on). Add to section_rules matcher (deploy|container|k8s|docker|helm|terraform|infra|migration|rollback). Implement CLI flags for the deploy section. Add validation: referenced services/migrations are internally consistent, cross-ref to db-model for migration validation, cross-ref to rest-api for port/route validation. Implement at least one codegen target (docker-compose YAML or k8s manifest). (2) Frontend codegen for wireframe/component/design-token section types (#937): all three schema definitions exist (wireframe YAML DSL, CEM JSON for component, W3C DTCG 2025.10 JSON for design-token) but have no validators or code generators. Phase 1: wireframe YAML → React component tree scaffold (page layout, form fields, nav routes). Phase 2: DTCG design tokens → CSS custom properties and/or Tailwind config object. Phase 3: CEM attributes/events/slots/CSS parts → TypeScript prop interface + component skeleton. Cross-section composition: wireframe + component + design-token → complete UI component; cross-ref to rest-api for data fetching hook generation. Both items extend the crate:sdd section type system and can share the section registration and validation infrastructure.
