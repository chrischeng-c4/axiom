---
id: multi-fixture-dom-wasm-layout-parity
summary: External contract for Multi Fixture Dom Wasm Layout Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Layout Parity

---
id: multi-fixture-dom-wasm-layout-parity
summary: External contract for Multi Fixture Dom Wasm Layout Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Layout Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-fixture-dom-wasm-layout-parity
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: multi-fixture-dom-wasm-layout-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_layout_parity -- --nocapture"
    assertions:
      - "static fixture initial DOM/WASM layout boxes match"
      - "class/state fixture initial and after-click DOM/WASM layout boxes match"
      - "list/state fixture initial and after-click DOM/WASM layout boxes match"
      - "layout mismatches include fixture id, phase, expected, and actual JSON"
```
