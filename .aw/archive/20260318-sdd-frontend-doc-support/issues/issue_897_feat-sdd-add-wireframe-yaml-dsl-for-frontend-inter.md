---
number: 897
title: "feat(sdd): add wireframe YAML DSL for frontend interface specs"
state: open
labels: [enhancement, crate:sdd]
group: "sdd-frontend-doc-artifacts"
---

# #897 — feat(sdd): add wireframe YAML DSL for frontend interface specs

## Summary

SDD workflow currently has formal DSLs for backend interfaces (OpenAPI, OpenRPC, AsyncAPI, JSON Schema, Serverless Workflow) but nothing equivalent for frontend/UI changes. Frontend specs fall back to prose, losing machine-readable → codegen capability.

## Proposal

Add a wireframe-level YAML DSL that describes UI structure + interaction intent (not visual styling):

- **Granularity**: wireframe (structure + semantics), not pixel-perfect design
- **Vocabulary**: framework-agnostic primitives (`stack`, `form`, `nav-list`, `heading`, `action-group`...)
- **Validation**: JSON Schema for the YAML format, consistent with existing SDD validation pipeline
- **Future**: `$ref` to component catalog once built

Example:
```yaml
page: settings
route: /settings
layout: sidebar-detail
sections:
  - id: nav
    type: nav-list
    items:
      - label: General
        route: /settings/general
  - id: content
    type: stack
    children:
      - type: form
        fields:
          - name: theme
            input: select
            options: [light, dark, system]
```

## Placement in SDD

Agent senses frontend changes → fills wireframe YAML into `ui_spec` section (or `api_spec` with type marker). No new `spec_type` needed.
