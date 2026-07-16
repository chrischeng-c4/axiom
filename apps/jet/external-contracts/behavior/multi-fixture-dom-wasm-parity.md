---
id: multi-fixture-dom-wasm-parity
summary: External contract for Multi Fixture Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Parity

---
id: multi-fixture-dom-wasm-parity
summary: External contract for Multi Fixture Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-fixture-dom-wasm-parity
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: multi-fixture-dom-wasm-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_parity -- --nocapture"
    assertions:
      - "static fixture initial DOM/WASM host trees match"
      - "class/list fixture initial DOM/WASM host trees match"
      - "stateful fixture initial and after-click DOM/WASM host trees match"
      - "stateful WASM bundle includes the updated hook value"
```
