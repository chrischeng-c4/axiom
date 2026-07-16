---
id: component-workbench-readiness
summary: External contract for Component Workbench Readiness.
fill_sections: [e2e-test]
---

# EC: Component Workbench Readiness

---
id: component-workbench-readiness
summary: External contract for Component Workbench Readiness.
fill_sections: [e2e-test]
---

# EC: Component Workbench Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: component-workbench-readiness
    capability_id: component-workbench
    claim_id: component-workbench-readiness
    contract_id: component-workbench-readiness
    category: behavior
    command: "cargo test -p jet --test stories_build -- --nocapture"
    assertions:
      - "The component workbench builds a usable static story surface with required manager and preview behavior."
```
