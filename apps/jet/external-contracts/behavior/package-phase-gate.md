---
id: package-phase-gate
summary: External contract for Package Phase Gate.
fill_sections: [e2e-test]
---

# EC: Package Phase Gate

---
id: package-phase-gate
summary: External contract for Package Phase Gate.
fill_sections: [e2e-test]
---

# EC: Package Phase Gate

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: package-phase-gate
    capability_id: package-manager
    claim_id: package-manager-readiness
    contract_id: package-phase-gate
    category: behavior
    command: "apps/jet/scripts/verify-basic-dom-gates.sh --phase package"
    assertions:
      - "The package-manager verification phase passes its complete required gate set."
```
