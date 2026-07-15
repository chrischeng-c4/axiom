---
id: full-jet-health-after-unit-repair
summary: External contract for Full Jet Health After Unit Repair.
fill_sections: [e2e-test]
---

# EC: Full Jet Health After Unit Repair

---
id: full-jet-health-after-unit-repair
summary: External contract for Full Jet Health After Unit Repair.
fill_sections: [e2e-test]
---

# EC: Full Jet Health After Unit Repair

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: full-jet-health-after-unit-repair
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: full-jet-health-after-unit-repair
    category: behavior
    command: "cargo test -p jet-wasm --test renderer_layout -- --nocapture"
    assertions:
      - "test gate no longer fails on apps/jet/wasm/tests/renderer_layout.rs stale v0 expectations"
      - "renderer_layout remains a required test gate with zero ignored cases"
```
