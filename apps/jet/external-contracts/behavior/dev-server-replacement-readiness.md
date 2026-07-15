---
id: dev-server-replacement-readiness
summary: External contract for Dev Server Replacement Readiness.
fill_sections: [e2e-test]
---

# EC: Dev Server Replacement Readiness

---
id: dev-server-replacement-readiness
summary: External contract for Dev Server Replacement Readiness.
fill_sections: [e2e-test]
---

# EC: Dev Server Replacement Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dev-server-replacement-readiness
    capability_id: dev-server-hmr
    claim_id: dev-server-replacement-readiness
    contract_id: dev-server-replacement-readiness
    category: behavior
    command: "cargo test -p jet --lib dev_server -- --nocapture"
    assertions:
      - "Jet dev server satisfies the replacement readiness suite for local serving, HMR, and proxy behavior."
```
