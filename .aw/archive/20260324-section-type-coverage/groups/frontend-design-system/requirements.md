---
change: section-type-coverage
group: frontend-design-system
date: 2026-03-24
---

# Requirements

Add design system awareness to the SDD codegen pipeline. This is NOT about adding new section types — it's about how existing types (wireframe, component, design-token) behave based on tech_stack config.

1. **tech_stack config schema** — add `design_system` field with `library` (mui/antd/etc) and `ux_patterns` (bool)
2. **Wireframe behavior** — when `ux_patterns: true`, wireframe can use shorthand layout references (e.g., `layout: dashboard-with-drawer`); when false, must describe layout structure explicitly
3. **Section optionality** — when design system covers tokens and components, `design-token` and `component` sections become optional in section selection
4. **UX pattern library** — deferred, but spec should define the extension point

This modifies section selection logic (which sections are required) and wireframe spec format, not the section type table itself.
