---
id: phase-3-build-gate
summary: External contract for Phase 3 Build Gate.
fill_sections: [e2e-test]
---

# EC: Phase 3 Build Gate

---
id: phase-3-build-gate
summary: External contract for Phase 3 Build Gate.
fill_sections: [e2e-test]
---

# EC: Phase 3 Build Gate

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: phase-3-build-gate
    capability_id: bundler-production-build
    claim_id: bundler-production-readiness
    contract_id: phase-3-build-gate
    category: behavior
    command: "apps/jet/scripts/verify-basic-dom-gates.sh --phase build"
    assertions:
      - "The production build verification phase passes its complete required gate set."
```
