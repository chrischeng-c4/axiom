---
number: 1052
title: "sdd: design system as tech-stack config + UX pattern library"
state: open
labels: [type:enhancement, priority:p2, crate:sdd]
group: "frontend-design-system"
---

# #1052 — sdd: design system as tech-stack config + UX pattern library

Parent: #1051

## Problem

Frontend spec density is too high for custom components. Design systems (MUI, Antd) already define most of the structure, but SDD treats them as implementation detail.

## Proposal

### 1. Design system in tech_stack config

```yaml
tech_stack:
  frontend:
    framework: react
    design_system:
      library: mui | antd
      ux_patterns: true | false
```

- `ux_patterns: true` (MUI) — `wireframe` can be thin (`layout: dashboard-with-drawer`), generator uses built-in recipes
- `ux_patterns: false` (Antd) — `wireframe` must describe layout structure explicitly
- When `ux_patterns: true`, `design-token` and `component` section types become optional

### 2. SDD-side UX pattern library (deferred)

Design-system-agnostic layout patterns (e.g., `dashboard-with-drawer`, `form-with-stepper`, `crud-table`). Generator translates to library-specific components.

This decouples spec authoring from component library choice.

## Skeleton output

```yaml
# wireframe input
page: order-list
layout: dashboard-with-drawer
sections:
  - slot: main
    component: DataTable
    props:
      columns: [id, customer, amount, status]
```

Generator (knowing MUI or Antd) produces complete page skeleton with correct imports, layout components, and data binding hooks.
