---
change: section-type-coverage
group: new-section-types
date: 2026-03-24
---

# Requirements

Extend the SDD section type system with new types for underserved roles. All changes are additive to the same spec/code area:

1. **Section Type → Spec Lang Mapping** in change-spec.md — add 16 new types
2. **Section rules** (keyword matching) — add match patterns for each new type
3. **Fill order** — assign priority positions for each new type
4. **CLI generator flags** (section_prompts) — define per-type flags for artifact CLI
5. **Cross-reference system** — ensure new types follow existing id/ref conventions

New types by role:
- QA: `e2e-scenario` (yaml, p1), `test-fixture` (json, p3), `perf-test` (yaml, p3)
- Security: `threat-model` (yaml, p2), `auth-matrix` (yaml, p2), `security-test` (yaml, p2)
- SRE: `container` (yaml, p3), `deploy` (yaml, p3), `cloud-resource` (yaml, p3), `pipeline` (yaml, p3), `observability` (yaml, p3)
- Backend: `grpc` (protobuf, p3), `graphql` (graphql, p3)
- MLE: `model` (yaml, p3)
- Agent: `prompt` (markdown, p3)

All types follow the same pattern: type annotation `<!-- type: X lang: Y -->`, code fence, cross-ref via standard mechanisms.
