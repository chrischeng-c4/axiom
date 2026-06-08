---
number: 1053
title: "sdd: add e2e-scenario section type for QA"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "new-section-types"
---

# #1053 — sdd: add e2e-scenario section type for QA

Parent: #1051

## Problem

QA needs to spec user-journey-level test scenarios (E2E). Current `test-plan` (requirementDiagram) only covers requirement traceability, not step-by-step actions + assertions.

## Proposal

New section type: `e2e-scenario`

| field | value |
|-------|-------|
| lang | `yaml` |
| code fence | ` ```yaml ` |
| skeleton output | Playwright / Cypress test file |

### Example spec content

```yaml
scenario: order-checkout-happy-path
steps:
  - action: navigate
    target: /orders/new
  - action: fill
    fields: { customer: "test-user", items: [{ sku: "A1", qty: 2 }] }
  - action: click
    target: submit
  - assert: toast
    text: "Order created"
  - assert: redirect
    to: /orders/{id}
```

### Section rule

```yaml
- match: "e2e|scenario|user journey|acceptance test|playwright|cypress"
  sections: [e2e-scenario]
```

### Fill order

After `test-plan` (priority 10.5) — needs all other sections to understand the full feature.
